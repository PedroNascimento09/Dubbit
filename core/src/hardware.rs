use std::process::{Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Gpu {
    pub vendor: String,
    pub name: String,
    pub memory_mb: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Hardware {
    pub os_name: String,
    pub os_version: String,
    pub arch: String,
    pub cpu_cores: usize,
    pub ram_mb: u64,
    pub gpus: Vec<Gpu>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Profile {
    pub name: String,
    pub asr_model: String,
    pub threads: usize,
    pub ram_budget_mb: u64,
    pub realtime_factor: f64,
    pub reason: String,
}

impl Hardware {
    pub fn nvidia(&self) -> Option<&Gpu> {
        self.gpus.iter().find(|gpu| gpu.vendor == "nvidia")
    }

    pub fn apple_silicon(&self) -> bool {
        self.os_name == "macos" && matches!(self.arch.as_str(), "aarch64" | "arm64")
    }

    pub fn summary(&self) -> String {
        let mut parts = vec![format!("{} {}", self.os_name, self.os_version)
            .trim()
            .to_string()];
        parts.push(format!("{} cores", self.cpu_cores));
        parts.push(format!("{} GB RAM", self.ram_mb / 1024));
        if self.apple_silicon() {
            parts.push("Apple Silicon".to_string());
        }
        for gpu in &self.gpus {
            if gpu.vendor == "apple" {
                continue;
            }
            match gpu.memory_mb {
                Some(memory) => parts.push(format!("{} ({} GB)", gpu.name, memory / 1024)),
                None => parts.push(gpu.name.clone()),
            }
        }
        parts.retain(|part| !part.is_empty());
        parts.join(", ")
    }
}

fn run_with_timeout(mut command: Command, timeout: Duration) -> Option<Output> {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().ok()?;
    let started = Instant::now();
    loop {
        if child.try_wait().ok()?.is_some() {
            return child.wait_with_output().ok();
        }
        if started.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            return None;
        }
        thread::sleep(Duration::from_millis(25));
    }
}

fn stdout_of(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn total_ram_mb() -> u64 {
    if cfg!(windows) {
        let mut command = Command::new("powershell");
        command.args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "(Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory",
        ]);
        let bytes = run_with_timeout(command, Duration::from_secs(20))
            .map(|output| stdout_of(&output))
            .and_then(|text| text.parse::<u64>().ok())
            .unwrap_or(0);
        return bytes / (1024 * 1024);
    }
    if cfg!(target_os = "macos") {
        let mut command = Command::new("sysctl");
        command.args(["-n", "hw.memsize"]);
        let bytes = run_with_timeout(command, Duration::from_secs(5))
            .map(|output| stdout_of(&output))
            .and_then(|text| text.parse::<u64>().ok())
            .unwrap_or(0);
        return bytes / (1024 * 1024);
    }
    std::fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|text| {
            text.lines()
                .find(|line| line.starts_with("MemTotal:"))
                .and_then(|line| line.split_whitespace().nth(1))
                .and_then(|value| value.parse::<u64>().ok())
        })
        .map(|kilobytes| kilobytes / 1024)
        .unwrap_or(0)
}

fn os_version() -> String {
    if cfg!(windows) {
        let mut command = Command::new("cmd");
        command.args(["/C", "ver"]);
        return run_with_timeout(command, Duration::from_secs(5))
            .map(|output| stdout_of(&output))
            .map(|text| {
                text.trim_start_matches("Microsoft Windows [Version ")
                    .trim_end_matches(']')
                    .to_string()
            })
            .unwrap_or_default();
    }
    if cfg!(target_os = "macos") {
        let mut command = Command::new("sw_vers");
        command.args(["-productVersion"]);
        return run_with_timeout(command, Duration::from_secs(5))
            .map(|output| stdout_of(&output))
            .unwrap_or_default();
    }
    let mut command = Command::new("uname");
    command.arg("-r");
    run_with_timeout(command, Duration::from_secs(5))
        .map(|output| stdout_of(&output))
        .unwrap_or_default()
}

fn vendor_of(name: &str) -> String {
    let lowered = name.to_lowercase();
    if lowered.contains("nvidia") {
        "nvidia".to_string()
    } else if lowered.contains("amd") || lowered.contains("radeon") {
        "amd".to_string()
    } else if lowered.contains("intel") {
        "intel".to_string()
    } else if lowered.contains("apple") {
        "apple".to_string()
    } else {
        "other".to_string()
    }
}

fn nvidia_gpus() -> Vec<Gpu> {
    let mut command = Command::new("nvidia-smi");
    command.args([
        "--query-gpu=name,memory.total",
        "--format=csv,noheader,nounits",
    ]);
    let Some(output) = run_with_timeout(command, Duration::from_secs(6)) else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }
    stdout_of(&output)
        .lines()
        .filter_map(|line| {
            let mut parts = line.split(',').map(str::trim);
            let name = parts.next()?;
            if name.is_empty() {
                return None;
            }
            let memory = parts.next().and_then(|value| value.parse::<u64>().ok());
            Some(Gpu {
                vendor: "nvidia".to_string(),
                name: name.to_string(),
                memory_mb: memory,
            })
        })
        .collect()
}

fn windows_display_gpus() -> Vec<Gpu> {
    if !cfg!(windows) {
        return Vec::new();
    }
    let mut command = Command::new("powershell");
    command.args([
        "-NoProfile",
        "-NonInteractive",
        "-Command",
        "Get-CimInstance Win32_VideoController | Select-Object -ExpandProperty Name",
    ]);
    let Some(output) = run_with_timeout(command, Duration::from_secs(12)) else {
        return Vec::new();
    };
    stdout_of(&output)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|name| Gpu {
            vendor: vendor_of(name),
            name: name.to_string(),
            memory_mb: None,
        })
        .collect()
}

pub fn detect() -> Hardware {
    let mut gpus = nvidia_gpus();
    if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
        gpus.push(Gpu {
            vendor: "apple".to_string(),
            name: "Apple Silicon".to_string(),
            memory_mb: None,
        });
    }
    for gpu in windows_display_gpus() {
        if !gpus
            .iter()
            .any(|existing| existing.vendor == gpu.vendor && existing.name == gpu.name)
        {
            gpus.push(gpu);
        }
    }
    Hardware {
        os_name: std::env::consts::OS.to_string(),
        os_version: os_version(),
        arch: std::env::consts::ARCH.to_string(),
        cpu_cores: thread::available_parallelism()
            .map(|value| value.get())
            .unwrap_or(1),
        ram_mb: total_ram_mb(),
        gpus,
    }
}

fn threads_for(hardware: &Hardware) -> usize {
    hardware.cpu_cores.saturating_sub(1).clamp(1, 8)
}

fn light(hardware: &Hardware, reason: &str) -> Profile {
    Profile {
        name: "light".to_string(),
        asr_model: "whisper-tiny-q5_1".to_string(),
        threads: threads_for(hardware),
        ram_budget_mb: 4096,
        realtime_factor: 1.6,
        reason: reason.to_string(),
    }
}

fn balanced(hardware: &Hardware, reason: &str) -> Profile {
    Profile {
        name: "balanced".to_string(),
        asr_model: "whisper-small-q5_1".to_string(),
        threads: threads_for(hardware),
        ram_budget_mb: 8192,
        realtime_factor: 0.9,
        reason: reason.to_string(),
    }
}

fn maximum(hardware: &Hardware, reason: &str) -> Profile {
    Profile {
        name: "max".to_string(),
        asr_model: "whisper-large-v3-turbo-q5_0".to_string(),
        threads: threads_for(hardware),
        ram_budget_mb: 16384,
        realtime_factor: 0.3,
        reason: reason.to_string(),
    }
}

pub fn resolve_profile(hardware: &Hardware, requested: &str) -> Profile {
    match requested.trim().to_lowercase().as_str() {
        "light" => light(hardware, "requested"),
        "balanced" => balanced(hardware, "requested"),
        "max" => maximum(hardware, "requested"),
        _ => {
            let vram = hardware.nvidia().and_then(|gpu| gpu.memory_mb).unwrap_or(0);
            if hardware.nvidia().is_some() && vram >= 12000 && hardware.ram_mb >= 16000 {
                maximum(hardware, "large GPU and at least 16 GB of RAM")
            } else if hardware.apple_silicon() || (hardware.nvidia().is_some() && vram >= 6000) {
                balanced(hardware, "capable hardware detected")
            } else {
                light(hardware, "CPU-only or small GPU")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hardware(cores: usize, ram_mb: u64, gpus: Vec<Gpu>) -> Hardware {
        Hardware {
            os_name: "linux".to_string(),
            os_version: "test".to_string(),
            arch: "x86_64".to_string(),
            cpu_cores: cores,
            ram_mb,
            gpus,
        }
    }

    #[test]
    fn modest_machine_uses_light_profile() {
        let profile = resolve_profile(&hardware(8, 8192, Vec::new()), "auto");
        assert_eq!(profile.name, "light");
        assert_eq!(profile.asr_model, "whisper-tiny-q5_1");
        assert_eq!(profile.threads, 7);
    }

    #[test]
    fn midrange_gpu_uses_balanced_profile() {
        let gpus = vec![Gpu {
            vendor: "nvidia".to_string(),
            name: "Test GPU".to_string(),
            memory_mb: Some(8192),
        }];
        let profile = resolve_profile(&hardware(16, 16384, gpus), "auto");
        assert_eq!(profile.name, "balanced");
        assert_eq!(profile.threads, 8);
    }

    #[test]
    fn large_machine_uses_max_profile() {
        let gpus = vec![Gpu {
            vendor: "nvidia".to_string(),
            name: "Test GPU".to_string(),
            memory_mb: Some(16384),
        }];
        let profile = resolve_profile(&hardware(16, 32768, gpus), "auto");
        assert_eq!(profile.name, "max");
        assert_eq!(profile.asr_model, "whisper-large-v3-turbo-q5_0");
    }

    #[test]
    fn requested_profile_wins() {
        let profile = resolve_profile(&hardware(8, 8192, Vec::new()), "max");
        assert_eq!(profile.name, "max");
        assert_eq!(profile.reason, "requested");
    }

    #[test]
    fn summary_has_core_counts() {
        let summary = hardware(8, 8192, Vec::new()).summary();
        assert!(summary.contains("8 cores"));
        assert!(summary.contains("8 GB RAM"));
    }
}

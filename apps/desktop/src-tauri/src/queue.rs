use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tauri::{AppHandle, Emitter};

use dubbit_core::config::{Settings, TranslatorChoice};
use dubbit_core::error::DubbitError;
use dubbit_core::models;
use dubbit_core::paths;
use dubbit_core::pipeline::{self, Event, Request};

use crate::view::{ErrorView, Job, JobStatus, NewJob};

const HISTORY: usize = 40;

pub struct Queue {
    jobs: Mutex<Vec<Job>>,
    wake: Condvar,
    cancels: Mutex<HashMap<u64, Arc<AtomicBool>>>,
    next_id: AtomicU64,
    app: AppHandle,
    file: PathBuf,
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_secs())
        .unwrap_or(0)
}

fn human(bytes: u64) -> String {
    let megabytes = bytes as f64 / (1024.0 * 1024.0);
    if megabytes >= 1024.0 {
        format!("{:.1} GB", megabytes / 1024.0)
    } else {
        format!("{megabytes:.0} MB")
    }
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| path.display().to_string())
}

fn output_for(input: &Path) -> PathBuf {
    let stem = input
        .file_stem()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| "project".to_string());
    let parent = input.parent().unwrap_or_else(|| Path::new("."));
    let mut candidate = parent.join(format!("{stem}.dubbit"));
    let mut counter = 2;
    while candidate.exists() {
        candidate = parent.join(format!("{stem} ({counter}).dubbit"));
        counter += 1;
    }
    candidate
}

fn load(file: &Path) -> Vec<Job> {
    let mut jobs: Vec<Job> = std::fs::read_to_string(file)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default();
    for job in jobs.iter_mut() {
        if job.status == JobStatus::Running {
            job.status = JobStatus::Queued;
            job.stage = None;
            job.detail = None;
            job.fraction = None;
        }
    }
    jobs
}

impl Queue {
    pub fn start(app: AppHandle) -> Arc<Self> {
        let file = paths::data_dir().join("queue.json");
        let jobs = load(&file);
        let next = jobs.iter().map(|job| job.id).max().unwrap_or(0) + 1;
        let queue = Arc::new(Self {
            jobs: Mutex::new(jobs),
            wake: Condvar::new(),
            cancels: Mutex::new(HashMap::new()),
            next_id: AtomicU64::new(next),
            app,
            file,
        });
        {
            let worker = queue.clone();
            std::thread::spawn(move || loop {
                if let Some(job) = worker.next() {
                    worker.run(job);
                }
            });
        }
        queue
    }

    fn persist(&self, jobs: &[Job]) {
        let mut kept: Vec<Job> = jobs.to_vec();
        let finished = kept
            .iter()
            .filter(|job| !matches!(job.status, JobStatus::Queued | JobStatus::Running))
            .count();
        if finished > HISTORY {
            let mut excess = finished - HISTORY;
            let mut index = 0;
            while index < kept.len() && excess > 0 {
                if matches!(
                    kept[index].status,
                    JobStatus::Done | JobStatus::Failed | JobStatus::Stopped
                ) {
                    kept.remove(index);
                    excess -= 1;
                } else {
                    index += 1;
                }
            }
        }
        if let Some(parent) = self.file.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(text) = serde_json::to_string_pretty(&kept) {
            let _ = std::fs::write(&self.file, text);
        }
    }

    pub fn snapshot(&self) -> Vec<Job> {
        self.jobs
            .lock()
            .map(|jobs| jobs.clone())
            .unwrap_or_default()
    }

    fn emit(&self) {
        let _ = self.app.emit("jobs", self.snapshot());
    }

    fn update<F: FnOnce(&mut Job)>(&self, id: u64, change: F) {
        if let Ok(mut jobs) = self.jobs.lock() {
            if let Some(job) = jobs.iter_mut().find(|job| job.id == id) {
                change(job);
                self.persist(&jobs);
            }
        }
        self.emit();
    }

    fn next(&self) -> Option<Job> {
        let mut jobs = self.jobs.lock().ok()?;
        loop {
            if let Some(job) = jobs.iter_mut().find(|job| job.status == JobStatus::Queued) {
                job.status = JobStatus::Running;
                job.started_at = Some(now());
                job.finished_at = None;
                job.stage = Some("prepare".to_string());
                job.detail = None;
                job.fraction = None;
                job.error = None;
                job.files.clear();
                job.segments = 0;
                job.elapsed_seconds = 0.0;
                let snapshot = job.clone();
                self.persist(&jobs);
                drop(jobs);
                self.emit();
                return Some(snapshot);
            }
            let (guard, _) = self
                .wake
                .wait_timeout(jobs, Duration::from_millis(500))
                .ok()?;
            jobs = guard;
        }
    }

    pub fn enqueue(&self, requests: Vec<NewJob>) -> Vec<Job> {
        let mut created = Vec::new();
        if let Ok(mut jobs) = self.jobs.lock() {
            for request in requests {
                let input = PathBuf::from(&request.input);
                let id = self.next_id.fetch_add(1, Ordering::Relaxed);
                created.push(Job {
                    id,
                    name: file_name(&input),
                    output: output_for(&input).display().to_string(),
                    input: input.display().to_string(),
                    model: request.model,
                    source_language: request.source_language,
                    target_language: request.target_language,
                    translator: request.translator,
                    status: JobStatus::Queued,
                    stage: None,
                    detail: None,
                    fraction: None,
                    queued_at: now(),
                    started_at: None,
                    finished_at: None,
                    elapsed_seconds: 0.0,
                    error: None,
                    files: Vec::new(),
                    segments: 0,
                });
            }
            jobs.extend(created.clone());
            self.persist(&jobs);
        }
        self.wake.notify_all();
        self.emit();
        created
    }

    pub fn action(&self, id: u64, action: &str) -> Vec<Job> {
        match action {
            "stop" => {
                let running = self
                    .snapshot()
                    .iter()
                    .any(|job| job.id == id && job.status == JobStatus::Running);
                if running {
                    if let Ok(mut cancels) = self.cancels.lock() {
                        cancels
                            .entry(id)
                            .or_insert_with(|| Arc::new(AtomicBool::new(false)))
                            .store(true, Ordering::Relaxed);
                    }
                    self.update(id, |job| {
                        job.detail = Some("Stopping after the current stage".to_string());
                    });
                }
            }
            "retry" => {
                let retryable = self.snapshot().iter().any(|job| {
                    job.id == id && matches!(job.status, JobStatus::Failed | JobStatus::Stopped)
                });
                if retryable {
                    if let Ok(mut cancels) = self.cancels.lock() {
                        if let Some(flag) = cancels.remove(&id) {
                            flag.store(false, Ordering::Relaxed);
                        }
                    }
                    self.update(id, |job| {
                        job.status = JobStatus::Queued;
                        job.stage = None;
                        job.detail = None;
                        job.fraction = None;
                        job.error = None;
                        job.started_at = None;
                        job.finished_at = None;
                        job.elapsed_seconds = 0.0;
                    });
                    self.wake.notify_all();
                }
            }
            "remove" => {
                if let Ok(mut jobs) = self.jobs.lock() {
                    let removable = jobs
                        .iter()
                        .find(|job| job.id == id)
                        .map(|job| job.status != JobStatus::Running)
                        .unwrap_or(false);
                    if removable {
                        jobs.retain(|job| job.id != id);
                        self.persist(&jobs);
                    }
                }
                self.emit();
            }
            "clear" => {
                if let Ok(mut jobs) = self.jobs.lock() {
                    jobs.retain(|job| {
                        job.status == JobStatus::Running || job.status == JobStatus::Queued
                    });
                    self.persist(&jobs);
                }
                self.emit();
            }
            _ => {}
        }
        self.snapshot()
    }

    fn cancel_flag(&self, id: u64) -> Arc<AtomicBool> {
        let mut cancels = match self.cancels.lock() {
            Ok(cancels) => cancels,
            Err(poisoned) => poisoned.into_inner(),
        };
        cancels
            .entry(id)
            .or_insert_with(|| Arc::new(AtomicBool::new(false)))
            .clone()
    }

    fn run(&self, job: Job) {
        let started = now();
        let flag = self.cancel_flag(job.id);
        let settings = Settings::default();
        let translator = job
            .translator
            .parse::<TranslatorChoice>()
            .unwrap_or_default();
        let request = Request {
            input: PathBuf::from(&job.input),
            output: PathBuf::from(&job.output),
            source_language: job.source_language.clone(),
            target_language: job.target_language.clone(),
            model: job.model.clone(),
            translator,
            ollama_host: settings.ollama_host.clone(),
            ollama_model: settings.ollama_model.clone(),
            translator_command: settings.translator_command.clone(),
            glossary: None,
            threads: 0,
            formats: vec!["srt".to_string(), "vtt".to_string()],
            translate: job.target_language.is_some(),
            resume: true,
            cancel: Some(flag.clone()),
        };

        let prepared = self.ensure_model(&job);
        let outcome = prepared.and_then(|()| {
            pipeline::run(&request, &mut |event| match event {
                Event::Stage(stage) => {
                    self.update(job.id, |current| {
                        current.stage = Some(stage.to_string());
                        current.detail = None;
                    });
                }
                Event::Detail(detail) => {
                    self.update(job.id, |current| {
                        current.detail = Some(detail);
                    });
                }
                Event::Download { received, total } => {
                    let fraction = if total == 0 {
                        0.0
                    } else {
                        received as f64 / total as f64
                    };
                    self.update(job.id, |current| {
                        current.stage = Some("model".to_string());
                        current.fraction = Some(fraction);
                        current.detail = Some(format!("{} of {}", human(received), human(total)));
                    });
                }
            })
        });

        if let Ok(mut cancels) = self.cancels.lock() {
            cancels.remove(&job.id);
        }

        let elapsed = now().saturating_sub(started) as f64;
        match outcome {
            Ok(outcome) => {
                self.update(job.id, |current| {
                    current.status = JobStatus::Done;
                    current.stage = Some("done".to_string());
                    current.detail = None;
                    current.fraction = None;
                    current.finished_at = Some(now());
                    current.elapsed_seconds = elapsed;
                    current.files = outcome.files.clone();
                    current.segments = outcome.segments;
                    current.error = None;
                });
            }
            Err(error) => {
                let stopped = error.code == "DUB-0002";
                let view = ErrorView {
                    code: error.code.to_string(),
                    title: error.title.clone(),
                    detail: error.detail.clone(),
                    hint: error.hint.clone(),
                };
                self.update(job.id, |current| {
                    current.status = if stopped {
                        JobStatus::Stopped
                    } else {
                        JobStatus::Failed
                    };
                    current.finished_at = Some(now());
                    current.elapsed_seconds = elapsed;
                    current.fraction = None;
                    current.detail = None;
                    current.error = if stopped { None } else { Some(view) };
                });
            }
        }
        self.wake.notify_all();
    }

    fn ensure_model(&self, job: &Job) -> Result<(), DubbitError> {
        let spec = models::get(&job.model)?;
        if models::is_installed(&spec) {
            return Ok(());
        }
        self.update(job.id, |current| {
            current.stage = Some("model".to_string());
            current.detail = Some(format!("Downloading {}", spec.file));
            current.fraction = Some(0.0);
        });
        let id = job.id;
        let mut last = 0_u64;
        models::install(&spec, &mut |received, total| {
            let step = received
                .saturating_mul(100)
                .checked_div(total)
                .unwrap_or(100);
            if step > last {
                last = step;
                self.update(id, |current| {
                    current.fraction = Some(step as f64 / 100.0);
                    current.detail = Some(format!("{} of {}", human(received), human(total)));
                });
            }
        })?;
        self.update(id, |current| {
            current.fraction = None;
            current.detail = None;
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn job(id: u64) -> Job {
        Job {
            id,
            input: "input.mp4".to_string(),
            name: "input.mp4".to_string(),
            output: "input.dubbit".to_string(),
            model: "whisper-tiny-q5_1".to_string(),
            source_language: None,
            target_language: None,
            translator: "auto".to_string(),
            status: JobStatus::Queued,
            stage: None,
            detail: None,
            fraction: None,
            queued_at: 0,
            started_at: None,
            finished_at: None,
            elapsed_seconds: 0.0,
            error: None,
            files: Vec::new(),
            segments: 0,
        }
    }

    #[test]
    fn output_folder_never_overwrites_an_existing_run() {
        let directory = tempfile::tempdir().unwrap();
        let input = directory.path().join("clip.mp4");
        assert_eq!(output_for(&input), directory.path().join("clip.dubbit"));
        std::fs::create_dir_all(directory.path().join("clip.dubbit")).unwrap();
        assert_eq!(output_for(&input), directory.path().join("clip (2).dubbit"));
    }

    #[test]
    fn interrupted_jobs_return_to_the_queue() {
        let directory = tempfile::tempdir().unwrap();
        let file = directory.path().join("queue.json");
        let mut running = job(1);
        running.status = JobStatus::Running;
        running.stage = Some("transcribe".to_string());
        let mut done = job(2);
        done.status = JobStatus::Done;
        let text = serde_json::to_string(&vec![running, done]).unwrap();
        std::fs::write(&file, text).unwrap();
        let restored = load(&file);
        assert_eq!(restored[0].status, JobStatus::Queued);
        assert_eq!(restored[0].stage, None);
        assert_eq!(restored[1].status, JobStatus::Done);
    }

    #[test]
    fn a_missing_queue_file_is_empty() {
        let directory = tempfile::tempdir().unwrap();
        assert!(load(&directory.path().join("queue.json")).is_empty());
    }
}

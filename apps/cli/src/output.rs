use dubbit_core::error::DubbitError;

pub fn print_error(error: &DubbitError, json: bool) {
    if json {
        let payload = serde_json::json!({ "error": error.json() });
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).unwrap_or_default()
        );
        return;
    }
    eprintln!("{}: {}", error.code, error.title);
    if let Some(detail) = &error.detail {
        eprintln!("  {detail}");
    }
    if let Some(hint) = &error.hint {
        eprintln!("  Next: {hint}");
    }
}

pub fn step(current: usize, total: usize, label: &str, json: bool) {
    if !json {
        eprintln!("({current}/{total}) {label}");
    }
}

pub fn done(line: &str, json: bool) {
    if !json {
        println!("{line}");
    }
}

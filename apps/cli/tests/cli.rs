use std::path::Path;
use std::process::{Command, Output};

fn run(home: &Path, arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_dubbit"))
        .args(arguments)
        .env("DUBBIT_HOME", home)
        .output()
        .expect("dubbit should start")
}

fn text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn lists_languages() {
    let home = tempfile::tempdir().unwrap();
    let output = run(home.path(), &["languages"]);
    assert!(output.status.success());
    let body = text(&output);
    assert!(body.contains("pt"));
    assert!(body.contains("Portuguese"));
}

#[test]
fn prints_the_config_path_inside_the_home() {
    let home = tempfile::tempdir().unwrap();
    let output = run(home.path(), &["config", "path"]);
    assert!(output.status.success());
    let body = text(&output);
    assert!(body.contains("config.json"));
    assert!(body.contains(&home.path().display().to_string()));
}

#[test]
fn stores_a_setting() {
    let home = tempfile::tempdir().unwrap();
    let set = run(home.path(), &["config", "set", "profile", "max"]);
    assert!(set.status.success(), "{}", text(&set));
    let show = run(home.path(), &["config", "show"]);
    assert!(show.status.success());
    assert!(text(&show).contains("max"));
}

#[test]
fn rejects_an_unknown_setting() {
    let home = tempfile::tempdir().unwrap();
    let output = run(home.path(), &["config", "set", "colour", "blue"]);
    assert!(!output.status.success());
    assert!(text(&output).contains("DUB-0001"));
}

#[test]
fn lists_models_as_json() {
    let home = tempfile::tempdir().unwrap();
    let output = run(home.path(), &["models", "list", "--json"]);
    assert!(output.status.success());
    let body = text(&output);
    assert!(body.contains("whisper-tiny-q5_1"));
    assert!(body.contains("installed"));
}

#[test]
fn reports_missing_input_with_a_code() {
    let home = tempfile::tempdir().unwrap();
    let output = run(home.path(), &["dub", "missing-file.mp4"]);
    assert!(!output.status.success());
    assert!(text(&output).contains("DUB-1002"));
}

#[test]
fn reports_an_unknown_model() {
    let home = tempfile::tempdir().unwrap();
    let output = run(home.path(), &["models", "install", "not-a-model"]);
    assert!(!output.status.success());
    assert!(text(&output).contains("DUB-3103"));
}

#[test]
fn reports_a_model_that_is_not_installed() {
    let home = tempfile::tempdir().unwrap();
    let output = run(home.path(), &["models", "path", "whisper-tiny-q5_1"]);
    assert!(output.status.success());
    assert!(text(&output).contains("ggml-tiny-q5_1.bin"));
}

#[test]
fn json_errors_keep_the_shape() {
    let home = tempfile::tempdir().unwrap();
    let output = run(home.path(), &["dub", "missing-file.mp4", "--json"]);
    assert!(!output.status.success());
    let body = text(&output);
    assert!(body.contains("\"error\""));
    assert!(body.contains("\"code\""));
    assert!(body.contains("\"hint\""));
}

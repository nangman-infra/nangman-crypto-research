use super::file::write_jsonl;
use super::single::write_pretty_json_file;
use super::validation::validate_output_key;
use serde_json::json;
use std::path::{Path, PathBuf};

fn unique_root(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!(
        "research-writer-{name}-{}-{nanos}",
        std::process::id()
    ))
}

#[test]
fn write_jsonl_rejects_relative_output_dir() {
    let error = write_jsonl(
        Path::new("relative-output"),
        "replay-run/schema=v1/part-000001.jsonl",
        &[json!({"ok": true})],
    )
    .unwrap_err()
    .to_string();

    assert!(error.contains("absolute path"));
}

#[test]
fn write_jsonl_rejects_ambiguous_absolute_output_dir() {
    let output_dir = std::env::temp_dir()
        .join("..")
        .join("research-ambiguous-output");
    let error = write_jsonl(
        &output_dir,
        "replay-run/schema=v1/part-000001.jsonl",
        &[json!({"ok": true})],
    )
    .unwrap_err()
    .to_string();

    assert!(error.contains("relative path components"));
    assert!(!output_dir.join("replay-run").exists());
}

#[test]
fn write_jsonl_rejects_traversal_key() {
    let output_dir =
        std::env::temp_dir().join(format!("research-io-traversal-test-{}", std::process::id()));
    let error = write_jsonl(&output_dir, "../escape.jsonl", &[json!({"ok": false})])
        .unwrap_err()
        .to_string();

    assert!(error.contains("output key"));
    assert!(!output_dir.join("escape.jsonl").exists());
}

#[test]
fn write_pretty_json_file_rejects_relative_output_file() {
    let error = write_pretty_json_file(Path::new("relative-output.json"), &json!({"ok": false}))
        .unwrap_err()
        .to_string();

    assert!(error.contains("absolute path"));
}

#[test]
fn write_pretty_json_file_rejects_ambiguous_absolute_output_file() {
    let error = write_pretty_json_file(
        Path::new("/tmp/../research-output.json"),
        &json!({"ok": false}),
    )
    .unwrap_err()
    .to_string();

    assert!(error.contains("relative path components"));
}

#[test]
fn write_pretty_json_file_writes_json_with_trailing_newline() {
    let root = unique_root("pretty-json");
    let path = root.join("nested/output.json");

    let written =
        write_pretty_json_file(&path, &json!({"ok": true})).expect("absolute output file writes");

    assert_eq!(written, path);
    let bytes = std::fs::read(&written).expect("written output is readable");
    assert!(bytes.ends_with(b"\n"));
    let value: serde_json::Value = serde_json::from_slice(&bytes).expect("output is JSON");
    assert_eq!(value, json!({"ok": true}));
    std::fs::remove_dir_all(&root).ok();
}

#[cfg(unix)]
#[test]
fn write_jsonl_rejects_symlink_output_file() {
    use std::os::unix::fs::symlink;

    let root = unique_root("jsonl-symlink");
    let outside = unique_root("jsonl-symlink-outside");
    let outside_file = outside.join("outside.jsonl");
    let link_parent = root.join("replay-run/schema=v1");
    std::fs::create_dir_all(&link_parent).unwrap();
    std::fs::create_dir_all(&outside).unwrap();
    std::fs::write(&outside_file, b"outside\n").unwrap();
    symlink(&outside_file, link_parent.join("part-000001.jsonl")).unwrap();

    let error = write_jsonl(
        &root,
        "replay-run/schema=v1/part-000001.jsonl",
        &[json!({"ok": false})],
    )
    .unwrap_err()
    .to_string();

    assert!(error.contains("output path must not be a symlink"));
    assert_eq!(std::fs::read(&outside_file).unwrap(), b"outside\n");
    std::fs::remove_dir_all(&root).ok();
    std::fs::remove_dir_all(&outside).ok();
}

#[cfg(unix)]
#[test]
fn write_pretty_json_file_rejects_symlink_output_file() {
    use std::os::unix::fs::symlink;

    let root = unique_root("pretty-symlink");
    let outside = unique_root("pretty-symlink-outside");
    let outside_file = outside.join("outside.json");
    let link = root.join("output.json");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::create_dir_all(&outside).unwrap();
    std::fs::write(&outside_file, b"{\"outside\":true}\n").unwrap();
    symlink(&outside_file, &link).unwrap();

    let error = write_pretty_json_file(&link, &json!({"ok": false}))
        .unwrap_err()
        .to_string();

    assert!(error.contains("output path must not be a symlink"));
    assert_eq!(
        std::fs::read(&outside_file).unwrap(),
        b"{\"outside\":true}\n"
    );
    std::fs::remove_dir_all(&root).ok();
    std::fs::remove_dir_all(&outside).ok();
}

#[test]
fn output_key_validation_accepts_normal_research_key() {
    validate_output_key(
        "research-run-report/schema=research_run_report_v1/dt=2026-05-29/hour=12/research_run_report_id=research_report_001/report.json",
    )
    .unwrap();
}

#[test]
fn output_key_validation_rejects_escape_shapes() {
    for key in [
        "/tmp/report.json",
        "research-run/./part-000001.jsonl",
        "research-run/../part-000001.jsonl",
        "research-run\\part-000001.jsonl",
        "research-run/\n/part-000001.jsonl",
    ] {
        let error = validate_output_key(key).unwrap_err().to_string();
        assert!(
            error.contains("output key"),
            "expected output key error for {key:?}, got {error}"
        );
    }
}

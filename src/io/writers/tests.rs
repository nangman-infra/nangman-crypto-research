use super::file::write_jsonl;
use super::validation::validate_output_key;
use serde_json::json;
use std::path::Path;

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

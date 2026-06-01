use super::*;

#[test]
fn parse_args_requires_absolute_input_path() {
    let error = parse_args(
        [
            "--input-bundle-file".to_owned(),
            "relative.jsonl".to_owned(),
        ]
        .into_iter(),
    )
    .expect_err("relative path should fail");
    assert!(error.to_string().contains("absolute path"));
}

#[test]
fn parse_args_rejects_paths_with_relative_components() {
    for (flag, value) in [
        ("--input-bundle-file", "/tmp/../bundle.jsonl"),
        ("--output-dir", "/tmp/./research-output"),
        ("--retest-horizon-status-file", "/tmp/../status.json"),
    ] {
        let error = parse_args([flag.to_owned(), value.to_owned()].into_iter())
            .expect_err("ambiguous absolute path should fail");
        let text = error.to_string();
        assert!(
            text.contains(flag),
            "expected {flag} in error for {value:?}, got {text}"
        );
        assert!(
            text.contains("relative path components"),
            "expected relative component error for {value:?}, got {text}"
        );
    }
}

#[test]
fn parse_args_requires_absolute_shadow_cycle_decision_path() {
    let error = parse_args(
        [
            "--shadow-cycle-decision-file".to_owned(),
            "relative.json".to_owned(),
        ]
        .into_iter(),
    )
    .expect_err("relative path should fail");
    assert!(error.to_string().contains("absolute path"));
}

#[test]
fn parse_args_requires_absolute_retest_horizon_status_path() {
    let error = parse_args(
        [
            "--retest-horizon-status-file".to_owned(),
            "relative.json".to_owned(),
        ]
        .into_iter(),
    )
    .expect_err("relative path should fail");
    assert!(error.to_string().contains("absolute path"));
}

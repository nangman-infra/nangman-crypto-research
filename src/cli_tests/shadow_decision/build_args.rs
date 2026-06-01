use super::*;

#[test]
fn build_shadow_cycle_decision_requires_output_target() {
    let error = parse_args(
        [
            "--build-shadow-cycle-decision".to_owned(),
            "--shadow-validation-run-file".to_owned(),
            "/tmp/shadow-runs.json".to_owned(),
        ]
        .into_iter(),
    )
    .expect_err("build mode requires an output target");

    assert!(error.to_string().contains("output"));
}

#[test]
fn build_shadow_cycle_decision_rejects_conflicting_decision_modes() {
    let error = parse_args(
        [
            "--build-shadow-cycle-decision".to_owned(),
            "--shadow-cycle-decision-file".to_owned(),
            "/tmp/shadow-cycle-decision.json".to_owned(),
            "--shadow-validation-run-file".to_owned(),
            "/tmp/shadow-runs.json".to_owned(),
            "--shadow-cycle-decision-output-file".to_owned(),
            "/tmp/shadow-cycle-output.json".to_owned(),
        ]
        .into_iter(),
    )
    .expect_err("build mode and decision validation mode are mutually exclusive");

    assert!(error.to_string().contains("separately"));
}

#[test]
fn build_shadow_cycle_decision_requires_numeric_latest_l1() {
    let error = parse_args(
        [
            "--build-shadow-cycle-decision".to_owned(),
            "--shadow-validation-run-file".to_owned(),
            "/tmp/shadow-runs.json".to_owned(),
            "--shadow-cycle-latest-l1-as-of-ms".to_owned(),
            "not-a-number".to_owned(),
            "--shadow-cycle-decision-output-file".to_owned(),
            "/tmp/shadow-cycle-output.json".to_owned(),
        ]
        .into_iter(),
    )
    .expect_err("latest L1 watermark must be numeric");

    assert!(error.to_string().contains("integer"));
}

#[test]
fn build_shadow_cycle_decision_rejects_conflicting_output_targets() {
    let error = parse_args(
        [
            "--build-shadow-cycle-decision".to_owned(),
            "--shadow-validation-run-file".to_owned(),
            "/tmp/shadow-runs.json".to_owned(),
            "--output-dir".to_owned(),
            "/tmp/shadow-cycle-output".to_owned(),
            "--output-s3-bucket".to_owned(),
            "research-bucket".to_owned(),
        ]
        .into_iter(),
    )
    .expect_err("output dir and output bucket are mutually exclusive");

    assert!(error.to_string().contains("output-dir"));
}

#[test]
fn build_shadow_cycle_decision_requires_s3_bucket_for_shadow_key() {
    let error = parse_args(
        [
            "--build-shadow-cycle-decision".to_owned(),
            "--shadow-validation-run-s3-key".to_owned(),
            "shadow-validation-run/part-000001.jsonl".to_owned(),
            "--shadow-cycle-decision-output-file".to_owned(),
            "/tmp/shadow-cycle-output.json".to_owned(),
        ]
        .into_iter(),
    )
    .expect_err("shadow validation S3 key requires bucket");

    assert!(
        error
            .to_string()
            .contains("shadow-validation-run-s3-bucket")
    );
}

#[test]
fn build_shadow_cycle_decision_requires_shadow_input_source() {
    let error = parse_args(
        [
            "--build-shadow-cycle-decision".to_owned(),
            "--shadow-cycle-decision-output-file".to_owned(),
            "/tmp/shadow-cycle-output.json".to_owned(),
        ]
        .into_iter(),
    )
    .expect_err("build mode requires shadow validation inputs");

    assert!(error.to_string().contains("shadow validation run file"));
}

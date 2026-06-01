use super::*;

#[test]
fn run_shadow_cycle_from_latest_state_requires_output_bucket() {
    let error = parse_args(["--run-shadow-cycle-from-latest-state".to_owned()].into_iter())
        .expect_err("latest shadow cycle mode requires S3 output bucket");

    assert!(error.to_string().contains("output-s3-bucket"));
}

#[test]
fn run_shadow_cycle_from_latest_state_rejects_explicit_shadow_inputs() {
    let error = parse_args(
        [
            "--run-shadow-cycle-from-latest-state".to_owned(),
            "--output-s3-bucket".to_owned(),
            "research-bucket".to_owned(),
            "--shadow-validation-run-s3-bucket".to_owned(),
            "research-bucket".to_owned(),
            "--shadow-validation-run-s3-key".to_owned(),
            "shadow-validation-run/part-000001.jsonl".to_owned(),
        ]
        .into_iter(),
    )
    .expect_err("latest shadow cycle mode discovers its own shadow inputs");

    assert!(error.to_string().contains("discovers shadow inputs"));
}

#[test]
fn run_shadow_cycle_from_latest_state_parses_with_market_l1_bucket() {
    let args = parse_args(
        [
            "--run-shadow-cycle-from-latest-state".to_owned(),
            "--output-s3-bucket".to_owned(),
            "research-bucket".to_owned(),
            "--market-l1-s3-bucket".to_owned(),
            "market-l1-bucket".to_owned(),
        ]
        .into_iter(),
    )
    .expect("latest shadow cycle args parse")
    .expect("latest shadow cycle args returned");

    assert!(args.run_shadow_cycle_from_latest_state);
    assert_eq!(args.output_s3_bucket.as_deref(), Some("research-bucket"));
    assert_eq!(
        args.market_l1_s3_bucket.as_deref(),
        Some("market-l1-bucket")
    );
}

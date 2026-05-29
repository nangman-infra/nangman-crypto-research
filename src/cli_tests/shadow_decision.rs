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

#[tokio::test]
async fn build_shadow_cycle_decision_from_shadow_runs() {
    let root = test_root("shadow-decision-build-cli");
    let shadow_file = root.join("shadow-runs.json");
    let output_file = root.join("shadow-cycle-decision.json");
    let decision_ms = 1_780_000_000_000_i64;
    let materialized_target_ms = decision_ms + DAY_MS;
    let later_decision_ms = decision_ms + 2 * 60 * 60 * 1000;
    write_json(
        &shadow_file,
        &json!([
            shadow_validation_run_json("shadow_a", "cand_a", "XAUT", decision_ms, 30),
            shadow_validation_run_json("shadow_b", "cand_b", "CHIP", later_decision_ms, 30)
        ]),
    );

    let args = parse_args(
        [
            "--build-shadow-cycle-decision".to_owned(),
            "--shadow-validation-run-file".to_owned(),
            shadow_file.display().to_string(),
            "--shadow-cycle-latest-l1-as-of-ms".to_owned(),
            materialized_target_ms.to_string(),
            "--shadow-cycle-decision-output-file".to_owned(),
            output_file.display().to_string(),
            "--now-ms".to_owned(),
            "1780100000000".to_owned(),
        ]
        .into_iter(),
    )
    .expect("build args parse")
    .expect("build args returned");

    let summary = run(args).await.expect("shadow cycle decision builds");
    assert_eq!(summary.shadow_cycle_decisions_created, 1);
    assert_eq!(summary.shadow_cycle_decisions_validated, 1);
    assert_eq!(
        summary.shadow_cycle_scheduler_action,
        Some(ShadowCycleSchedulerAction::WaitUntilTargetWindowMaterializes)
    );
    assert_eq!(summary.shadow_validation_runs_loaded, 2);
    assert_eq!(
        summary.output_files,
        vec![output_file.display().to_string()]
    );

    let decision: Value = serde_json::from_slice(
        &fs::read(&output_file).expect("shadow cycle decision file is written"),
    )
    .expect("shadow cycle decision parses");
    assert_eq!(
        decision["source_verdict"],
        json!("WAIT_FOR_TARGET_HOLDING_WINDOW")
    );
    assert_eq!(
        decision["shadow_sample_state"]["target_window_materialized_count"],
        json!(1)
    );
    assert_eq!(
        decision["shadow_sample_state"]["pending_target_window_candidate_count"],
        json!(1)
    );
    assert_eq!(decision["safety"]["order_execution_enabled"], json!(false));
}

#[tokio::test]
async fn build_shadow_cycle_decision_writes_partitioned_output_dir() {
    let root = test_root("shadow-decision-build-output-dir");
    let shadow_file = root.join("shadow-runs.json");
    let output_dir = root.join("outputs");
    let decision_ms = 1_780_000_000_000_i64;
    let materialized_target_ms = decision_ms + DAY_MS;
    write_json(
        &shadow_file,
        &json!([shadow_validation_run_json(
            "shadow_a",
            "cand_a",
            "XAUT",
            decision_ms,
            30
        )]),
    );

    let args = parse_args(
        [
            "--build-shadow-cycle-decision".to_owned(),
            "--shadow-validation-run-file".to_owned(),
            shadow_file.display().to_string(),
            "--shadow-cycle-latest-l1-as-of-ms".to_owned(),
            materialized_target_ms.to_string(),
            "--output-dir".to_owned(),
            output_dir.display().to_string(),
            "--now-ms".to_owned(),
            "1780100000000".to_owned(),
        ]
        .into_iter(),
    )
    .expect("build args parse")
    .expect("build args returned");

    let summary = run(args).await.expect("shadow cycle decision builds");
    assert_eq!(summary.shadow_cycle_decisions_created, 1);
    assert_eq!(summary.output_files.len(), 1);

    let output_file = PathBuf::from(&summary.output_files[0]);
    assert!(output_file.starts_with(&output_dir));
    assert!(
        output_file
            .display()
            .to_string()
            .contains("shadow-cycle-decision/schema=research_shadow_cycle_decision_v1")
    );
    assert!(output_file.exists());
}

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

#[tokio::test]
async fn build_shadow_cycle_decision_rejects_empty_shadow_runs() {
    let root = test_root("shadow-decision-build-empty");
    let shadow_file = root.join("shadow-runs.json");
    let output_file = root.join("shadow-cycle-decision.json");
    write_json(&shadow_file, &json!([]));

    let args = parse_args(
        [
            "--build-shadow-cycle-decision".to_owned(),
            "--shadow-validation-run-file".to_owned(),
            shadow_file.display().to_string(),
            "--shadow-cycle-decision-output-file".to_owned(),
            output_file.display().to_string(),
        ]
        .into_iter(),
    )
    .expect("build args parse")
    .expect("build args returned");

    let error = run(args)
        .await
        .expect_err("empty shadow validation input is rejected");
    assert!(
        error
            .to_string()
            .contains("at least one shadow validation run")
    );
}

#[tokio::test]
async fn shadow_cycle_decision_file_validates_without_research_inputs() {
    let root = test_root("shadow-decision-cli");
    let decision_file = root.join("shadow-cycle-decision.json");
    write_json(&decision_file, &shadow_cycle_wait_decision_json());

    let args = parse_args(
        [
            "--shadow-cycle-decision-file".to_owned(),
            decision_file.display().to_string(),
        ]
        .into_iter(),
    )
    .expect("decision args parse")
    .expect("decision args returned");
    let summary = run(args).await.expect("decision validates");

    assert_eq!(summary.shadow_cycle_decisions_validated, 1);
    assert_eq!(
        summary.shadow_cycle_scheduler_action,
        Some(ShadowCycleSchedulerAction::WaitUntilPendingShadowTargetWindowMaterializes)
    );
    assert_eq!(
        summary.shadow_cycle_run_not_before_ms,
        Some(1_779_670_979_756)
    );
    assert_eq!(summary.shadow_cycle_focused_research_manifest_file, None);
    assert_eq!(summary.processed_bundles, 0);
    assert!(summary.output_files.is_empty());
}

#[tokio::test]
async fn shadow_cycle_decision_file_rejects_order_execution_enabled() {
    let root = test_root("shadow-decision-unsafe-cli");
    let decision_file = root.join("shadow-cycle-decision.json");
    let mut decision = shadow_cycle_wait_decision_json();
    decision["safety"]["order_execution_enabled"] = json!(true);
    write_json(&decision_file, &decision);

    let args = parse_args(
        [
            "--shadow-cycle-decision-file".to_owned(),
            decision_file.display().to_string(),
        ]
        .into_iter(),
    )
    .expect("decision args parse")
    .expect("decision args returned");
    let error = run(args)
        .await
        .expect_err("unsafe shadow cycle decision is rejected");

    assert!(error.to_string().contains("paper/live/order execution"));
}

use super::*;

#[tokio::test]
async fn retest_horizon_status_file_validates_without_research_inputs() {
    let root = test_root("retest-status-cli");
    let status_file = root.join("retest-horizon-status.json");
    write_json(&status_file, &retest_horizon_wait_status_json());

    let args = parse_args(
        [
            "--retest-horizon-status-file".to_owned(),
            status_file.display().to_string(),
        ]
        .into_iter(),
    )
    .expect("status args parse")
    .expect("status args returned");
    let summary = run(args).await.expect("status validates");

    assert_eq!(summary.retest_horizon_statuses_validated, 1);
    assert_eq!(
        summary.retest_cycle_scheduler_action,
        Some("WAIT_UNTIL_MARKET_L1_HORIZON_MATERIALIZES".to_owned())
    );
    assert_eq!(
        summary.retest_cycle_run_not_before_ms,
        Some(1_779_719_361_452)
    );
    assert_eq!(summary.processed_bundles, 0);
    assert!(summary.output_files.is_empty());
}

#[tokio::test]
async fn retest_horizon_status_file_rejects_live_enabled() {
    let root = test_root("retest-status-unsafe-cli");
    let status_file = root.join("retest-horizon-status.json");
    let mut status = retest_horizon_wait_status_json();
    status["stage_state"]["live_enabled"] = json!(true);
    write_json(&status_file, &status);

    let args = parse_args(
        [
            "--retest-horizon-status-file".to_owned(),
            status_file.display().to_string(),
        ]
        .into_iter(),
    )
    .expect("status args parse")
    .expect("status args returned");
    let error = run(args)
        .await
        .expect_err("unsafe retest status is rejected");

    assert!(error.to_string().contains("live trading"));
}

#[tokio::test]
async fn build_retest_horizon_status_from_plan_file() {
    let root = test_root("retest-status-build-cli");
    let plan_file = root.join("retest-horizon-plan.json");
    let output_file = root.join("retest-horizon-status.json");
    write_json(&plan_file, &retest_horizon_plan_json());

    let args = parse_args(
        [
            "--build-retest-horizon-status".to_owned(),
            "--retest-horizon-plan-file".to_owned(),
            plan_file.display().to_string(),
            "--retest-horizon-status-output-file".to_owned(),
            output_file.display().to_string(),
            "--now-ms".to_owned(),
            "1779714000000".to_owned(),
        ]
        .into_iter(),
    )
    .expect("status build args parse")
    .expect("status build args returned");
    let summary = run(args).await.expect("status builds");

    assert_eq!(summary.retest_horizon_statuses_validated, 1);
    assert_eq!(
        summary.retest_cycle_scheduler_action,
        Some("RUN_FOCUSED_RETEST_RESEARCH".to_owned())
    );
    assert_eq!(summary.retest_cycle_run_not_before_ms, None);
    assert_eq!(
        summary.output_files,
        vec![output_file.display().to_string()]
    );

    let status: Value =
        serde_json::from_slice(&fs::read(&output_file).expect("status")).expect("status json");
    assert_eq!(
        status["schema_version"],
        json!("research_horizon_status_checkpoint_v1")
    );
    assert_eq!(status["safety"]["checkpoint_s3_write"], json!(false));
    assert_eq!(status["selected_symbols"], json!(["AAVE"]));
    assert_eq!(
        status["by_symbol"][0]["candidates"][1]["horizons"][0]["next_action"],
        json!("wait_for_market_l1_horizon")
    );
}

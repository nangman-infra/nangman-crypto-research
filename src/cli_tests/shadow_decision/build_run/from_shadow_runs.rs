use super::super::*;

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

use super::*;

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

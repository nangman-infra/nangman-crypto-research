use super::*;

#[tokio::test]
async fn retest_refresh_cycle_waits_without_writing_focused_manifest() {
    let root = test_root("retest-refresh-wait");
    let (manifest, report_file) = write_refresh_cycle_inputs(&root).await;
    let output = root.join("cycle-out");

    let args = parse_args(
        [
            "--run-retest-refresh-cycle".to_owned(),
            "--input-manifest-file".to_owned(),
            manifest.display().to_string(),
            "--research-report-file".to_owned(),
            report_file.display().to_string(),
            "--retest-horizon-latest-l1-as-of-ms".to_owned(),
            "1000".to_owned(),
            "--output-dir".to_owned(),
            output.display().to_string(),
            "--now-ms".to_owned(),
            "2000".to_owned(),
        ]
        .into_iter(),
    )
    .expect("refresh args parse")
    .expect("refresh args returned");
    let summary = run(args).await.expect("refresh cycle waits");

    assert_eq!(summary.retest_horizon_plans_created, 1);
    assert_eq!(summary.retest_horizon_statuses_validated, 1);
    assert_eq!(
        summary.retest_cycle_scheduler_action,
        Some("WAIT_UNTIL_MARKET_L1_HORIZON_MATERIALIZES".to_owned())
    );
    assert_eq!(summary.focused_retest_manifests_created, 0);
    assert!(output.join("retest-horizon-plan.json").exists());
    assert!(output.join("retest-horizon-status.json").exists());
    assert!(!output.join("research-input-manifest.json").exists());
}

#[tokio::test]
async fn retest_refresh_cycle_writes_focused_manifest_for_accumulation_ready_horizon() {
    let root = test_root("retest-refresh-run");
    let (manifest, report_file) = write_refresh_cycle_inputs(&root).await;
    let output = root.join("cycle-out");

    let args = parse_args(
        [
            "--run-retest-refresh-cycle".to_owned(),
            "--input-manifest-file".to_owned(),
            manifest.display().to_string(),
            "--research-report-file".to_owned(),
            report_file.display().to_string(),
            "--retest-horizon-latest-l1-as-of-ms".to_owned(),
            "7201300".to_owned(),
            "--output-dir".to_owned(),
            output.display().to_string(),
            "--research-packet-id".to_owned(),
            "refresh_cycle_focus".to_owned(),
            "--now-ms".to_owned(),
            "7400000".to_owned(),
        ]
        .into_iter(),
    )
    .expect("refresh args parse")
    .expect("refresh args returned");
    let summary = run(args).await.expect("refresh cycle writes focus");

    assert_eq!(summary.retest_horizon_plans_created, 1);
    assert_eq!(summary.retest_horizon_statuses_validated, 1);
    assert_eq!(
        summary.retest_cycle_scheduler_action,
        Some("RUN_FOCUSED_RETEST_RESEARCH".to_owned())
    );
    assert_eq!(summary.focused_retest_manifests_created, 1);
    assert_eq!(summary.focused_retest_candidate_bundle_refs, 1);
    assert!(output.join("retest-horizon-plan.json").exists());
    assert!(output.join("retest-horizon-status.json").exists());
    assert!(output.join("research-input-manifest.json").exists());
    assert!(output.join("research-input-manifest.summary.json").exists());
}

async fn write_refresh_cycle_inputs(root: &Path) -> (PathBuf, PathBuf) {
    let bundle =
        root.join("candidate-evidence-bundle/priority=p0/candidate_id=cand_001/part-000001.json");
    let delta = root.join("delta.json");
    let regime = root.join("regime.json");
    let manifest = root.join("manifest.json");
    let research_output = root.join("research-out");

    write_json(&bundle, &bundle_json_with_gate_inputs(1, 1_300));
    write_json(
        &delta,
        &json!([market_delta_json("delta_001", 1_300, 3_601_300, 0.021)]),
    );
    write_json(
        &regime,
        &json!([market_regime_json("regime_001", 1_300, 3_601_300)]),
    );
    write_json(
        &manifest,
        &json!({
            "schema_version": "research_input_manifest_v1",
            "research_packet_id": "manifest_packet",
            "run_scope": "manifest_batch",
            "candidate_bundle_refs": [{ "uri": bundle.display().to_string() }],
            "market_feature_delta_refs": [{ "uri": delta.display().to_string() }],
            "market_regime_context_refs": [{ "uri": regime.display().to_string() }],
            "runtime_budget_policy": {
                "max_candidate_bundle_count": 10,
                "max_market_artifact_ref_count": 10,
                "max_historical_replay_run_ref_count": 10,
                "max_replay_run_count": 20
            }
        }),
    );
    let research_summary = run(Args {
        input_manifest_file: Some(manifest.clone()),
        output_dir: Some(research_output),
        now_ms: Some(7_300_000),
        ..default_args()
    })
    .await
    .expect("research report builds");
    let report_file = output_file_containing(&research_summary, "research-run-report");
    (manifest, report_file)
}

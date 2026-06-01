use super::*;

#[tokio::test]
async fn build_retest_horizon_plan_from_manifest_and_report() {
    let root = test_root("retest-plan-build-cli");
    let bundle = root.join("bundle.json");
    let delta = root.join("delta.json");
    let regime = root.join("regime.json");
    let manifest = root.join("manifest.json");
    let research_output = root.join("research-out");
    let plan_output = root.join("retest-horizon-plan.json");

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

    let args = parse_args(
        [
            "--build-retest-horizon-plan".to_owned(),
            "--input-manifest-file".to_owned(),
            manifest.display().to_string(),
            "--research-report-file".to_owned(),
            report_file.display().to_string(),
            "--retest-horizon-plan-output-file".to_owned(),
            plan_output.display().to_string(),
            "--retest-horizon-latest-l1-as-of-ms".to_owned(),
            "7201300".to_owned(),
            "--now-ms".to_owned(),
            "7400000".to_owned(),
        ]
        .into_iter(),
    )
    .expect("plan build args parse")
    .expect("plan build args returned");
    let summary = run(args).await.expect("plan builds");

    assert_eq!(summary.retest_horizon_plans_created, 1);
    assert_eq!(
        summary.output_files,
        vec![plan_output.display().to_string()]
    );
    let plan: Value =
        serde_json::from_slice(&fs::read(&plan_output).expect("plan")).expect("plan json");
    assert_eq!(
        plan["schema_version"],
        json!("research_retest_horizon_plan_v1")
    );
    assert_eq!(plan["generated_at_ms"], json!(7_400_000));
    assert_eq!(plan["latest_l1_as_of_ms"], json!(7_201_300));
    assert_eq!(plan["summary"]["candidate_count"], json!(1));
    assert_eq!(plan["summary"]["horizon_count"], json!(1));
    assert_eq!(
        plan["horizon_rows"][0]["next_action"],
        json!("accumulate_completed_native_replay_samples")
    );
}

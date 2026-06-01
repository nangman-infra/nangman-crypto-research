use super::*;

#[tokio::test]
async fn manifest_batch_input_processes_multiple_candidate_refs() {
    let root = test_root("manifest-batch");
    let bundle_a = root.join("bundle-a.json");
    let bundle_b = root.join("bundle-b.json");
    let delta = root.join("delta.json");
    let regime = root.join("regime.json");
    let manifest = root.join("manifest.json");
    let output = root.join("out");

    write_json(&bundle_a, &bundle_json_with_gate_inputs(1, 1_300));
    write_json(&bundle_b, &bundle_json_with_gate_inputs(2, 3_601_300));
    write_json(
        &delta,
        &json!([
            market_delta_json("delta_001", 1_300, 3_601_300, 0.021),
            market_delta_json("delta_002", 3_601_300, 7_201_300, -0.004)
        ]),
    );
    write_json(
        &regime,
        &json!([
            market_regime_json("regime_001", 1_300, 3_601_300),
            market_regime_json("regime_002", 3_601_300, 7_201_300)
        ]),
    );
    write_json(
        &manifest,
        &json!({
            "schema_version": "research_input_manifest_v1",
            "research_packet_id": "manifest_packet",
            "run_scope": "manifest_batch",
            "candidate_bundle_refs": [
                { "uri": bundle_a.display().to_string() },
                { "uri": bundle_b.display().to_string() }
            ],
            "market_feature_delta_refs": [
                { "uri": delta.display().to_string() }
            ],
            "market_regime_context_refs": [
                { "uri": regime.display().to_string() }
            ],
            "runtime_budget_policy": {
                "max_candidate_bundle_count": 10,
                "max_market_artifact_ref_count": 10,
                "max_historical_replay_run_ref_count": 10,
                "max_replay_run_count": 20
            }
        }),
    );

    let summary = run(Args {
        input_manifest_file: Some(manifest),
        output_dir: Some(output),
        research_packet_id: "cli_packet".to_owned(),
        run_scope: "cli_scope".to_owned(),
        now_ms: Some(7_300_000),
        ..default_args()
    })
    .await
    .expect("manifest batch run succeeds");

    assert_eq!(summary.processed_bundles, 2);
    assert_eq!(summary.replay_runs_created, 2);
    assert!(
        summary
            .output_files
            .iter()
            .any(|path| path.contains("replay-run-index"))
    );
    let report: Value =
        serde_json::from_str(&fs::read_to_string(&summary.output_files[0]).expect("report exists"))
            .expect("report json parses");
    assert_eq!(report["research_packet_id"], json!("manifest_packet"));
    assert_eq!(report["run_scope"], json!("manifest_batch"));
    assert_eq!(
        report["source_candidate_ids"],
        json!(["cand_001", "cand_002"])
    );
}

#[tokio::test]
async fn manifest_runtime_budget_blocks_oversized_batch() {
    let root = test_root("manifest-budget");
    let bundle_a = root.join("bundle-a.json");
    let bundle_b = root.join("bundle-b.json");
    let manifest = root.join("manifest.json");

    write_json(&bundle_a, &bundle_json_with_gate_inputs(1, 1_300));
    write_json(&bundle_b, &bundle_json_with_gate_inputs(2, 3_601_300));
    write_json(
        &manifest,
        &json!({
            "schema_version": "research_input_manifest_v1",
            "candidate_bundle_refs": [
                { "uri": bundle_a.display().to_string() },
                { "uri": bundle_b.display().to_string() }
            ],
            "runtime_budget_policy": {
                "max_candidate_bundle_count": 1,
                "max_market_artifact_ref_count": 10,
                "max_historical_replay_run_ref_count": 10,
                "max_replay_run_count": 20
            }
        }),
    );

    let error = run(Args {
        input_manifest_file: Some(manifest),
        output_dir: Some(root.join("out")),
        research_packet_id: "packet_test".to_owned(),
        run_scope: "test".to_owned(),
        now_ms: Some(7_300_000),
        ..default_args()
    })
    .await
    .expect_err("oversized manifest is rejected");

    assert!(error.to_string().contains("runtime budget exceeded"));
}

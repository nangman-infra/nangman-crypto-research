use super::*;

#[tokio::test]
async fn negative_market_replay_prunes_candidate() {
    let root = test_root("negative-market");
    let input = root.join("bundles.jsonl");
    let delta = root.join("delta.json");
    let output = root.join("out");
    write_json(&input, &bundle_json());
    write_json(
        &delta,
        &json!([{
            "schema_version": "market_feature_delta_v1",
            "feature_delta_id": "delta_001",
            "l1_run_id": "l1_001",
            "metric_name": "price",
            "venue": "binance",
            "symbol_native": "SUIUSDT",
            "symbol_canonical": "SUI",
            "market_type": "spot",
            "value_now": 1.0,
            "price_change_same_window": -0.3,
            "window_start_ms": 1_300,
            "window_end_ms": 3_601_300,
            "known_as_of_ms": 3_601_400,
            "quality_status": "available",
            "missing_reasons": []
        }]),
    );

    let summary = run(Args {
        shadow_cycle_decision_file: None,
        input_manifest_file: None,
        input_manifest_s3_bucket: None,
        input_manifest_s3_key: None,
        input_bundle_file: Some(input),
        input_bundle_s3_bucket: None,
        input_bundle_s3_key: None,
        market_feature_delta_file: Some(delta),
        market_regime_context_file: None,
        market_l1_s3_bucket: None,
        market_feature_delta_s3_keys: Vec::new(),
        market_regime_context_s3_keys: Vec::new(),
        historical_replay_run_files: Vec::new(),
        historical_replay_run_index_files: Vec::new(),
        oss_adapter_run_files: Vec::new(),
        shadow_validation_run_files: Vec::new(),
        oss_adapter_run_s3_bucket: None,
        oss_adapter_run_s3_keys: Vec::new(),
        shadow_validation_run_s3_bucket: None,
        shadow_validation_run_s3_keys: Vec::new(),
        historical_replay_run_s3_bucket: None,
        historical_replay_run_s3_keys: Vec::new(),
        historical_replay_run_index_s3_bucket: None,
        historical_replay_run_index_s3_keys: Vec::new(),
        output_dir: Some(output),
        output_s3_bucket: None,
        output_s3_prefix: None,
        research_packet_id: "packet_test".to_owned(),
        run_scope: "test".to_owned(),
        now_ms: Some(1_800_000),
        ..default_args()
    })
    .await
    .expect("run succeeds");

    let report_text = fs::read_to_string(&summary.output_files[0]).expect("report exists");
    assert!(report_text.contains("PRUNE_BIAS"));
    assert!(report_text.contains("native_replay_net_edge_non_positive"));
}

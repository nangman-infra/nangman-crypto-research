use super::*;

#[tokio::test]
async fn valid_bundle_without_market_data_becomes_retest_report() {
    let root = test_root("missing-market");
    let input = root.join("bundles.jsonl");
    let output = root.join("out");
    write_json(&input, &bundle_json());

    let summary = run(Args {
        shadow_cycle_decision_file: None,
        input_manifest_file: None,
        input_manifest_s3_bucket: None,
        input_manifest_s3_key: None,
        input_bundle_file: Some(input),
        input_bundle_s3_bucket: None,
        input_bundle_s3_key: None,
        market_feature_delta_file: None,
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
        output_dir: Some(output.clone()),
        output_s3_bucket: None,
        output_s3_prefix: None,
        research_packet_id: "packet_test".to_owned(),
        run_scope: "test".to_owned(),
        now_ms: Some(1_800_000),
        ..default_args()
    })
    .await
    .expect("run succeeds");

    assert_eq!(summary.processed_bundles, 1);
    assert_eq!(summary.replay_runs_created, 1);
    let report_text = fs::read_to_string(&summary.output_files[0]).expect("report exists");
    assert!(report_text.contains("RETEST_BIAS"));
    assert!(report_text.contains("missing_native_replay_market_data"));
    assert!(!report_text.contains("EXECUTION_APPROVED"));
    assert!(!report_text.contains("LIVE_READY"));
}

#[tokio::test]
async fn horizon_over_72h_is_invalid_input() {
    let root = test_root("holding-horizon");
    let input = root.join("bundles.jsonl");
    let output = root.join("out");
    let mut bundle = bundle_json();
    bundle["allowed_horizons"] = json!(["7d"]);
    write_json(&input, &bundle);

    let summary = run(Args {
        shadow_cycle_decision_file: None,
        input_manifest_file: None,
        input_manifest_s3_bucket: None,
        input_manifest_s3_key: None,
        input_bundle_file: Some(input),
        input_bundle_s3_bucket: None,
        input_bundle_s3_key: None,
        market_feature_delta_file: None,
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
    .expect("run succeeds with invalid replay record");

    let report_text = fs::read_to_string(&summary.output_files[0]).expect("report exists");
    assert!(report_text.contains("holding_horizon_contract_violation"));
    assert!(report_text.contains("invalid_input"));
    assert_eq!(summary.shadow_validation_runs_created, 0);
}

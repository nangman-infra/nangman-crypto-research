use super::*;

#[tokio::test]
async fn oss_adapter_prune_bias_blocks_candidate_even_when_native_retest() {
    let root = test_root("oss-prune");
    let input = root.join("bundles.jsonl");
    let oss = root.join("oss.json");
    let output = root.join("out");
    write_json(&input, &bundle_json());
    write_json(&oss, &oss_adapter_run_json("cand_001:v1", "PRUNE_BIAS"));

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
        oss_adapter_run_files: vec![oss],
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

    let report: Value =
        serde_json::from_str(&fs::read_to_string(&summary.output_files[0]).expect("report exists"))
            .expect("report json parses");
    assert_eq!(summary.oss_adapter_runs_loaded, 1);
    assert_eq!(report["summary_findings"][0]["bias"], json!("PRUNE_BIAS"));
    assert_eq!(report["oss_adapter_reject_count"], json!(1));
    assert!(
        report["summary_findings"][0]["reason_codes"]
            .as_array()
            .expect("reason codes")
            .contains(&json!("oss_adapter_prune_bias"))
    );
}

#[tokio::test]
async fn oss_adapter_holding_violation_fails_before_report() {
    let root = test_root("oss-holding-violation");
    let input = root.join("bundles.jsonl");
    let oss = root.join("oss.json");
    let output = root.join("out");
    let mut adapter = oss_adapter_run_json("cand_001:v1", "RETEST_BIAS");
    adapter["holding_horizon_check_result"] = json!("holding_horizon_contract_violation");
    write_json(&input, &bundle_json());
    write_json(&oss, &adapter);

    let error = run(Args {
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
        oss_adapter_run_files: vec![oss],
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
    .expect_err("holding horizon violation must fail");

    assert!(error.to_string().contains("holding horizon check"));
}

use super::*;

#[tokio::test]
async fn invalid_regime_context_does_not_prune_positive_replay() {
    let root = test_root("invalid-regime-context");
    let input = root.join("bundles.jsonl");
    let delta = root.join("delta.json");
    let regime = root.join("regime.json");
    let output = root.join("out");
    let mut invalid_regime = market_regime_json("regime_invalid", 1_300, 3_601_300);
    invalid_regime["btc_return_same_window"] = json!(0.9);
    invalid_regime["volatility_regime"] = json!("invalid_should_be_ignored");
    invalid_regime["quality_status"] = json!("invalid");
    write_json(&input, &bundle_json());
    write_json(
        &delta,
        &json!([market_delta_json("delta_001", 1_300, 3_601_300, 0.5)]),
    );
    write_json(&regime, &json!([invalid_regime]));

    let summary = run(Args {
        shadow_cycle_decision_file: None,
        input_manifest_file: None,
        input_manifest_s3_bucket: None,
        input_manifest_s3_key: None,
        input_bundle_file: Some(input),
        input_bundle_s3_bucket: None,
        input_bundle_s3_key: None,
        market_feature_delta_file: Some(delta),
        market_regime_context_file: Some(regime),
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

    let report: Value =
        serde_json::from_str(&fs::read_to_string(&summary.output_files[0]).expect("report exists"))
            .expect("report json parses");
    assert_eq!(report["summary_findings"][0]["bias"], json!("RETEST_BIAS"));
    let replay_file = output_file_containing(&summary, "/replay-run/");
    let replay_text = fs::read_to_string(&replay_file).expect("replay output exists");
    let replay: Value = serde_json::from_str(
        replay_text
            .lines()
            .next()
            .expect("replay output has one line"),
    )
    .expect("replay line parses");
    assert!(
        replay["result_summary"]["reason_codes"]
            .as_array()
            .expect("reason codes")
            .contains(&json!("market_regime_context_missing"))
    );
}

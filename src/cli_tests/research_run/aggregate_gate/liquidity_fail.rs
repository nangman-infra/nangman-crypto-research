use super::*;

#[tokio::test]
async fn aggregate_gate_blocks_zero_volume_liquidity_filter() {
    let root = test_root("aggregate-zero-liquidity");
    let input = root.join("bundles.json");
    let delta = root.join("delta.json");
    let regime = root.join("regime.json");
    let output = root.join("out");
    let mut bundle = bundle_json_with_gate_inputs(0, 1_300);
    bundle["validation_requirements"]["include_liquidity_filter"] = json!(true);

    write_json(&input, &Value::Array(vec![bundle]));
    write_json(
        &delta,
        &Value::Array(vec![
            market_delta_json("delta_price_000", 1_300, 3_601_300, 0.5),
            market_liquidity_delta_json_with_value("delta_liquidity_000", 1_300, 3_601_300, 0.0),
        ]),
    );
    write_json(
        &regime,
        &Value::Array(vec![market_regime_json("regime_000", 1_300, 3_601_300)]),
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
        now_ms: Some(120_000_000),
        ..default_args()
    })
    .await
    .expect("run succeeds");

    let report: Value =
        serde_json::from_str(&fs::read_to_string(&summary.output_files[0]).expect("report exists"))
            .expect("report json parses");
    let aggregate = &report["partition_aggregates"][0];
    let gate_reasons = aggregate["gate_reason_codes"]
        .as_array()
        .expect("gate reasons are present");
    assert_eq!(aggregate["liquidity_filter_materialized_count"], json!(1));
    assert_eq!(aggregate["liquidity_filter_passed_count"], json!(0));
    assert_eq!(aggregate["liquidity_filter_failed_count"], json!(1));
    assert!(gate_reasons.contains(&json!("liquidity_filter_failed")));
    assert!(!gate_reasons.contains(&json!("liquidity_filter_not_materialized")));

    let replay_output_file = output_file_containing(&summary, "/replay-run/");
    let replay_output_text = fs::read_to_string(&replay_output_file).expect("replay output exists");
    let replay: Value = serde_json::from_str(
        replay_output_text
            .lines()
            .next()
            .expect("replay output has one line"),
    )
    .expect("replay line parses");
    let liquidity_summary = &replay["result_summary"]["liquidity_filter_summary"];
    assert_eq!(liquidity_summary["status"], json!("failed"));
    assert_eq!(
        liquidity_summary["reason_codes"],
        json!(["liquidity_filter_no_positive_volume_observed"])
    );
    assert_eq!(liquidity_summary["observed_metric_count"], json!(1));
    assert_eq!(liquidity_summary["positive_volume_metric_count"], json!(0));
}

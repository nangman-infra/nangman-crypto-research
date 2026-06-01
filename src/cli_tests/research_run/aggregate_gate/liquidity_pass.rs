use super::*;

#[tokio::test]
async fn aggregate_gate_accepts_materialized_liquidity_filter() {
    let root = test_root("aggregate-liquidity");
    let input = root.join("bundles.json");
    let delta = root.join("delta.json");
    let regime = root.join("regime.json");
    let output = root.join("out");
    let mut bundles = Vec::new();
    let mut deltas = Vec::new();
    let mut regimes = Vec::new();

    for index in 0..31 {
        let decision_ms = 1_300 + (index as i64 * 3_600_000);
        let window_end_ms = decision_ms + 3_600_000;
        let mut bundle = bundle_json_with_gate_inputs(index, decision_ms);
        bundle["validation_requirements"]["include_liquidity_filter"] = json!(true);
        bundles.push(bundle);
        deltas.push(market_delta_json(
            &format!("delta_price_{index:03}"),
            decision_ms,
            window_end_ms,
            0.5,
        ));
        deltas.push(market_liquidity_delta_json(
            &format!("delta_liquidity_{index:03}"),
            decision_ms,
            window_end_ms,
        ));
        regimes.push(market_regime_json(
            &format!("regime_{index:03}"),
            decision_ms,
            window_end_ms,
        ));
    }

    write_json(&input, &Value::Array(bundles));
    write_json(&delta, &Value::Array(deltas));
    write_json(&regime, &Value::Array(regimes));

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

    assert_eq!(summary.shadow_validation_runs_created, 31);

    let report: Value =
        serde_json::from_str(&fs::read_to_string(&summary.output_files[0]).expect("report exists"))
            .expect("report json parses");
    let aggregate = &report["partition_aggregates"][0];
    assert_eq!(aggregate["completed_count"], json!(31));
    assert_eq!(aggregate["liquidity_filter_materialized_count"], json!(31));
    assert_eq!(aggregate["liquidity_filter_passed_count"], json!(31));
    assert_eq!(aggregate["liquidity_filter_failed_count"], json!(0));
    assert_eq!(aggregate["gate_bias"], json!("PROMOTE_TO_SHADOW_BIAS"));
    assert_eq!(
        aggregate["gate_reason_codes"],
        json!(["deterministic_shadow_gate_passed"])
    );

    let replay_output_file = output_file_containing(&summary, "/replay-run/");
    let replay_output_text = fs::read_to_string(&replay_output_file).expect("replay output exists");
    for line in replay_output_text.lines() {
        let replay: Value = serde_json::from_str(line).expect("replay line parses");
        let liquidity_summary = &replay["result_summary"]["liquidity_filter_summary"];
        assert_eq!(liquidity_summary["status"], json!("passed"));
        assert_eq!(
            liquidity_summary["reason_codes"],
            json!(["liquidity_filter_positive_volume_observed"])
        );
        assert_eq!(liquidity_summary["observed_metric_count"], json!(1));
        assert_eq!(liquidity_summary["positive_volume_metric_count"], json!(1));
    }
}

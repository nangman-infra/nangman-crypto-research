use super::super::*;

#[tokio::test]
async fn portfolio_rejects_critical_event_symbol_and_emits_reduce_only() {
    let root = test_root("portfolio-critical");
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
        if index == 0 {
            bundle["event_types"] = json!(["exchange_delisting"]);
        }
        bundles.push(bundle);
        deltas.push(market_delta_json(
            &format!("delta_{index:03}"),
            decision_ms,
            window_end_ms,
            0.5,
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

    assert!(summary.portfolio_risk_reject_events_created > 0);
    assert_eq!(summary.portfolio_reduce_only_signals_created, 1);
    let report: Value =
        serde_json::from_str(&fs::read_to_string(&summary.output_files[0]).expect("report exists"))
            .expect("report json parses");
    assert_eq!(
        report["portfolio_allocation_snapshot"]["max_total_notional_pct"],
        json!(0.0)
    );
    assert!(
        report["portfolio_allocation_snapshot"]["reason_codes"]
            .as_array()
            .expect("reason codes")
            .contains(&json!("exchange_delisting"))
    );
    let reduce_only_file = output_file_containing(&summary, "/portfolio-reduce-only-signal/");
    let reduce_only_text = fs::read_to_string(&reduce_only_file).expect("reduce-only exists");
    assert!(reduce_only_text.contains("exchange_delisting"));
}

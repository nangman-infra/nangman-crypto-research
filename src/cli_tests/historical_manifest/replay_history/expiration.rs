use super::*;

#[tokio::test]
async fn expired_historical_replay_runs_are_excluded_from_promotion_gate() {
    let root = test_root("expired-history");
    let history_input = root.join("history-bundles.json");
    let history_delta = root.join("history-delta.json");
    let history_regime = root.join("history-regime.json");
    let history_output = root.join("history-out");
    let mut history_bundles = Vec::new();
    let mut history_deltas = Vec::new();
    let mut history_regimes = Vec::new();

    for index in 0..30 {
        let decision_ms = 1_300 + (index as i64 * 3_600_000);
        let window_end_ms = decision_ms + 3_600_000;
        history_bundles.push(bundle_json_with_gate_inputs(index, decision_ms));
        history_deltas.push(market_delta_json(
            &format!("history_delta_{index:03}"),
            decision_ms,
            window_end_ms,
            0.5,
        ));
        history_regimes.push(market_regime_json(
            &format!("history_regime_{index:03}"),
            decision_ms,
            window_end_ms,
        ));
    }

    write_json(&history_input, &Value::Array(history_bundles));
    write_json(&history_delta, &Value::Array(history_deltas));
    write_json(&history_regime, &Value::Array(history_regimes));

    let history_summary = run(Args {
        input_bundle_file: Some(history_input),
        market_feature_delta_file: Some(history_delta),
        market_regime_context_file: Some(history_regime),
        output_dir: Some(history_output),
        research_packet_id: "packet_test".to_owned(),
        run_scope: "test".to_owned(),
        now_ms: Some(120_000_000),
        ..default_args()
    })
    .await
    .expect("history run succeeds");
    let history_replay_file = output_file_containing(&history_summary, "/replay-run/");

    let current_input = root.join("current-bundles.json");
    let current_delta = root.join("current-delta.json");
    let current_regime = root.join("current-regime.json");
    let current_output = root.join("current-out");
    let current_decision_ms = (100 * DAY_MS) + 1_300;
    let current_window_end_ms = current_decision_ms + 3_600_000;
    write_json(
        &current_input,
        &Value::Array(vec![bundle_json_with_gate_inputs(999, current_decision_ms)]),
    );
    write_json(
        &current_delta,
        &Value::Array(vec![market_delta_json(
            "current_delta_999",
            current_decision_ms,
            current_window_end_ms,
            0.5,
        )]),
    );
    write_json(
        &current_regime,
        &Value::Array(vec![market_regime_json(
            "current_regime_999",
            current_decision_ms,
            current_window_end_ms,
        )]),
    );

    let summary = run(Args {
        input_bundle_file: Some(current_input),
        market_feature_delta_file: Some(current_delta),
        market_regime_context_file: Some(current_regime),
        historical_replay_run_files: vec![history_replay_file],
        output_dir: Some(current_output),
        research_packet_id: "packet_test".to_owned(),
        run_scope: "test".to_owned(),
        now_ms: Some(current_window_end_ms + 100_000),
        ..default_args()
    })
    .await
    .expect("current run succeeds");

    assert_eq!(summary.replay_runs_created, 1);
    assert_eq!(summary.historical_replay_runs_loaded, 30);
    assert_eq!(summary.shadow_validation_runs_created, 0);
    let report: Value =
        serde_json::from_str(&fs::read_to_string(&summary.output_files[0]).expect("report exists"))
            .expect("report json parses");
    let aggregate = &report["partition_aggregates"][0];
    assert_eq!(aggregate["gate_bias"], json!("RETEST_BIAS"));
    assert_eq!(aggregate["replay_run_count"], json!(31));
    assert_eq!(aggregate["active_replay_run_count"], json!(1));
    assert_eq!(aggregate["expired_replay_run_count"], json!(30));
    assert_eq!(aggregate["completed_count"], json!(1));
    assert_eq!(aggregate["expired_completed_count"], json!(30));
    assert_eq!(aggregate["effective_completed_sample_weight"], json!(1.0));
    assert_eq!(aggregate["inferred_unseen_window_count"], json!(0));
    let gate_reasons = aggregate["gate_reason_codes"]
        .as_array()
        .expect("gate reasons are an array");
    assert!(gate_reasons.contains(&json!("promotion_sample_count_below_minimum")));
    assert!(gate_reasons.contains(&json!("promotion_effective_sample_weight_below_minimum")));
    assert_eq!(report["summary_findings"][0]["bias"], json!("RETEST_BIAS"));
    assert_eq!(report["shadow_validation_runs"], json!([]));
}

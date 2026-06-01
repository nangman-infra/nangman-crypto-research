use super::*;

#[tokio::test]
async fn historical_replay_runs_are_filtered_to_current_aggregate_keys() {
    let root = test_root("historical-filter");
    let history_input = root.join("history-bundles.json");
    let history_delta = root.join("history-delta.json");
    let history_regime = root.join("history-regime.json");
    let history_output = root.join("history-out");

    let sui_decision_ms = 1_300;
    let btc_decision_ms = 3_601_300;
    let mut btc_bundle = bundle_json_with_gate_inputs(2, btc_decision_ms);
    retarget_bundle_symbol(&mut btc_bundle, "BTC");

    let mut btc_delta = market_delta_json(
        "history_delta_btc",
        btc_decision_ms,
        btc_decision_ms + 3_600_000,
        0.5,
    );
    retarget_market_delta_symbol(&mut btc_delta, "BTC");

    write_json(
        &history_input,
        &Value::Array(vec![
            bundle_json_with_gate_inputs(1, sui_decision_ms),
            btc_bundle,
        ]),
    );
    write_json(
        &history_delta,
        &Value::Array(vec![
            market_delta_json(
                "history_delta_sui",
                sui_decision_ms,
                sui_decision_ms + 3_600_000,
                0.5,
            ),
            btc_delta,
        ]),
    );
    write_json(
        &history_regime,
        &Value::Array(vec![
            market_regime_json(
                "history_regime_sui",
                sui_decision_ms,
                sui_decision_ms + 3_600_000,
            ),
            market_regime_json(
                "history_regime_btc",
                btc_decision_ms,
                btc_decision_ms + 3_600_000,
            ),
        ]),
    );

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
    assert_eq!(history_summary.replay_runs_created, 2);
    let history_index_file = output_file_containing(&history_summary, "/replay-run-index/");

    let current_input = root.join("current-bundles.json");
    let current_delta = root.join("current-delta.json");
    let current_regime = root.join("current-regime.json");
    let current_output = root.join("current-out");
    let current_decision_ms = 7_201_300;
    write_json(
        &current_input,
        &Value::Array(vec![bundle_json_with_gate_inputs(999, current_decision_ms)]),
    );
    write_json(
        &current_delta,
        &Value::Array(vec![market_delta_json(
            "current_delta_sui",
            current_decision_ms,
            current_decision_ms + 3_600_000,
            0.5,
        )]),
    );
    write_json(
        &current_regime,
        &Value::Array(vec![market_regime_json(
            "current_regime_sui",
            current_decision_ms,
            current_decision_ms + 3_600_000,
        )]),
    );

    let summary = run(Args {
        input_bundle_file: Some(current_input),
        market_feature_delta_file: Some(current_delta),
        market_regime_context_file: Some(current_regime),
        historical_replay_run_index_files: vec![history_index_file],
        output_dir: Some(current_output),
        research_packet_id: "packet_test".to_owned(),
        run_scope: "test".to_owned(),
        now_ms: Some(124_000_000),
        ..default_args()
    })
    .await
    .expect("current run succeeds");

    assert_eq!(summary.replay_runs_created, 1);
    assert_eq!(summary.historical_replay_runs_loaded, 1);
    let report: Value =
        serde_json::from_str(&fs::read_to_string(&summary.output_files[0]).expect("report exists"))
            .expect("report json parses");
    assert_eq!(report["partition_count"], json!(1));
    assert_eq!(
        report["partition_aggregates"][0]["symbol_canonical"],
        json!("SUI")
    );
    assert_eq!(
        report["partition_aggregates"][0]["replay_run_count"],
        json!(2)
    );
}

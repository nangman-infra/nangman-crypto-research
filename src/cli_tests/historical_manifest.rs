use super::*;

#[tokio::test]
async fn historical_replay_runs_are_loaded_into_decay_aware_aggregate() {
    let root = test_root("historical-aggregate");
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
        shadow_cycle_decision_file: None,
        input_manifest_file: None,
        input_manifest_s3_bucket: None,
        input_manifest_s3_key: None,
        input_bundle_file: Some(history_input),
        input_bundle_s3_bucket: None,
        input_bundle_s3_key: None,
        market_feature_delta_file: Some(history_delta),
        market_regime_context_file: Some(history_regime),
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
        output_dir: Some(history_output),
        output_s3_bucket: None,
        output_s3_prefix: None,
        research_packet_id: "packet_test".to_owned(),
        run_scope: "test".to_owned(),
        now_ms: Some(120_000_000),
        ..default_args()
    })
    .await
    .expect("history run succeeds");
    assert_eq!(history_summary.replay_runs_created, 30);
    assert_eq!(history_summary.shadow_validation_runs_created, 30);
    let history_index_file = output_file_containing(&history_summary, "/replay-run-index/");

    let current_input = root.join("current-bundles.json");
    let current_delta = root.join("current-delta.json");
    let current_regime = root.join("current-regime.json");
    let current_output = root.join("current-out");
    let current_decision_ms = 1_300 + (30 * 3_600_000);
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
        shadow_cycle_decision_file: None,
        input_manifest_file: None,
        input_manifest_s3_bucket: None,
        input_manifest_s3_key: None,
        input_bundle_file: Some(current_input),
        input_bundle_s3_bucket: None,
        input_bundle_s3_key: None,
        market_feature_delta_file: Some(current_delta),
        market_regime_context_file: Some(current_regime),
        market_l1_s3_bucket: None,
        market_feature_delta_s3_keys: Vec::new(),
        market_regime_context_s3_keys: Vec::new(),
        historical_replay_run_files: Vec::new(),
        historical_replay_run_index_files: vec![history_index_file],
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
        output_dir: Some(current_output),
        output_s3_bucket: None,
        output_s3_prefix: None,
        research_packet_id: "packet_test".to_owned(),
        run_scope: "test".to_owned(),
        now_ms: Some(124_000_000),
        ..default_args()
    })
    .await
    .expect("current run succeeds");

    assert_eq!(summary.replay_runs_created, 1);
    assert_eq!(summary.historical_replay_runs_loaded, 30);
    assert_eq!(summary.shadow_validation_runs_created, 1);
    let report: Value =
        serde_json::from_str(&fs::read_to_string(&summary.output_files[0]).expect("report exists"))
            .expect("report json parses");
    let aggregate = &report["partition_aggregates"][0];
    assert_eq!(aggregate["gate_bias"], json!("PROMOTE_TO_SHADOW_BIAS"));
    assert_eq!(aggregate["replay_run_count"], json!(31));
    assert_eq!(aggregate["active_replay_run_count"], json!(31));
    assert_eq!(aggregate["expired_replay_run_count"], json!(0));
    assert_eq!(aggregate["completed_count"], json!(31));
    assert_eq!(aggregate["expired_completed_count"], json!(0));
    assert_eq!(aggregate["effective_completed_sample_weight"], json!(31.0));
    assert_eq!(aggregate["weighted_mean_net_after_cost_bps"], json!(33.0));
    assert_eq!(
        aggregate["gate_reason_codes"],
        json!(["deterministic_shadow_gate_passed"])
    );
    assert_eq!(
        report["summary_findings"][0]["bias"],
        json!("PROMOTE_TO_SHADOW_BIAS")
    );
    assert_eq!(
        report["shadow_validation_runs"]
            .as_array()
            .expect("shadow runs are present")
            .len(),
        1
    );
}

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
        shadow_cycle_decision_file: None,
        input_manifest_file: None,
        input_manifest_s3_bucket: None,
        input_manifest_s3_key: None,
        input_bundle_file: Some(history_input),
        input_bundle_s3_bucket: None,
        input_bundle_s3_key: None,
        market_feature_delta_file: Some(history_delta),
        market_regime_context_file: Some(history_regime),
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
        output_dir: Some(history_output),
        output_s3_bucket: None,
        output_s3_prefix: None,
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
        shadow_cycle_decision_file: None,
        input_manifest_file: None,
        input_manifest_s3_bucket: None,
        input_manifest_s3_key: None,
        input_bundle_file: Some(current_input),
        input_bundle_s3_bucket: None,
        input_bundle_s3_key: None,
        market_feature_delta_file: Some(current_delta),
        market_regime_context_file: Some(current_regime),
        market_l1_s3_bucket: None,
        market_feature_delta_s3_keys: Vec::new(),
        market_regime_context_s3_keys: Vec::new(),
        historical_replay_run_files: Vec::new(),
        historical_replay_run_index_files: vec![history_index_file],
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
        output_dir: Some(current_output),
        output_s3_bucket: None,
        output_s3_prefix: None,
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
        shadow_cycle_decision_file: None,
        input_manifest_file: None,
        input_manifest_s3_bucket: None,
        input_manifest_s3_key: None,
        input_bundle_file: Some(history_input),
        input_bundle_s3_bucket: None,
        input_bundle_s3_key: None,
        market_feature_delta_file: Some(history_delta),
        market_regime_context_file: Some(history_regime),
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
        output_dir: Some(history_output),
        output_s3_bucket: None,
        output_s3_prefix: None,
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
        shadow_cycle_decision_file: None,
        input_manifest_file: None,
        input_manifest_s3_bucket: None,
        input_manifest_s3_key: None,
        input_bundle_file: Some(current_input),
        input_bundle_s3_bucket: None,
        input_bundle_s3_key: None,
        market_feature_delta_file: Some(current_delta),
        market_regime_context_file: Some(current_regime),
        market_l1_s3_bucket: None,
        market_feature_delta_s3_keys: Vec::new(),
        market_regime_context_s3_keys: Vec::new(),
        historical_replay_run_files: vec![history_replay_file],
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
        output_dir: Some(current_output),
        output_s3_bucket: None,
        output_s3_prefix: None,
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

#[tokio::test]
async fn lookahead_mismatch_is_invalid_input() {
    let root = test_root("lookahead");
    let input = root.join("bundles.jsonl");
    let output = root.join("out");
    let mut bundle = bundle_json();
    bundle["forbidden_lookahead_boundary_ms"] = json!(1_299);
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
    .expect("run succeeds with partial report");

    let report_text = fs::read_to_string(&summary.output_files[0]).expect("report exists");
    assert!(report_text.contains("invalid_input"));
    assert!(report_text.contains("lookahead_boundary_mismatch"));
}

#[tokio::test]
async fn report_id_and_output_key_are_stable_without_now_ms() {
    let root = test_root("stable-report");
    let input = root.join("bundles.jsonl");
    let output_a = root.join("out-a");
    let output_b = root.join("out-b");
    write_json(&input, &bundle_json());

    let args = |output_dir: PathBuf| Args {
        shadow_cycle_decision_file: None,
        input_manifest_file: None,
        input_manifest_s3_bucket: None,
        input_manifest_s3_key: None,
        input_bundle_file: Some(input.clone()),
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
        output_dir: Some(output_dir),
        output_s3_bucket: None,
        output_s3_prefix: None,
        research_packet_id: "packet_test".to_owned(),
        run_scope: "test".to_owned(),
        now_ms: None,
        ..default_args()
    };

    let summary_a = run(args(output_a.clone()))
        .await
        .expect("first run succeeds");
    let summary_b = run(args(output_b.clone()))
        .await
        .expect("second run succeeds");
    let report_a: Value = serde_json::from_str(
        &fs::read_to_string(&summary_a.output_files[0]).expect("first report exists"),
    )
    .expect("first report json parses");
    let report_b: Value = serde_json::from_str(
        &fs::read_to_string(&summary_b.output_files[0]).expect("second report exists"),
    )
    .expect("second report json parses");

    assert_eq!(
        report_a["research_run_report_id"],
        report_b["research_run_report_id"]
    );
    assert_eq!(report_a["created_at_ms"], json!(7_200_000));
    assert_eq!(report_b["created_at_ms"], json!(7_200_000));
    let relative_a = Path::new(&summary_a.output_files[0])
        .strip_prefix(&output_a)
        .expect("first output is under output dir");
    let relative_b = Path::new(&summary_b.output_files[0])
        .strip_prefix(&output_b)
        .expect("second output is under output dir");
    assert_eq!(relative_a, relative_b);
}

#[tokio::test]
async fn retest_cycle_source_state_links_manifest_and_report_for_scheduler() {
    let root = test_root("retest-source-state");
    let input = root.join("bundles.jsonl");
    let output = root.join("out");
    write_json(&input, &bundle_json());

    let summary = run(Args {
        input_bundle_file: Some(input),
        output_dir: Some(output),
        research_packet_id: "packet_state_test".to_owned(),
        run_scope: "focused_retest_local_validation".to_owned(),
        now_ms: Some(1_800_000),
        ..default_args()
    })
    .await
    .expect("research run succeeds");
    let report_file = output_file_containing(&summary, "research-run-report");
    let report = crate::io::read_research_run_report(&report_file).expect("report parses");
    let state = build_retest_cycle_source_state(
        1_900_000,
        "research-bucket",
        "research-input-manifest/schema=research_input_manifest_v1/dedupe_key=packet/manifest.json",
        "research-bucket",
        "research-run-report/schema=research_run_report_v1/dt=1970-01-01/hour=00/research_run_report_id=report/report.json",
        &report,
    );

    assert_eq!(
        state.schema_version,
        crate::model::RETEST_CYCLE_SOURCE_STATE_SCHEMA_VERSION
    );
    assert_eq!(state.research_packet_id, "packet_state_test");
    assert_eq!(state.run_scope, "focused_retest_local_validation");
    assert_eq!(state.source_candidate_ids, vec!["cand_001".to_owned()]);
    assert_eq!(state.replay_run_id_count, report.replay_run_ids.len());
    assert!(!state.safety.shadow_paper_live_enabled);
    assert_eq!(
        research_report_s3_key_from_output_files(
            "research-bucket",
            &[format!(
                "s3://research-bucket/research-run-report/schema=research_run_report_v1/dt=1970-01-01/hour=00/research_run_report_id={}/report.json",
                report.research_run_report_id
            )],
            &report.research_run_report_id,
        )
        .expect("report key is extracted"),
        format!(
            "research-run-report/schema=research_run_report_v1/dt=1970-01-01/hour=00/research_run_report_id={}/report.json",
            report.research_run_report_id
        )
    );
}

#[test]
fn output_partition_uses_execution_time_without_rewriting_report_time() {
    let root = test_root("output-partition-time");
    let bundles = vec![
        serde_json::from_value(bundle_json()).expect("candidate bundle test json matches model"),
    ];
    let report =
        crate::report::build_report("packet_test", "test", 7_200_000, &bundles, &[], &[], &[]);

    let output_artifacts = crate::io::ResearchOutputArtifacts {
        report: &report,
        replay_runs: &[],
        shadow_validation_runs: &[],
        paper_watch_candidates: &[],
        paper_trade_candidates: &[],
        paper_trade_runs: &[],
        paper_trade_summaries: &[],
        paper_trade_marks: &[],
        output_partition_at_ms: 3_600_000,
    };
    let written = crate::io::write_research_outputs(&root, &output_artifacts).expect("write ok");

    let relative = written[0]
        .strip_prefix(&root)
        .expect("output is under test root")
        .display()
        .to_string();
    assert!(
        relative.contains("dt=1970-01-01/hour=01"),
        "output partition should use execution time, got {relative}"
    );
    let report_json: Value =
        serde_json::from_str(&fs::read_to_string(&written[0]).expect("report exists"))
            .expect("report json parses");
    assert_eq!(report_json["created_at_ms"], json!(7_200_000));
}

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
        shadow_cycle_decision_file: None,
        input_manifest_file: Some(manifest),
        input_manifest_s3_bucket: None,
        input_manifest_s3_key: None,
        input_bundle_file: None,
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
        shadow_cycle_decision_file: None,
        input_manifest_file: Some(manifest),
        input_manifest_s3_bucket: None,
        input_manifest_s3_key: None,
        input_bundle_file: None,
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
        output_dir: Some(root.join("out")),
        output_s3_bucket: None,
        output_s3_prefix: None,
        research_packet_id: "packet_test".to_owned(),
        run_scope: "test".to_owned(),
        now_ms: Some(7_300_000),
        ..default_args()
    })
    .await
    .expect_err("oversized manifest is rejected");

    assert!(error.to_string().contains("runtime budget exceeded"));
}

#[tokio::test]
async fn derives_market_l1_s3_keys_from_candidate_bundle() {
    let mut bundle = bundle_json();
    bundle["data_quality_summary"]["market_data_quality_summary_key"] =
        json!("market_data_quality_summary/run_id=l1_001/summary.json");
    bundle["selected_market_artifacts"] = json!([
        {
            "artifact_type": "market_feature_delta",
            "artifact_id": "delta_002",
            "artifact_key": "s3://nangman-crypto-dev-market-ingest-l1-<account-suffix>/market_feature_delta/run_id=l1_direct/delta.json",
            "l1_run_id": "l1_direct",
            "symbol_canonical": "SUI",
            "metric_name": "price",
            "window_start_ms": 1_000,
            "window_end_ms": 1_300,
            "known_as_of_ms": 1_300,
            "quality_status": "available"
        },
        {
            "artifact_type": "market_feature_delta_summary",
            "artifact_id": "delta_summary_001",
            "artifact_key": "market_feature_delta_summary/run_id=l1_selected/summary.json",
            "l1_run_id": "l1_selected",
            "symbol_canonical": "SUI",
            "metric_name": "price",
            "window_start_ms": 1_000,
            "window_end_ms": 1_300,
            "known_as_of_ms": 1_300,
            "quality_status": "available"
        },
        {
            "artifact_type": "market_feature_delta_summary",
            "artifact_id": "delta_summary_002",
            "artifact_key": "market_feature_delta_summary/run_id=l1_summary_key_only/summary.json",
            "symbol_canonical": "SUI",
            "metric_name": "price",
            "window_start_ms": 1_000,
            "window_end_ms": 1_300,
            "known_as_of_ms": 1_300,
            "quality_status": "available"
        },
        {
            "artifact_type": "market_regime_context",
            "artifact_id": "regime_001",
            "artifact_key": "s3://nangman-crypto-dev-market-ingest-l1-<account-suffix>/market_regime_context/run_id=l1_selected/context.json",
            "l1_run_id": "l1_selected",
            "scope": "market",
            "window_start_ms": 1_000,
            "window_end_ms": 1_300,
            "known_as_of_ms": 1_300,
            "quality_status": "available"
        }
    ]);
    let bundles =
        vec![serde_json::from_value(bundle).expect("candidate bundle test json matches model")];
    let args = Args {
        shadow_cycle_decision_file: None,
        input_manifest_file: None,
        input_manifest_s3_bucket: None,
        input_manifest_s3_key: None,
        input_bundle_file: None,
        input_bundle_s3_bucket: Some(
            "nangman-crypto-dev-intel-candidate-<account-suffix>".to_owned(),
        ),
        input_bundle_s3_key: Some(
            "candidate-evidence-bundle/priority=p0/part-000001.jsonl".to_owned(),
        ),
        market_feature_delta_file: None,
        market_regime_context_file: None,
        market_l1_s3_bucket: None,
        market_feature_delta_s3_keys: vec![
            "market_feature_delta/run_id=l1_cli/delta.json".to_owned(),
        ],
        market_regime_context_s3_keys: vec![
            "market_regime_context/run_id=l1_cli/context.json".to_owned(),
        ],
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
        output_dir: None,
        output_s3_bucket: None,
        output_s3_prefix: None,
        research_packet_id: "packet_test".to_owned(),
        run_scope: "test".to_owned(),
        now_ms: Some(0),
        ..default_args()
    };

    assert_eq!(
        market_feature_delta_s3_keys(&args, &bundles)
            .await
            .expect("market feature delta keys derive"),
        vec![
            "market_feature_delta/run_id=l1_001/delta.json",
            "market_feature_delta/run_id=l1_cli/delta.json",
            "market_feature_delta/run_id=l1_direct/delta.json",
            "market_feature_delta/run_id=l1_selected/delta.json",
            "market_feature_delta/run_id=l1_summary_key_only/delta.json",
        ]
    );
    assert_eq!(
        market_regime_context_s3_keys(&args, &bundles)
            .await
            .expect("market regime context keys derive"),
        vec![
            "market_regime_context/run_id=l1_001/context.json",
            "market_regime_context/run_id=l1_cli/context.json",
            "market_regime_context/run_id=l1_selected/context.json",
        ]
    );
}

#[test]
fn skips_replay_window_discovery_for_invalid_or_missing_horizon_bundle() {
    let mut invalid_bundle = bundle_json_with_gate_inputs(0, 1_300);
    invalid_bundle["research_eligible"] = json!(false);
    let mut missing_horizon_bundle = bundle_json_with_gate_inputs(1, 1_300);
    missing_horizon_bundle["allowed_horizons"] = json!(["unsupported"]);
    let bundles = vec![
        serde_json::from_value(invalid_bundle).expect("invalid bundle json matches model"),
        serde_json::from_value(missing_horizon_bundle)
            .expect("missing horizon bundle json matches model"),
    ];

    assert_eq!(
        market_l1_replay_window_starts(&bundles, 2_100_000),
        Vec::<i64>::new()
    );
}

#[tokio::test]
async fn market_s3_key_budget_is_enforced_before_reading_s3_objects() {
    let mut bundle = bundle_json();
    bundle["selected_market_artifacts"] = json!([
        {
            "artifact_type": "market_feature_delta",
            "artifact_id": "delta_001",
            "artifact_key": "market_feature_delta/run_id=l1_selected/delta.json",
            "l1_run_id": "l1_selected",
            "symbol_canonical": "SUI",
            "metric_name": "price",
            "window_start_ms": 1_000,
            "window_end_ms": 1_300,
            "known_as_of_ms": 1_300,
            "quality_status": "available"
        },
        {
            "artifact_type": "market_regime_context",
            "artifact_id": "regime_001",
            "artifact_key": "market_regime_context/run_id=l1_selected/context.json",
            "l1_run_id": "l1_selected",
            "scope": "market",
            "window_start_ms": 1_000,
            "window_end_ms": 1_300,
            "known_as_of_ms": 1_300,
            "quality_status": "available"
        }
    ]);
    let bundles =
        vec![serde_json::from_value(bundle).expect("candidate bundle test json matches model")];
    let args = Args {
        shadow_cycle_decision_file: None,
        input_manifest_file: None,
        input_manifest_s3_bucket: None,
        input_manifest_s3_key: None,
        input_bundle_file: None,
        input_bundle_s3_bucket: Some(
            "nangman-crypto-dev-intel-candidate-<account-suffix>".to_owned(),
        ),
        input_bundle_s3_key: Some(
            "candidate-evidence-bundle/priority=p0/part-000001.jsonl".to_owned(),
        ),
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
        output_dir: None,
        output_s3_bucket: None,
        output_s3_prefix: None,
        research_packet_id: "packet_test".to_owned(),
        run_scope: "test".to_owned(),
        now_ms: Some(0),
        ..default_args()
    };

    let delta_error = load_market_deltas(&args, &bundles, None, 0)
        .await
        .expect_err("market delta key budget fails before S3 read");
    assert!(
        delta_error
            .to_string()
            .contains("market_feature_delta_s3_key_count")
    );

    let context_error = load_regime_contexts(&args, &bundles, None, 0)
        .await
        .expect_err("market context key budget fails before S3 read");
    assert!(
        context_error
            .to_string()
            .contains("market_regime_context_s3_key_count")
    );
}

#[test]
fn derives_market_l1_replay_window_starts_from_candidate_horizons() {
    let mut bundle = bundle_json_with_gate_inputs(0, 1_300);
    bundle["allowed_horizons"] = json!(["1h", "4h", "24h"]);
    let bundles =
        vec![serde_json::from_value(bundle).expect("candidate bundle test json matches model")];

    assert_eq!(
        market_l1_replay_window_starts(&bundles, 2_100_000),
        vec![0, 900_000, 1_800_000]
    );
}

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

#[tokio::test]
async fn partial_market_replay_window_stays_insufficient_until_horizon_materializes() {
    let root = test_root("partial-horizon");
    let input = root.join("bundles.jsonl");
    let delta = root.join("delta.json");
    let output = root.join("out");
    write_json(&input, &bundle_json());
    write_json(
        &delta,
        &json!([market_delta_json("delta_partial", 1_300, 901_300, 0.5)]),
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

    let replay_file = output_file_containing(&summary, "/replay-run/");
    let replay_text = fs::read_to_string(&replay_file).expect("replay output exists");
    let replay: Value = serde_json::from_str(
        replay_text
            .lines()
            .next()
            .expect("replay output has one line"),
    )
    .expect("replay line parses");
    assert_eq!(
        replay["result_summary"]["status"],
        json!("insufficient_evidence")
    );
    assert_eq!(
        replay["result_summary"]["reason_codes"],
        json!(["native_replay_horizon_not_materialized"])
    );
    assert_eq!(replay["result_summary"]["raw_return_bps"], Value::Null);

    let report_text = fs::read_to_string(&summary.output_files[0]).expect("report exists");
    assert!(report_text.contains("native_replay_horizon_not_materialized"));
    assert!(!report_text.contains("native_replay_positive_but_survival_not_proven"));
}

#[tokio::test]
async fn positive_single_replay_stays_retest_until_gate_evidence_exists() {
    let root = test_root("positive-single-gated");
    let input = root.join("bundles.jsonl");
    let delta = root.join("delta.json");
    let regime = root.join("regime.json");
    let output = root.join("out");
    write_json(&input, &bundle_json());
    write_json(
        &delta,
        &json!([market_delta_json("delta_001", 1_300, 3_601_300, 0.5)]),
    );
    write_json(
        &regime,
        &json!([market_regime_json("regime_001", 1_300, 3_601_300)]),
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
        now_ms: Some(1_800_000),
        ..default_args()
    })
    .await
    .expect("run succeeds");
    assert_eq!(summary.shadow_validation_runs_created, 0);

    let report: Value =
        serde_json::from_str(&fs::read_to_string(&summary.output_files[0]).expect("report exists"))
            .expect("report json parses");
    assert_eq!(report["summary_findings"][0]["bias"], json!("RETEST_BIAS"));
    assert_eq!(report["shadow_validation_runs"], json!([]));
    let gate_reasons = report["partition_aggregates"][0]["gate_reason_codes"]
        .as_array()
        .expect("gate reasons are an array");
    assert!(gate_reasons.contains(&json!("promotion_sample_count_below_minimum")));
    assert!(gate_reasons.contains(&json!("train_validation_split_not_materialized")));
    assert!(gate_reasons.contains(&json!("liquidity_filter_not_materialized")));

    let replay_index_file = output_file_containing(&summary, "/replay-run-index/");
    let replay_index_text =
        fs::read_to_string(&replay_index_file).expect("replay index output exists");
    let replay_index: Value = serde_json::from_str(
        replay_index_text
            .lines()
            .next()
            .expect("replay index has one line"),
    )
    .expect("replay index line parses");
    assert_eq!(replay_index["schema_version"], json!("replay_run_index_v1"));
    assert_eq!(
        replay_index["research_aggregate_key"],
        report["partition_aggregates"][0]["research_aggregate_key"]
    );
    assert!(
        replay_index["replay_run_uri"]
            .as_str()
            .expect("replay run uri is present")
            .contains("/replay-run/")
    );
    assert_eq!(replay_index["replay_run_s3_bucket"], Value::Null);
    assert_eq!(replay_index["replay_run_s3_key"], Value::Null);

    let registry_file = output_file_containing(&summary, "/research-aggregate-registry/");
    let registry_text = fs::read_to_string(&registry_file).expect("registry output exists");
    let registry: Value = serde_json::from_str(
        registry_text
            .lines()
            .next()
            .expect("registry output has one line"),
    )
    .expect("registry line parses");
    assert_eq!(
        registry["schema_version"],
        json!("research_aggregate_registry_record_v1")
    );
    assert_eq!(registry["current_research_stage"], json!("retest"));
    assert_eq!(registry["gate_bias"], json!("RETEST_BIAS"));
    assert_eq!(registry["linked_shadow_validation_run_ids"], json!([]));
    assert!(!registry_text.contains("EXECUTION_APPROVED"));
    assert!(!registry_text.contains("LIVE_READY"));
}

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

#[tokio::test]
async fn aggregate_gate_promotes_only_to_shadow_when_enterprise_blockers_clear() {
    let root = test_root("aggregate-shadow");
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
        bundles.push(bundle_json_with_gate_inputs(index, decision_ms));
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
    assert_eq!(summary.shadow_validation_runs_created, 31);
    let shadow_output_file = output_file_containing(&summary, "/shadow-validation-run/");
    let shadow_output_text =
        fs::read_to_string(&shadow_output_file).expect("shadow validation output exists");
    assert_eq!(shadow_output_text.lines().count(), 31);
    assert!(!shadow_output_text.contains("EXECUTION_APPROVED"));
    assert!(!shadow_output_text.contains("LIVE_READY"));

    let report: Value =
        serde_json::from_str(&fs::read_to_string(&summary.output_files[0]).expect("report exists"))
            .expect("report json parses");
    assert_eq!(
        report["partition_aggregates"][0]["gate_bias"],
        json!("PROMOTE_TO_SHADOW_BIAS")
    );
    assert_eq!(report["paper_trade_candidates"], json!([]));
    assert_eq!(
        report["research_gate_policy"]["allow_promote_to_paper_bias"],
        json!(false)
    );
    assert_eq!(
        report["partition_aggregates"][0]["train_validation_split_summary"]["passed"],
        json!(true)
    );
    assert_eq!(
        report["partition_aggregates"][0]["cost_stressed_mean_net_after_cost_bps"],
        json!(16.0)
    );
    assert_eq!(
        report["partition_aggregates"][0]["gate_reason_codes"],
        json!(["deterministic_shadow_gate_passed"])
    );
    assert_eq!(
        report["partition_aggregates"][0]["completed_count"],
        json!(31)
    );
    assert_eq!(
        report["partition_aggregates"][0]["inferred_unseen_window_count"],
        json!(30)
    );
    assert_eq!(
        report["shadow_validation_runs"]
            .as_array()
            .expect("shadow run ids are present")
            .len(),
        31
    );
    assert_eq!(
        report["shadow_validation_runs"][0]["schema_version"],
        json!("shadow_validation_run_v1")
    );
    assert_eq!(
        report["shadow_validation_runs"][0]["watch_window_policy"]["mode"],
        json!("forward_observation_only")
    );
    assert_eq!(
        report["shadow_validation_runs"][0]["termination_policy"]["no_order_execution"],
        json!(true)
    );
    let registry_file = output_file_containing(&summary, "/research-aggregate-registry/");
    let registry_text = fs::read_to_string(&registry_file).expect("registry output exists");
    let registry: Value = serde_json::from_str(
        registry_text
            .lines()
            .next()
            .expect("registry output has one line"),
    )
    .expect("registry line parses");
    assert_eq!(
        registry["current_research_stage"],
        json!("shadow_candidate")
    );
    assert_eq!(registry["gate_bias"], json!("PROMOTE_TO_SHADOW_BIAS"));
    assert_eq!(
        registry["linked_shadow_validation_run_ids"]
            .as_array()
            .expect("shadow validation ids are recorded")
            .len(),
        31
    );
    let report_text = serde_json::to_string(&report).expect("report serializes");
    assert!(!report_text.contains("EXECUTION_APPROVED"));
    assert!(!report_text.contains("LIVE_READY"));
}

#[tokio::test]
async fn completed_shadow_validation_input_creates_paper_artifacts_without_live_approval() {
    let root = test_root("paper-from-shadow");
    let input = root.join("bundles.json");
    let delta = root.join("delta.json");
    let regime = root.join("regime.json");
    let shadow_output = root.join("shadow-out");
    let paper_output = root.join("paper-out");
    let completed_shadow_file = root.join("completed-shadow.json");
    let mut bundles = Vec::new();
    let mut deltas = Vec::new();
    let mut regimes = Vec::new();

    for index in 0..31 {
        let decision_ms = 1_300 + (index as i64 * 3_600_000);
        let window_end_ms = decision_ms + 3_600_000;
        bundles.push(bundle_json_with_gate_inputs(index, decision_ms));
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

    let shadow_summary = run(Args {
        shadow_cycle_decision_file: None,
        input_manifest_file: None,
        input_manifest_s3_bucket: None,
        input_manifest_s3_key: None,
        input_bundle_file: Some(input.clone()),
        input_bundle_s3_bucket: None,
        input_bundle_s3_key: None,
        market_feature_delta_file: Some(delta.clone()),
        market_regime_context_file: Some(regime.clone()),
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
        output_dir: Some(shadow_output),
        output_s3_bucket: None,
        output_s3_prefix: None,
        research_packet_id: "packet_test".to_owned(),
        run_scope: "test".to_owned(),
        now_ms: Some(120_000_000),
        ..default_args()
    })
    .await
    .expect("shadow run succeeds");

    let shadow_output_file = output_file_containing(&shadow_summary, "/shadow-validation-run/");
    let completed_shadow_runs = fs::read_to_string(&shadow_output_file)
        .expect("shadow output exists")
        .lines()
        .map(|line| {
            let mut run: Value = serde_json::from_str(line).expect("shadow line parses");
            run["status"] = json!("completed");
            run["passed"] = json!(true);
            run["paper_trade_candidate_contract_version"] = json!("paper_trade_candidate_v1");
            run
        })
        .collect::<Vec<_>>();
    write_json(&completed_shadow_file, &Value::Array(completed_shadow_runs));

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
        shadow_validation_run_files: vec![completed_shadow_file],
        oss_adapter_run_s3_bucket: None,
        oss_adapter_run_s3_keys: Vec::new(),
        shadow_validation_run_s3_bucket: None,
        shadow_validation_run_s3_keys: Vec::new(),
        historical_replay_run_s3_bucket: None,
        historical_replay_run_s3_keys: Vec::new(),
        historical_replay_run_index_s3_bucket: None,
        historical_replay_run_index_s3_keys: Vec::new(),
        output_dir: Some(paper_output),
        output_s3_bucket: None,
        output_s3_prefix: None,
        research_packet_id: "packet_test".to_owned(),
        run_scope: "test".to_owned(),
        now_ms: Some(120_000_000),
        ..default_args()
    })
    .await
    .expect("paper run succeeds");

    assert_eq!(summary.shadow_validation_runs_loaded, 31);
    assert_eq!(summary.shadow_validation_runs_created, 0);
    assert_eq!(summary.paper_trade_candidates_created, 31);
    assert_eq!(summary.paper_trade_runs_created, 31);
    assert_eq!(summary.paper_trade_summaries_created, 31);
    assert_eq!(summary.paper_trade_marks_created, 31);

    let report: Value =
        serde_json::from_str(&fs::read_to_string(&summary.output_files[0]).expect("report exists"))
            .expect("report json parses");
    assert_eq!(
        report["summary_findings"][0]["bias"],
        json!("PROMOTE_TO_PAPER_BIAS")
    );
    assert_eq!(
        report["paper_trade_candidates"]
            .as_array()
            .expect("paper candidate ids")
            .len(),
        31
    );
    let candidate_file = output_file_containing(&summary, "/paper-trade-candidate/");
    let run_file = output_file_containing(&summary, "/paper-trade-run/");
    let summary_file = output_file_containing(&summary, "/paper-trade-summary/");
    let mark_file = output_file_containing(&summary, "/paper-trade-mark/");
    assert_eq!(
        fs::read_to_string(candidate_file)
            .expect("candidate output exists")
            .lines()
            .count(),
        31
    );
    assert_eq!(
        fs::read_to_string(run_file)
            .expect("run output exists")
            .lines()
            .count(),
        31
    );
    assert_eq!(
        fs::read_to_string(summary_file)
            .expect("summary output exists")
            .lines()
            .count(),
        31
    );
    assert_eq!(
        fs::read_to_string(mark_file)
            .expect("mark output exists")
            .lines()
            .count(),
        31
    );
    let registry_file = output_file_containing(&summary, "/research-aggregate-registry/");
    let registry_text = fs::read_to_string(&registry_file).expect("registry output exists");
    let registry: Value = serde_json::from_str(
        registry_text
            .lines()
            .next()
            .expect("registry output has one line"),
    )
    .expect("registry line parses");
    assert_eq!(
        registry["current_research_stage"],
        json!("paper_candidate_bias")
    );
    let report_text = serde_json::to_string(&report).expect("report serializes");
    assert!(!report_text.contains("EXECUTION_APPROVED"));
    assert!(!report_text.contains("LIVE_READY"));
}

#[tokio::test]
async fn data_missing_retest_does_not_create_paper_watch() {
    let root = test_root("paper-watch-data-missing");
    let input = root.join("bundles.json");
    let output = root.join("out");

    write_json(
        &input,
        &Value::Array(vec![bundle_json_with_gate_inputs(8, 1_300)]),
    );

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
        now_ms: Some(120_000_000),
        ..default_args()
    })
    .await
    .expect("research run succeeds");

    let report: Value =
        serde_json::from_str(&fs::read_to_string(&summary.output_files[0]).expect("report exists"))
            .expect("report json parses");
    assert_eq!(report["summary_findings"][0]["bias"], json!("RETEST_BIAS"));
    assert_eq!(report["paper_watch_candidates"], json!([]));
    assert!(
        summary
            .output_files
            .iter()
            .all(|path| !path.contains("/paper-watch-candidate/"))
    );
}

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

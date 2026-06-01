use super::*;

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

use super::*;

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

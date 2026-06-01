use super::*;

#[tokio::test]
async fn positive_retest_creates_paper_watch_without_live_or_order_approval() {
    let root = test_root("paper-watch-positive-retest");
    let input = root.join("bundles.json");
    let delta = root.join("delta.json");
    let regime = root.join("regime.json");
    let output = root.join("out");
    let decision_ms = 1_300;
    let window_end_ms = decision_ms + 3_600_000;

    write_json(
        &input,
        &Value::Array(vec![bundle_json_with_gate_inputs(7, decision_ms)]),
    );
    write_json(
        &delta,
        &Value::Array(vec![market_delta_json(
            "delta_positive",
            decision_ms,
            window_end_ms,
            0.5,
        )]),
    );
    write_json(
        &regime,
        &Value::Array(vec![market_regime_json(
            "regime_positive",
            decision_ms,
            window_end_ms,
        )]),
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
    .expect("research run succeeds");

    assert_eq!(summary.shadow_validation_runs_created, 0);
    assert_eq!(summary.paper_trade_candidates_created, 0);
    assert_eq!(summary.paper_trade_runs_created, 0);
    let report: Value =
        serde_json::from_str(&fs::read_to_string(&summary.output_files[0]).expect("report exists"))
            .expect("report json parses");
    assert_eq!(report["summary_findings"][0]["bias"], json!("RETEST_BIAS"));
    assert_eq!(
        report["paper_watch_candidates"]
            .as_array()
            .expect("paper watch ids")
            .len(),
        1
    );
    assert_eq!(report["paper_trade_candidates"], json!([]));
    assert_eq!(report["shadow_validation_runs"], json!([]));

    let watch_file = output_file_containing(&summary, "/paper-watch-candidate/");
    let watch_text = fs::read_to_string(watch_file).expect("paper watch output exists");
    assert!(!watch_text.contains("EXECUTION_APPROVED"));
    assert!(!watch_text.contains("LIVE_READY"));
    let watch: Value = serde_json::from_str(watch_text.lines().next().expect("watch line exists"))
        .expect("watch json parses");
    assert_eq!(watch["schema_version"], json!("paper_watch_candidate_v1"));
    assert_eq!(watch["source_research_bias"], json!("RETEST_BIAS"));
    assert_eq!(watch["safety"]["paper_only"], json!(true));
    assert_eq!(watch["safety"]["live_enabled"], json!(false));
    assert_eq!(watch["safety"]["order_execution_enabled"], json!(false));
    assert_eq!(watch["safety"]["execution_approval_emitted"], json!(false));
    assert_eq!(
        watch["admission_reason_codes"],
        json!([
            "retest_positive_watch_admitted",
            "paper_only_no_order_execution"
        ])
    );
}

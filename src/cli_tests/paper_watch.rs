use super::*;
use serde_json::{Value, json};
use std::fs;

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

#[tokio::test]
async fn paper_watch_live_cycle_marks_live_ticks_without_order_approval() {
    let root = test_root("paper-watch-live-cycle");
    let candidates_file = root.join("paper-watch-candidates.json");
    let ticks_file = root.join("market-live-ticks.json");
    let output = root.join("out");

    write_json(
        &candidates_file,
        &json!([{
            "paper_watch_candidate_id": "watch_001",
            "candidate_id": "cand_001",
            "candidate_lifecycle_key": "cand_001:v1",
            "symbol_canonical": "SUI",
            "source_research_run_id": "research_run_001",
            "source_research_packet_id": "packet_001",
            "source_research_bias": "RETEST_BIAS",
            "historical_survival_band": "stable",
            "admission_reason_codes": ["retest_positive_watch_admitted"],
            "blocked_promotion_reason_codes": ["needs_forward_observation"],
            "replay_sample_summary": {
                "research_aggregate_key": "agg_001",
                "replay_run_count": 10,
                "completed_count": 5,
                "positive_net_count": 3,
                "non_positive_net_count": 2,
                "missing_market_replay_data_count": 0,
                "insufficient_evidence_count": 0,
                "effective_completed_sample_weight": 5.0,
                "weighted_mean_net_after_cost_bps": 12.5,
                "weighted_profit_factor_ppm": 1200000
            },
            "expected_cost_profile": {
                "fee_model_version": "fee",
                "slippage_model_version": "slippage",
                "estimated_cost_bps": 8.0,
                "cost_stressed_mean_net_after_cost_bps": 4.5
            },
            "expected_risk_profile": {
                "survival_band": "stable",
                "max_drawdown_band": "low",
                "positive_net_count": 3,
                "non_positive_net_count": 2
            },
            "target_max_holding_hours": 24,
            "absolute_max_holding_hours": 72,
            "force_flat_policy": "paper_watch_only_no_order_execution",
            "paper_start_recommendation": "start_forward_paper_watch",
            "safety": {
                "paper_only": true,
                "live_enabled": false,
                "order_execution_enabled": false,
                "execution_approval_emitted": false
            },
            "created_at_ms": 1_000,
            "schema_version": "paper_watch_candidate_v1"
        }]),
    );
    write_json(
        &ticks_file,
        &json!([
            market_live_tick_json("tick_001", "SUI", 2_000, 1.0),
            market_live_tick_json("tick_002", "ETH", 2_100, 10.0),
            market_live_tick_json("tick_003", "SUI", 2_200, 1.03)
        ]),
    );

    let summary = run(Args {
        run_paper_watch_live_cycle: true,
        paper_watch_candidate_file: Some(candidates_file),
        market_live_tick_file: Some(ticks_file),
        output_dir: Some(output),
        now_ms: Some(120_000_000),
        ..default_args()
    })
    .await
    .expect("paper watch live cycle succeeds");

    assert_eq!(summary.paper_watch_live_marks_created, 2);
    let mark_file = output_file_containing(&summary, "/paper-watch-live-mark/");
    let mark_text = fs::read_to_string(mark_file).expect("paper watch mark output exists");
    assert!(!mark_text.contains("EXECUTION_APPROVED"));
    assert!(!mark_text.contains("LIVE_READY"));
    let marks = mark_text
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).expect("mark json parses"))
        .collect::<Vec<_>>();
    assert_eq!(marks[0]["safety"]["paper_only"], json!(true));
    assert_eq!(marks[0]["safety"]["live_enabled"], json!(false));
    assert_eq!(marks[0]["safety"]["order_execution_enabled"], json!(false));
    assert_eq!(marks[0]["net_return_bps"], json!(0.0));
    assert_eq!(marks[1]["source_market_live_event_id"], json!("tick_003"));
}

#[test]
fn paper_watch_live_cycle_defaults_nats_subjects_to_candidate_symbols() {
    let candidates = serde_json::from_value::<Vec<crate::model::PaperWatchCandidate>>(json!([
        paper_watch_candidate_json("watch_ton", "TON"),
        paper_watch_candidate_json("watch_zec", "ZEC"),
        paper_watch_candidate_json("watch_ton_duplicate", "ton")
    ]))
    .expect("paper watch candidates parse");
    let args = default_args();

    let configs =
        market_live_nats_configs_for_candidates(&args, &candidates, "nats://127.0.0.1:4222", 123);

    let subjects = configs
        .iter()
        .map(|config| config.subject.as_str())
        .collect::<Vec<_>>();
    let consumers = configs
        .iter()
        .map(|config| config.consumer.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        subjects,
        vec![
            "market_live_tick.created.*.ton",
            "market_live_tick.created.*.zec"
        ]
    );
    assert_eq!(
        consumers,
        vec![
            "research-paper-watch-live-123-ton",
            "research-paper-watch-live-123-zec"
        ]
    );
    assert!(
        configs
            .iter()
            .all(|config| config.url == "nats://127.0.0.1:4222")
    );
    assert!(
        configs
            .iter()
            .all(|config| config.delete_consumer_after_read)
    );
}

#[test]
fn paper_watch_live_cycle_keeps_explicit_nats_subject() {
    let candidates = serde_json::from_value::<Vec<crate::model::PaperWatchCandidate>>(json!([
        paper_watch_candidate_json("watch_ton", "TON"),
        paper_watch_candidate_json("watch_zec", "ZEC")
    ]))
    .expect("paper watch candidates parse");
    let args = Args {
        market_live_nats_subject: "market_live_tick.created.binance.ton".to_owned(),
        market_live_nats_consumer: "custom-consumer".to_owned(),
        ..default_args()
    };

    let configs =
        market_live_nats_configs_for_candidates(&args, &candidates, "nats://127.0.0.1:4222", 123);

    assert_eq!(configs.len(), 1);
    assert_eq!(configs[0].subject, "market_live_tick.created.binance.ton");
    assert_eq!(configs[0].consumer, "custom-consumer");
    assert!(!configs[0].delete_consumer_after_read);
}

#[test]
fn paper_watch_live_cycle_rejects_conflicting_candidate_inputs() {
    let root = test_root("paper-watch-live-conflicting-candidate-inputs");
    let err = parse_args(
        [
            "--run-paper-watch-live-cycle",
            "--paper-watch-candidate-file",
            root.join("paper-watch-candidates.json").to_str().unwrap(),
            "--paper-watch-candidate-s3-bucket",
            "research-bucket",
            "--paper-watch-candidate-s3-key",
            "paper-watch-candidate/example.jsonl",
            "--market-live-tick-file",
            root.join("market-live-ticks.json").to_str().unwrap(),
            "--output-dir",
            root.join("out").to_str().unwrap(),
        ]
        .into_iter()
        .map(str::to_owned),
    )
    .expect_err("conflicting candidate inputs are rejected");

    assert!(
        err.to_string()
            .contains("use either --paper-watch-candidate-file")
    );
}

#[test]
fn paper_watch_live_cycle_rejects_bad_market_live_inputs() {
    let root = test_root("paper-watch-live-bad-market-live-inputs");
    let err = parse_args(
        [
            "--run-paper-watch-live-cycle",
            "--paper-watch-candidate-file",
            root.join("paper-watch-candidates.json").to_str().unwrap(),
            "--market-live-tick-file",
            root.join("market-live-ticks.json").to_str().unwrap(),
            "--market-live-nats-url",
            "nats://127.0.0.1:4222",
            "--output-dir",
            root.join("out").to_str().unwrap(),
        ]
        .into_iter()
        .map(str::to_owned),
    )
    .expect_err("conflicting market live inputs are rejected");

    assert!(
        err.to_string()
            .contains("use either --market-live-tick-file")
    );
}

#[test]
fn paper_watch_live_cycle_rejects_relative_and_non_nats_inputs() {
    let relative_candidate = parse_args(
        [
            "--run-paper-watch-live-cycle",
            "--paper-watch-candidate-file",
            "paper-watch-candidates.json",
            "--market-live-nats-url",
            "nats://127.0.0.1:4222",
            "--output-dir",
            test_root("paper-watch-live-relative-candidate")
                .join("out")
                .to_str()
                .unwrap(),
        ]
        .into_iter()
        .map(str::to_owned),
    )
    .expect_err("relative candidate file is rejected");
    assert!(
        relative_candidate
            .to_string()
            .contains("--paper-watch-candidate-file requires an absolute path")
    );

    let bad_url = parse_args(
        [
            "--run-paper-watch-live-cycle",
            "--paper-watch-candidate-s3-bucket",
            "research-bucket",
            "--paper-watch-candidate-s3-key",
            "paper-watch-candidate/example.jsonl",
            "--market-live-nats-url",
            "http://127.0.0.1:4222",
            "--output-dir",
            test_root("paper-watch-live-bad-nats-url")
                .join("out")
                .to_str()
                .unwrap(),
        ]
        .into_iter()
        .map(str::to_owned),
    )
    .expect_err("non-nats url is rejected");
    assert!(
        bad_url
            .to_string()
            .contains("--market-live-nats-url must start with nats://")
    );
}

#[test]
fn paper_watch_live_cycle_rejects_observer_mode_combo() {
    let root = test_root("paper-watch-live-observer-mode-combo");
    let err = parse_args(
        [
            "--run-paper-watch-live-cycle",
            "--run-paper-watch-observer",
            "--paper-watch-candidate-file",
            root.join("paper-watch-candidates.json").to_str().unwrap(),
            "--market-live-nats-url",
            "nats://127.0.0.1:4222",
            "--output-dir",
            root.join("out").to_str().unwrap(),
        ]
        .into_iter()
        .map(str::to_owned),
    )
    .expect_err("paper watch modes must be isolated");

    assert!(
        err.to_string()
            .contains("use --run-paper-watch-live-cycle separately")
    );
}

#[test]
fn paper_watch_observer_parses_service_mode_with_durable_nats_consumer() {
    let args = parse_args(
        [
            "--run-paper-watch-observer",
            "--paper-watch-candidate-s3-bucket",
            "research-bucket",
            "--paper-watch-candidate-s3-prefix",
            "paper-watch-candidate/schema=paper_watch_candidate_v1",
            "--market-live-nats-url",
            "nats://127.0.0.1:4222",
            "--output-s3-bucket",
            "research-bucket",
            "--output-s3-prefix",
            "paper-watch-observer-state/schema=paper_watch_observer_snapshot_v1",
            "--paper-watch-observer-max-iterations",
            "0",
        ]
        .into_iter()
        .map(str::to_owned),
    )
    .expect("observer mode parses")
    .expect("observer args are returned");

    let config = paper_watch_observer_nats_config(&args).expect("observer nats config is valid");
    assert!(args.run_paper_watch_observer);
    assert_eq!(args.paper_watch_observer_max_iterations, 0);
    assert_eq!(config.consumer, "research-paper-watch-observer");
    assert_eq!(config.deliver_policy, "new");
    assert!(!config.delete_consumer_after_read);
}

#[test]
fn paper_watch_observer_rejects_one_shot_candidate_key() {
    let err = parse_args(
        [
            "--run-paper-watch-observer",
            "--paper-watch-candidate-s3-bucket",
            "research-bucket",
            "--paper-watch-candidate-s3-key",
            "paper-watch-candidate/schema=paper_watch_candidate_v1/part-000001.jsonl",
            "--market-live-nats-url",
            "nats://127.0.0.1:4222",
            "--output-s3-bucket",
            "research-bucket",
        ]
        .into_iter()
        .map(str::to_owned),
    )
    .expect_err("observer should reject one-shot candidate keys");

    assert!(
        err.to_string()
            .contains("--run-paper-watch-observer uses --paper-watch-candidate-s3-prefix")
    );
}

#[tokio::test]
async fn paper_watch_observer_writes_local_marks_and_snapshot() {
    let root = test_root("paper-watch-observer-local-output");
    let args = Args {
        output_dir: Some(root.clone()),
        output_s3_bucket: None,
        output_s3_prefix: Some(
            "paper-watch-observer-state/schema=paper_watch_observer_snapshot_v1".to_owned(),
        ),
        ..default_args()
    };
    let candidates = serde_json::from_value::<Vec<crate::model::PaperWatchCandidate>>(json!([
        paper_watch_candidate_json("watch_xrp", "XRP")
    ]))
    .expect("paper watch candidates parse");
    let ticks = serde_json::from_value::<Vec<crate::model::MarketLiveTick>>(json!([
        market_live_tick_json("tick_001", "XRP", 2_000, 1.0),
        market_live_tick_json("tick_002", "XRP", 3_000, 1.01)
    ]))
    .expect("market live ticks parse");
    let marks = crate::paper_live::build_paper_watch_live_marks(&candidates, &ticks);

    let mark_files = write_paper_watch_observer_live_marks(&args, &marks, 3_600_000)
        .await
        .expect("observer local marks are written");
    let snapshot_file = write_paper_watch_observer_snapshot(
        &args,
        &json!({
            "schema_version": "paper_watch_observer_snapshot_v1",
            "active_candidate_count": 1,
            "safety": {
                "paper_only": true,
                "live_enabled": false,
                "order_execution_enabled": false,
                "execution_approval_emitted": false
            }
        }),
        3_600_000,
    )
    .await
    .expect("observer local snapshot is written");

    assert_eq!(mark_files.len(), 1);
    let mark_bytes = fs::read(&mark_files[0]).expect("mark file exists");
    let parsed_marks =
        crate::io::read_paper_watch_live_marks_from_bytes("observer-test", &mark_bytes)
            .expect("paper watch live marks parse");
    assert_eq!(parsed_marks.len(), 2);
    assert!(snapshot_file.contains("paper-watch-observer-state/"));
    let snapshot: Value =
        serde_json::from_slice(&fs::read(snapshot_file).expect("snapshot file exists"))
            .expect("snapshot json parses");
    assert_eq!(snapshot["safety"]["order_execution_enabled"], json!(false));
}

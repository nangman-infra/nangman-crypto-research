use super::*;
use crate::time::now_ms;
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

pub(super) const DAY_MS: i64 = 24 * 60 * 60 * 1000;

pub(super) fn test_root(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "research-app-{name}-{}-{}",
        std::process::id(),
        now_ms()
    ))
}

pub(super) fn write_json(path: &Path, value: &Value) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("test parent directory is created");
    }
    fs::write(
        path,
        serde_json::to_vec_pretty(value).expect("test json serializes"),
    )
    .expect("test json is written");
}

pub(super) fn output_file_containing(summary: &RunSummary, needle: &str) -> PathBuf {
    summary
        .output_files
        .iter()
        .find(|path| path.contains(needle))
        .map(PathBuf::from)
        .unwrap_or_else(|| panic!("expected output file containing {needle}"))
}

pub(super) fn default_args() -> Args {
    Args {
        build_shadow_cycle_decision: false,
        run_shadow_cycle_from_latest_state: false,
        build_retest_horizon_plan: false,
        run_retest_refresh_cycle: false,
        run_retest_refresh_cycle_from_latest_state: false,
        run_retest_cycle_scheduler: false,
        build_retest_horizon_status: false,
        build_focused_retest_manifest: false,
        run_paper_watch_live_cycle: false,
        run_paper_watch_observer: false,
        shadow_cycle_decision_file: None,
        shadow_cycle_decision_output_file: None,
        shadow_cycle_latest_l1_as_of_ms: None,
        retest_horizon_plan_file: None,
        retest_horizon_plan_s3_bucket: None,
        retest_horizon_plan_s3_key: None,
        retest_horizon_plan_output_file: None,
        retest_horizon_latest_l1_as_of_ms: None,
        retest_horizon_status_output_file: None,
        retest_driver_summary_file: None,
        retest_horizon_status_file: None,
        retest_horizon_status_s3_bucket: None,
        retest_horizon_status_s3_key: None,
        focused_retest_manifest_output_file: None,
        focused_retest_summary_output_file: None,
        focused_retest_next_actions: crate::focused_retest::default_focused_retest_actions(),
        focused_retest_historical_replay_index_ref_mode:
            crate::focused_retest::HistoricalReplayIndexRefMode::Auto,
        input_manifest_file: None,
        input_manifest_s3_bucket: None,
        input_manifest_s3_key: None,
        research_report_file: None,
        research_report_s3_bucket: None,
        research_report_s3_key: None,
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
        paper_watch_candidate_file: None,
        paper_watch_candidate_s3_bucket: None,
        paper_watch_candidate_s3_key: None,
        paper_watch_candidate_s3_prefix: DEFAULT_PAPER_WATCH_CANDIDATE_PREFIX.to_owned(),
        paper_watch_observer_read_limit: DEFAULT_PAPER_WATCH_OBSERVER_READ_LIMIT,
        paper_watch_observer_scan_limit: DEFAULT_PAPER_WATCH_OBSERVER_SCAN_LIMIT,
        paper_watch_observer_poll_secs: DEFAULT_PAPER_WATCH_OBSERVER_POLL_SECS,
        paper_watch_observer_max_iterations: 0,
        paper_watch_live_mark_s3_prefix: DEFAULT_PAPER_WATCH_LIVE_MARK_PREFIX.to_owned(),
        paper_watch_live_mark_read_limit: DEFAULT_PAPER_WATCH_OBSERVER_READ_LIMIT,
        paper_watch_live_mark_scan_limit: DEFAULT_PAPER_WATCH_OBSERVER_SCAN_LIMIT,
        market_live_tick_file: None,
        market_live_nats_url: None,
        market_live_nats_stream: crate::paper_live::DEFAULT_MARKET_LIVE_NATS_STREAM.to_owned(),
        market_live_nats_subject: crate::paper_live::DEFAULT_MARKET_LIVE_NATS_SUBJECT.to_owned(),
        market_live_nats_consumer: crate::paper_live::DEFAULT_MARKET_LIVE_NATS_CONSUMER.to_owned(),
        market_live_nats_deliver_policy: crate::paper_live::DEFAULT_MARKET_LIVE_NATS_DELIVER_POLICY
            .to_owned(),
        market_live_nats_batch_size: crate::paper_live::DEFAULT_MARKET_LIVE_NATS_BATCH_SIZE,
        market_live_nats_max_messages: crate::paper_live::DEFAULT_MARKET_LIVE_NATS_MAX_MESSAGES,
        market_live_nats_ack_wait_secs: crate::paper_live::DEFAULT_MARKET_LIVE_NATS_ACK_WAIT_SECS,
        historical_replay_run_s3_bucket: None,
        historical_replay_run_s3_keys: Vec::new(),
        historical_replay_run_index_s3_bucket: None,
        historical_replay_run_index_s3_keys: Vec::new(),
        output_dir: None,
        output_s3_bucket: None,
        output_s3_prefix: None,
        research_packet_id: "test_packet".to_owned(),
        run_scope: "test_scope".to_owned(),
        now_ms: None,
    }
}

pub(super) fn bundle_json() -> Value {
    json!({
        "candidate_id": "cand_001",
        "candidate_lifecycle_key": "cand_001:v1",
        "bundle_key": "candidate-evidence-bundle/priority=p0/schema=intel_candidate_evidence_bundle_v1/part-000001.jsonl",
        "producer_app": "intel-candidate-app",
        "producer_run_id": "run_001",
        "created_at_ms": 7_200_000,
        "event_time_ms": 1_000,
        "published_at_ms": 1_000,
        "fetched_at_ms": 1_100,
        "structured_at_ms": 1_200,
        "candidate_created_at_ms": 1_300,
        "decision_available_at_ms": 1_300,
        "forbidden_lookahead_boundary_ms": 1_300,
        "schema_version": "intel_candidate_evidence_bundle_v1",
        "scoring_policy_version": "scoring-policy.v1",
        "normalized_symbols": ["SUI"],
        "symbol_universe_snapshot_id": "universe_001",
        "universe_as_of_ms": 1_200,
        "approved_universe_symbol": true,
        "event_types": ["project_notice"],
        "hypothesis_type": "event_reaction",
        "allowed_horizons": ["1h"],
        "source_story_cluster_ids": ["cluster_001"],
        "source_structured_packet_ids": ["packet_001"],
        "source_context_flag_packet_ids": [],
        "evidence_refs": ["packet_001"],
        "metric_evidence": [],
        "data_quality_summary": {
            "market_data_quality_summary_key": "market_data_quality_summary/run/summary.json",
            "status": "available"
        },
        "selected_market_artifacts": [],
        "candidate_class": "research_candidate",
        "candidate_score": 72,
        "research_priority": "p0",
        "research_eligible": true,
        "validation_requirements": {
            "required_adapters": ["native_replay"],
            "optional_adapters": ["freqtrade_style"],
            "min_unseen_windows": 1,
            "include_fee": true,
            "include_slippage": true,
            "include_latency_assumption": true,
            "include_liquidity_filter": true,
            "required_train_validation_split": true,
            "max_adapter_runtime_minutes": 15
        },
        "source_independence": {
            "source_event_count": 1,
            "independent_source_count": 1,
            "official_source_present": true,
            "duplicate_content_hashes": [],
            "original_source_ids": ["official"]
        },
        "symbol_resolution_trace": [{
            "raw_mentions": ["SUI"],
            "resolved_project": "Sui",
            "resolved_asset": "SUI",
            "canonical_symbol": "SUI",
            "venue_symbols": ["SUIUSDT"],
            "mapping_confidence": "strong"
        }],
        "confidence_summary": {},
        "observe_or_reject_reasons": [],
        "parent_artifact_ids": ["packet_001"],
        "storage_uri": "s3://bucket/key",
        "checksum": "checksum",
        "idempotency_key": "idem_001"
    })
}

pub(super) fn market_live_tick_json(
    event_id: &str,
    symbol: &str,
    timestamp_ms: i64,
    mark_price: f64,
) -> Value {
    json!({
        "schema_version": "market_live_tick_v1",
        "event_id": event_id,
        "producer_run_id": "market_run_001",
        "venue": "binance",
        "source_role": "reference",
        "market_type": "spot",
        "event_type": "trade",
        "symbol_native": format!("{symbol}USDT"),
        "symbol_canonical": symbol,
        "base_asset": symbol,
        "quote_asset": "USDT",
        "exchange_timestamp_ms": timestamp_ms,
        "ingest_timestamp_ms": timestamp_ms + 10,
        "latency_ms": 10,
        "sequence_id": event_id,
        "sequence_tag": "trade_id",
        "price_source": "last_price",
        "last_price": mark_price,
        "mark_price": mark_price,
        "quantity": 1.0,
        "raw_payload_sha256": "sha256:test"
    })
}

pub(super) fn paper_watch_candidate_json(id: &str, symbol: &str) -> Value {
    json!({
        "paper_watch_candidate_id": id,
        "candidate_id": format!("cand_{id}"),
        "candidate_lifecycle_key": format!("cand_{id}:v1"),
        "symbol_canonical": symbol,
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
    })
}

pub(super) fn bundle_json_with_gate_inputs(index: usize, decision_ms: i64) -> Value {
    let mut bundle = bundle_json();
    bundle["candidate_id"] = json!(format!("cand_{index:03}"));
    bundle["candidate_lifecycle_key"] = json!(format!("cand_{index:03}:v1"));
    bundle["idempotency_key"] = json!(format!("idem_{index:03}"));
    bundle["created_at_ms"] = json!(decision_ms);
    bundle["event_time_ms"] = json!(decision_ms - 300);
    bundle["published_at_ms"] = json!(decision_ms - 250);
    bundle["fetched_at_ms"] = json!(decision_ms - 200);
    bundle["structured_at_ms"] = json!(decision_ms - 100);
    bundle["candidate_created_at_ms"] = json!(decision_ms);
    bundle["decision_available_at_ms"] = json!(decision_ms);
    bundle["forbidden_lookahead_boundary_ms"] = json!(decision_ms);
    bundle["universe_as_of_ms"] = json!(decision_ms - 100);
    bundle["validation_requirements"]["min_unseen_windows"] = json!(1);
    bundle["validation_requirements"]["include_liquidity_filter"] = json!(false);
    bundle["validation_requirements"]["required_train_validation_split"] = json!(true);
    bundle
}

pub(super) fn market_delta_json(
    feature_delta_id: &str,
    window_start_ms: i64,
    window_end_ms: i64,
    price_change_same_window: f64,
) -> Value {
    json!({
        "schema_version": "market_feature_delta_v1",
        "feature_delta_id": feature_delta_id,
        "l1_run_id": "l1_001",
        "metric_name": "price",
        "venue": "binance",
        "symbol_native": "SUIUSDT",
        "symbol_canonical": "SUI",
        "market_type": "spot",
        "value_now": 1.0,
        "price_change_same_window": price_change_same_window,
        "window_start_ms": window_start_ms,
        "window_end_ms": window_end_ms,
        "known_as_of_ms": window_end_ms + 100,
        "quality_status": "available",
        "missing_reasons": []
    })
}

pub(super) fn retarget_bundle_symbol(bundle: &mut Value, symbol: &str) {
    bundle["normalized_symbols"] = json!([symbol]);
    bundle["symbol_resolution_trace"][0]["raw_mentions"] = json!([symbol]);
    bundle["symbol_resolution_trace"][0]["resolved_project"] = json!(symbol);
    bundle["symbol_resolution_trace"][0]["resolved_asset"] = json!(symbol);
    bundle["symbol_resolution_trace"][0]["canonical_symbol"] = json!(symbol);
    bundle["symbol_resolution_trace"][0]["venue_symbols"] = json!([format!("{symbol}USDT")]);
}

pub(super) fn retarget_market_delta_symbol(delta: &mut Value, symbol: &str) {
    delta["symbol_native"] = json!(format!("{symbol}USDT"));
    delta["symbol_canonical"] = json!(symbol);
}

pub(super) fn market_liquidity_delta_json(
    feature_delta_id: &str,
    window_start_ms: i64,
    window_end_ms: i64,
) -> Value {
    market_liquidity_delta_json_with_value(
        feature_delta_id,
        window_start_ms,
        window_end_ms,
        10_000.0,
    )
}

pub(super) fn market_liquidity_delta_json_with_value(
    feature_delta_id: &str,
    window_start_ms: i64,
    window_end_ms: i64,
    value_now: f64,
) -> Value {
    json!({
        "schema_version": "market_feature_delta_v1",
        "feature_delta_id": feature_delta_id,
        "l1_run_id": "l1_001",
        "metric_name": "trade_volume",
        "venue": "binance",
        "symbol_native": "SUIUSDT",
        "symbol_canonical": "SUI",
        "market_type": "spot",
        "value_now": value_now,
        "volume_change_same_window": 0.12,
        "window_start_ms": window_start_ms,
        "window_end_ms": window_end_ms,
        "known_as_of_ms": window_end_ms + 100,
        "quality_status": "available",
        "missing_reasons": []
    })
}

pub(super) fn market_regime_json(
    regime_context_id: &str,
    window_start_ms: i64,
    window_end_ms: i64,
) -> Value {
    json!({
        "schema_version": "market_regime_context_v1",
        "regime_context_id": regime_context_id,
        "l1_run_id": "l1_001",
        "scope": "market",
        "window_start_ms": window_start_ms,
        "window_end_ms": window_end_ms,
        "btc_return_same_window": 0.0,
        "eth_return_same_window": 0.0,
        "sector_return_same_window": 0.0,
        "volatility_regime": "low_volatility",
        "correlation_to_btc": 0.2,
        "known_as_of_ms": window_end_ms + 100,
        "quality_status": "available",
        "missing_reasons": []
    })
}

#[test]
pub(super) fn market_delta_symbol_filter_keeps_only_candidate_symbols() {
    let sui = market_delta_json("delta_sui", 1_300, 3_601_300, 0.5);
    let mut btc = market_delta_json("delta_btc", 1_300, 3_601_300, 0.5);
    btc["symbol_native"] = json!("BTCUSDT");
    btc["symbol_canonical"] = json!("BTC");
    let bytes = serde_json::to_vec(&json!([sui, btc])).expect("test deltas serialize");
    let symbols = BTreeSet::from(["SUI".to_owned()]);

    let deltas = crate::io::read_market_feature_deltas_matching_symbols_from_bytes(
        "test-market-deltas",
        &bytes,
        &symbols,
    )
    .expect("filtered deltas parse");

    assert_eq!(deltas.len(), 1);
    assert_eq!(deltas[0].symbol_canonical, "SUI");
}

pub(super) fn oss_adapter_run_json(candidate_lifecycle_key: &str, verdict: &str) -> Value {
    json!({
        "schema_version": "oss_adapter_run_v1",
        "oss_adapter_run_id": format!("oss_run_{candidate_lifecycle_key}_{verdict}"),
        "adapter_name": "vectorbt",
        "adapter_version": "test",
        "candidate_lifecycle_key": candidate_lifecycle_key,
        "input_artifact_refs": ["candidate_bundle"],
        "market_window": "test_window",
        "fee_model_used": "test_fee",
        "slippage_model_used": "test_slippage",
        "trade_count": 3,
        "net_return_bps": 12.0,
        "max_drawdown_bps": 5.0,
        "profit_factor": 1.5,
        "sharpe_like_score": 0.7,
        "lookahead_check_result": "passed",
        "holding_horizon_check_result": "passed",
        "adapter_warnings": [],
        "normalized_verdict_bias": verdict
    })
}

pub(super) fn shadow_cycle_wait_decision_json() -> Value {
    json!({
        "schema_version": "research_shadow_cycle_decision_v1",
        "generated_at": "2026-05-24T12:16:00Z",
        "decision_id": "shadow_cycle_decision:run:WAIT_FOR_PENDING_SHADOW_TARGET_WINDOW_MATERIALIZATION:1779670979756",
        "source_cycle_summary_file": "/tmp/run/shadow-sample-accumulation-cycle-summary.json",
        "run_dir": "/tmp/run",
        "scheduler_action": "WAIT_UNTIL_PENDING_SHADOW_TARGET_WINDOW_MATERIALIZES",
        "source_verdict": "WAIT_FOR_PENDING_SHADOW_TARGET_WINDOW_MATERIALIZATION",
        "run_not_before_ms": 1_779_670_979_756_i64,
        "run_not_before_at": "2026-05-25T01:02:59Z",
        "run_not_before_source": "pending_shadow_target_exit_deadline_ms",
        "focused_research_manifest_file": null,
        "focused_research_summary_file": null,
        "latest_l1_as_of_ms": null,
        "shadow_sample_state": {
            "shadow_validation_count": 24,
            "target_window_materialized_count": 12,
            "candidate_lifecycle_count": 6,
            "partially_materialized_candidate_count": 6,
            "pending_target_window_candidate_count": 6,
            "total_sample_deficit": 168,
            "symbols": ["BTC", "DOGE", "ETH", "SOL", "TON", "ZEC"]
        },
        "safe_next_actions": ["wait_for_pending_shadow_target_window_materialization"],
        "blocked_actions": [
            "do_not_mark_pending_shadow_passed_from_sample_counts_only",
            "do_not_create_paper_without_completed_passed_shadow",
            "do_not_enable_live_from_shadow_sample_gap_manifest"
        ],
        "safety": {
            "s3_write": false,
            "ecs_task_started": false,
            "dispatcher_mode_changed": false,
            "local_decision_only": true,
            "shadow_status_mutated": false,
            "paper_live_enabled": false,
            "live_enabled": false,
            "order_execution_enabled": false
        }
    })
}

pub(super) fn retest_horizon_wait_status_json() -> Value {
    json!({
        "schema_version": "research_horizon_status_checkpoint_v1",
        "generated_at": "2026-05-25T12:09:40Z",
        "safety": {
            "s3_write": false,
            "ecs_task_started": false,
            "dispatcher_mode_changed": false,
            "local_summary_only": true,
            "shadow_paper_live_enabled": false
        },
        "stage_state": {
            "candidate_generated": true,
            "research_replay_completed": true,
            "promotion_passed": false,
            "shadow_created": false,
            "paper_created": false,
            "live_enabled": false
        },
        "next_decision": {
            "verdict": "WAIT_FOR_MARKET_L1_HORIZON",
            "scheduler_hint": {
                "latest_l1_as_of_ms": 1_779_710_400_000_i64,
                "latest_l1_as_of_iso": "2026-05-25T12:00:00Z",
                "run_research_after_l1_as_of_ms": 1_779_719_361_452_i64,
                "run_research_after_l1_as_of_iso": "2026-05-25T14:29:21Z",
                "wait_deficit_ms": 8_961_452,
                "run_now_replay_ready": false,
                "promotion_ready_for_review": false
            },
            "blocked_actions": [
                "do_not_create_shadow_without_promotion",
                "do_not_create_paper_without_passed_shadow",
                "do_not_enable_live_from_research_batch"
            ]
        }
    })
}

pub(super) fn focused_retest_status_json() -> Value {
    let mut status = retest_horizon_wait_status_json();
    status["by_symbol"] = json!([
        {
            "symbol": "AAVE",
            "candidates": [
                {
                    "candidate_id": "cand_focus",
                    "candidate_lifecycle_key": "cand_focus:v1",
                    "hypothesis_type": "event_reaction",
                    "research_priority": "p0",
                    "horizons": [
                        {
                            "horizon": "1h",
                            "next_action": "accumulate_completed_native_replay_samples",
                            "symbols": ["AAVE"],
                            "replay_run_count": 2,
                            "completed_count": 1,
                            "completed_sample_deficit": 2,
                            "reason_codes": ["sample_deficit"]
                        }
                    ]
                },
                {
                    "candidate_id": "cand_wait",
                    "candidate_lifecycle_key": "cand_wait:v1",
                    "hypothesis_type": "event_reaction",
                    "research_priority": "p0",
                    "horizons": [
                        {
                            "horizon": "4h",
                            "next_action": "wait_for_market_l1_horizon",
                            "symbols": ["AAVE"],
                            "reason_codes": ["waiting_for_l1"]
                        }
                    ]
                }
            ]
        }
    ]);
    status
}

pub(super) fn retest_horizon_plan_json() -> Value {
    json!({
        "schema_version": "research_retest_horizon_plan_v1",
        "latest_l1_as_of_ms": 1_779_710_400_000_i64,
        "horizon_rows": [
            {
                "candidate_id": "cand_focus",
                "candidate_lifecycle_key": "cand_focus:v1",
                "symbols": ["AAVE"],
                "primary_symbol": "AAVE",
                "hypothesis_type": "event_reaction",
                "research_priority": "p0",
                "horizon": "1h",
                "horizon_due_ms": 1_779_710_300_000_i64,
                "horizon_market_data_materialized": true,
                "replay_run_count": 2,
                "completed_count": 1,
                "completed_sample_deficit": 29,
                "inferred_unseen_window_count": 1,
                "unseen_window_deficit": 19,
                "reason_codes": ["sample_deficit"],
                "next_action": "accumulate_completed_native_replay_samples"
            },
            {
                "candidate_id": "cand_wait",
                "candidate_lifecycle_key": "cand_wait:v1",
                "symbols": ["AAVE"],
                "primary_symbol": "AAVE",
                "hypothesis_type": "event_reaction",
                "research_priority": "p0",
                "horizon": "4h",
                "horizon_due_ms": 1_779_719_361_452_i64,
                "horizon_market_data_materialized": false,
                "replay_run_count": 0,
                "completed_count": 0,
                "completed_sample_deficit": 30,
                "inferred_unseen_window_count": 0,
                "unseen_window_deficit": 20,
                "reason_codes": ["waiting_for_l1"],
                "next_action": "wait_for_market_l1_horizon"
            }
        ]
    })
}

pub(super) fn focused_retest_run_now_status_json() -> Value {
    let mut status = focused_retest_status_json();
    status["next_decision"]["verdict"] = json!("REPLAY_READY_FOR_SOME_HORIZONS");
    status["next_decision"]["scheduler_hint"]["run_now_replay_ready"] = json!(true);
    status["next_decision"]["scheduler_hint"]["run_research_after_l1_as_of_ms"] = Value::Null;
    status["next_decision"]["scheduler_hint"]["run_research_after_l1_as_of_iso"] = Value::Null;
    status["next_decision"]["scheduler_hint"]["wait_deficit_ms"] = Value::Null;
    status
}

pub(super) fn focused_retest_source_manifest_json() -> Value {
    json!({
        "schema_version": "research_input_manifest_v1",
        "research_packet_id": "source_packet",
        "run_scope": "current_approved",
        "candidate_bundle_refs": [
            {
                "uri": "s3://bucket/candidate-evidence-bundle/priority=p0/candidate_id=cand_focus/part-000001.jsonl"
            },
            {
                "uri": "s3://bucket/candidate-evidence-bundle/priority=p0/candidate_id=cand_wait/part-000001.jsonl"
            }
        ],
        "historical_replay_run_index_refs": [
            {
                "uri": "s3://research/replay-run-index/part-000001.jsonl"
            }
        ],
        "runtime_budget_policy": {
            "max_candidate_bundle_count": 10,
            "max_market_artifact_ref_count": 20,
            "max_shadow_validation_run_ref_count": 20,
            "max_hypothesis_harness_result_ref_count": 20,
            "max_oss_adapter_run_ref_count": 20,
            "max_historical_replay_run_ref_count": 20,
            "max_replay_run_count": 100
        }
    })
}

pub(super) fn shadow_validation_run_json(
    shadow_validation_run_id: &str,
    candidate_lifecycle_key: &str,
    symbol: &str,
    decision_available_ms: i64,
    min_shadow_samples: usize,
) -> Value {
    json!({
        "schema_version": "shadow_validation_run_v1",
        "shadow_validation_run_id": shadow_validation_run_id,
        "candidate_lifecycle_key": candidate_lifecycle_key,
        "symbol_canonical": symbol,
        "trigger_research_run_id": "research_report_test",
        "start_condition_summary": {
            "research_aggregate_key": "aggregate_test",
            "gate_policy_version": "test_gate_policy",
            "completed_count": 30,
            "mean_net_after_cost_bps": 12.0,
            "win_rate_ppm": 600000,
            "profit_factor_ppm": 1200000,
            "gate_reason_codes": ["deterministic_shadow_gate_passed"]
        },
        "expected_survival_band": "stable",
        "watch_window_policy": {
            "mode": "forward_observation_only",
            "min_shadow_samples": min_shadow_samples,
            "max_shadow_age_days": 30
        },
        "termination_policy": {
            "prune_on_non_positive_mean_net": true,
            "prune_on_max_age_without_samples": true,
            "no_order_execution": true
        },
        "holding_policy": {
            "target_max_holding_hours": 24,
            "absolute_max_holding_hours": 72,
            "absolute_exit_deadline_ms": decision_available_ms + (72_i64 * 60 * 60 * 1000),
            "force_flat_policy": "daily_or_ttl_exit",
            "overnight_risk_exception": false,
            "holding_policy_version": "crypto_intraday_holding_policy_v1_2026_05_12"
        },
        "status": "pending",
        "passed": false,
        "paper_trade_candidate_contract_version": "paper_trade_candidate_v1"
    })
}

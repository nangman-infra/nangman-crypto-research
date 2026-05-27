use super::*;
use crate::model::ShadowCycleSchedulerAction;
use crate::time::now_ms;
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

const DAY_MS: i64 = 24 * 60 * 60 * 1000;

fn test_root(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "research-app-{name}-{}-{}",
        std::process::id(),
        now_ms()
    ))
}

fn write_json(path: &Path, value: &Value) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("test parent directory is created");
    }
    fs::write(
        path,
        serde_json::to_vec_pretty(value).expect("test json serializes"),
    )
    .expect("test json is written");
}

fn output_file_containing(summary: &RunSummary, needle: &str) -> PathBuf {
    summary
        .output_files
        .iter()
        .find(|path| path.contains(needle))
        .map(PathBuf::from)
        .unwrap_or_else(|| panic!("expected output file containing {needle}"))
}

fn default_args() -> Args {
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

fn bundle_json() -> Value {
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

fn market_live_tick_json(
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

fn paper_watch_candidate_json(id: &str, symbol: &str) -> Value {
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

fn bundle_json_with_gate_inputs(index: usize, decision_ms: i64) -> Value {
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

fn market_delta_json(
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

fn retarget_bundle_symbol(bundle: &mut Value, symbol: &str) {
    bundle["normalized_symbols"] = json!([symbol]);
    bundle["symbol_resolution_trace"][0]["raw_mentions"] = json!([symbol]);
    bundle["symbol_resolution_trace"][0]["resolved_project"] = json!(symbol);
    bundle["symbol_resolution_trace"][0]["resolved_asset"] = json!(symbol);
    bundle["symbol_resolution_trace"][0]["canonical_symbol"] = json!(symbol);
    bundle["symbol_resolution_trace"][0]["venue_symbols"] = json!([format!("{symbol}USDT")]);
}

fn retarget_market_delta_symbol(delta: &mut Value, symbol: &str) {
    delta["symbol_native"] = json!(format!("{symbol}USDT"));
    delta["symbol_canonical"] = json!(symbol);
}

fn market_liquidity_delta_json(
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

fn market_liquidity_delta_json_with_value(
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

fn market_regime_json(regime_context_id: &str, window_start_ms: i64, window_end_ms: i64) -> Value {
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
fn market_delta_symbol_filter_keeps_only_candidate_symbols() {
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

fn oss_adapter_run_json(candidate_lifecycle_key: &str, verdict: &str) -> Value {
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

fn shadow_cycle_wait_decision_json() -> Value {
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

fn retest_horizon_wait_status_json() -> Value {
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

fn focused_retest_status_json() -> Value {
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

fn retest_horizon_plan_json() -> Value {
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

fn focused_retest_run_now_status_json() -> Value {
    let mut status = focused_retest_status_json();
    status["next_decision"]["verdict"] = json!("REPLAY_READY_FOR_SOME_HORIZONS");
    status["next_decision"]["scheduler_hint"]["run_now_replay_ready"] = json!(true);
    status["next_decision"]["scheduler_hint"]["run_research_after_l1_as_of_ms"] = Value::Null;
    status["next_decision"]["scheduler_hint"]["run_research_after_l1_as_of_iso"] = Value::Null;
    status["next_decision"]["scheduler_hint"]["wait_deficit_ms"] = Value::Null;
    status
}

fn focused_retest_source_manifest_json() -> Value {
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

fn shadow_validation_run_json(
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

#[test]
fn parse_args_requires_absolute_input_path() {
    let error = parse_args(
        [
            "--input-bundle-file".to_owned(),
            "relative.jsonl".to_owned(),
        ]
        .into_iter(),
    )
    .expect_err("relative path should fail");
    assert!(error.to_string().contains("absolute path"));
}

#[test]
fn parse_args_requires_absolute_shadow_cycle_decision_path() {
    let error = parse_args(
        [
            "--shadow-cycle-decision-file".to_owned(),
            "relative.json".to_owned(),
        ]
        .into_iter(),
    )
    .expect_err("relative path should fail");
    assert!(error.to_string().contains("absolute path"));
}

#[test]
fn parse_args_requires_absolute_retest_horizon_status_path() {
    let error = parse_args(
        [
            "--retest-horizon-status-file".to_owned(),
            "relative.json".to_owned(),
        ]
        .into_iter(),
    )
    .expect_err("relative path should fail");
    assert!(error.to_string().contains("absolute path"));
}

#[tokio::test]
async fn build_shadow_cycle_decision_from_shadow_runs() {
    let root = test_root("shadow-decision-build-cli");
    let shadow_file = root.join("shadow-runs.json");
    let output_file = root.join("shadow-cycle-decision.json");
    let decision_ms = 1_780_000_000_000_i64;
    let materialized_target_ms = decision_ms + DAY_MS;
    let later_decision_ms = decision_ms + 2 * 60 * 60 * 1000;
    write_json(
        &shadow_file,
        &json!([
            shadow_validation_run_json("shadow_a", "cand_a", "XAUT", decision_ms, 30),
            shadow_validation_run_json("shadow_b", "cand_b", "CHIP", later_decision_ms, 30)
        ]),
    );

    let args = parse_args(
        [
            "--build-shadow-cycle-decision".to_owned(),
            "--shadow-validation-run-file".to_owned(),
            shadow_file.display().to_string(),
            "--shadow-cycle-latest-l1-as-of-ms".to_owned(),
            materialized_target_ms.to_string(),
            "--shadow-cycle-decision-output-file".to_owned(),
            output_file.display().to_string(),
            "--now-ms".to_owned(),
            "1780100000000".to_owned(),
        ]
        .into_iter(),
    )
    .expect("build args parse")
    .expect("build args returned");

    let summary = run(args).await.expect("shadow cycle decision builds");
    assert_eq!(summary.shadow_cycle_decisions_created, 1);
    assert_eq!(summary.shadow_cycle_decisions_validated, 1);
    assert_eq!(
        summary.shadow_cycle_scheduler_action,
        Some(ShadowCycleSchedulerAction::WaitUntilTargetWindowMaterializes)
    );
    assert_eq!(summary.shadow_validation_runs_loaded, 2);
    assert_eq!(
        summary.output_files,
        vec![output_file.display().to_string()]
    );

    let decision: Value = serde_json::from_slice(
        &fs::read(&output_file).expect("shadow cycle decision file is written"),
    )
    .expect("shadow cycle decision parses");
    assert_eq!(
        decision["source_verdict"],
        json!("WAIT_FOR_TARGET_HOLDING_WINDOW")
    );
    assert_eq!(
        decision["shadow_sample_state"]["target_window_materialized_count"],
        json!(1)
    );
    assert_eq!(
        decision["shadow_sample_state"]["pending_target_window_candidate_count"],
        json!(1)
    );
    assert_eq!(decision["safety"]["order_execution_enabled"], json!(false));
}

#[tokio::test]
async fn build_shadow_cycle_decision_writes_partitioned_output_dir() {
    let root = test_root("shadow-decision-build-output-dir");
    let shadow_file = root.join("shadow-runs.json");
    let output_dir = root.join("outputs");
    let decision_ms = 1_780_000_000_000_i64;
    let materialized_target_ms = decision_ms + DAY_MS;
    write_json(
        &shadow_file,
        &json!([shadow_validation_run_json(
            "shadow_a",
            "cand_a",
            "XAUT",
            decision_ms,
            30
        )]),
    );

    let args = parse_args(
        [
            "--build-shadow-cycle-decision".to_owned(),
            "--shadow-validation-run-file".to_owned(),
            shadow_file.display().to_string(),
            "--shadow-cycle-latest-l1-as-of-ms".to_owned(),
            materialized_target_ms.to_string(),
            "--output-dir".to_owned(),
            output_dir.display().to_string(),
            "--now-ms".to_owned(),
            "1780100000000".to_owned(),
        ]
        .into_iter(),
    )
    .expect("build args parse")
    .expect("build args returned");

    let summary = run(args).await.expect("shadow cycle decision builds");
    assert_eq!(summary.shadow_cycle_decisions_created, 1);
    assert_eq!(summary.output_files.len(), 1);

    let output_file = PathBuf::from(&summary.output_files[0]);
    assert!(output_file.starts_with(&output_dir));
    assert!(
        output_file
            .display()
            .to_string()
            .contains("shadow-cycle-decision/schema=research_shadow_cycle_decision_v1")
    );
    assert!(output_file.exists());
}

#[test]
fn build_shadow_cycle_decision_requires_output_target() {
    let error = parse_args(
        [
            "--build-shadow-cycle-decision".to_owned(),
            "--shadow-validation-run-file".to_owned(),
            "/tmp/shadow-runs.json".to_owned(),
        ]
        .into_iter(),
    )
    .expect_err("build mode requires an output target");

    assert!(error.to_string().contains("output"));
}

#[test]
fn build_shadow_cycle_decision_rejects_conflicting_decision_modes() {
    let error = parse_args(
        [
            "--build-shadow-cycle-decision".to_owned(),
            "--shadow-cycle-decision-file".to_owned(),
            "/tmp/shadow-cycle-decision.json".to_owned(),
            "--shadow-validation-run-file".to_owned(),
            "/tmp/shadow-runs.json".to_owned(),
            "--shadow-cycle-decision-output-file".to_owned(),
            "/tmp/shadow-cycle-output.json".to_owned(),
        ]
        .into_iter(),
    )
    .expect_err("build mode and decision validation mode are mutually exclusive");

    assert!(error.to_string().contains("separately"));
}

#[test]
fn build_shadow_cycle_decision_requires_numeric_latest_l1() {
    let error = parse_args(
        [
            "--build-shadow-cycle-decision".to_owned(),
            "--shadow-validation-run-file".to_owned(),
            "/tmp/shadow-runs.json".to_owned(),
            "--shadow-cycle-latest-l1-as-of-ms".to_owned(),
            "not-a-number".to_owned(),
            "--shadow-cycle-decision-output-file".to_owned(),
            "/tmp/shadow-cycle-output.json".to_owned(),
        ]
        .into_iter(),
    )
    .expect_err("latest L1 watermark must be numeric");

    assert!(error.to_string().contains("integer"));
}

#[test]
fn build_shadow_cycle_decision_rejects_conflicting_output_targets() {
    let error = parse_args(
        [
            "--build-shadow-cycle-decision".to_owned(),
            "--shadow-validation-run-file".to_owned(),
            "/tmp/shadow-runs.json".to_owned(),
            "--output-dir".to_owned(),
            "/tmp/shadow-cycle-output".to_owned(),
            "--output-s3-bucket".to_owned(),
            "research-bucket".to_owned(),
        ]
        .into_iter(),
    )
    .expect_err("output dir and output bucket are mutually exclusive");

    assert!(error.to_string().contains("output-dir"));
}

#[test]
fn build_shadow_cycle_decision_requires_s3_bucket_for_shadow_key() {
    let error = parse_args(
        [
            "--build-shadow-cycle-decision".to_owned(),
            "--shadow-validation-run-s3-key".to_owned(),
            "shadow-validation-run/part-000001.jsonl".to_owned(),
            "--shadow-cycle-decision-output-file".to_owned(),
            "/tmp/shadow-cycle-output.json".to_owned(),
        ]
        .into_iter(),
    )
    .expect_err("shadow validation S3 key requires bucket");

    assert!(
        error
            .to_string()
            .contains("shadow-validation-run-s3-bucket")
    );
}

#[test]
fn build_shadow_cycle_decision_requires_shadow_input_source() {
    let error = parse_args(
        [
            "--build-shadow-cycle-decision".to_owned(),
            "--shadow-cycle-decision-output-file".to_owned(),
            "/tmp/shadow-cycle-output.json".to_owned(),
        ]
        .into_iter(),
    )
    .expect_err("build mode requires shadow validation inputs");

    assert!(error.to_string().contains("shadow validation run file"));
}

#[test]
fn run_shadow_cycle_from_latest_state_requires_output_bucket() {
    let error = parse_args(["--run-shadow-cycle-from-latest-state".to_owned()].into_iter())
        .expect_err("latest shadow cycle mode requires S3 output bucket");

    assert!(error.to_string().contains("output-s3-bucket"));
}

#[test]
fn run_shadow_cycle_from_latest_state_rejects_explicit_shadow_inputs() {
    let error = parse_args(
        [
            "--run-shadow-cycle-from-latest-state".to_owned(),
            "--output-s3-bucket".to_owned(),
            "research-bucket".to_owned(),
            "--shadow-validation-run-s3-bucket".to_owned(),
            "research-bucket".to_owned(),
            "--shadow-validation-run-s3-key".to_owned(),
            "shadow-validation-run/part-000001.jsonl".to_owned(),
        ]
        .into_iter(),
    )
    .expect_err("latest shadow cycle mode discovers its own shadow inputs");

    assert!(error.to_string().contains("discovers shadow inputs"));
}

#[test]
fn run_shadow_cycle_from_latest_state_parses_with_market_l1_bucket() {
    let args = parse_args(
        [
            "--run-shadow-cycle-from-latest-state".to_owned(),
            "--output-s3-bucket".to_owned(),
            "research-bucket".to_owned(),
            "--market-l1-s3-bucket".to_owned(),
            "market-l1-bucket".to_owned(),
        ]
        .into_iter(),
    )
    .expect("latest shadow cycle args parse")
    .expect("latest shadow cycle args returned");

    assert!(args.run_shadow_cycle_from_latest_state);
    assert_eq!(args.output_s3_bucket.as_deref(), Some("research-bucket"));
    assert_eq!(
        args.market_l1_s3_bucket.as_deref(),
        Some("market-l1-bucket")
    );
}

#[tokio::test]
async fn build_shadow_cycle_decision_rejects_empty_shadow_runs() {
    let root = test_root("shadow-decision-build-empty");
    let shadow_file = root.join("shadow-runs.json");
    let output_file = root.join("shadow-cycle-decision.json");
    write_json(&shadow_file, &json!([]));

    let args = parse_args(
        [
            "--build-shadow-cycle-decision".to_owned(),
            "--shadow-validation-run-file".to_owned(),
            shadow_file.display().to_string(),
            "--shadow-cycle-decision-output-file".to_owned(),
            output_file.display().to_string(),
        ]
        .into_iter(),
    )
    .expect("build args parse")
    .expect("build args returned");

    let error = run(args)
        .await
        .expect_err("empty shadow validation input is rejected");
    assert!(
        error
            .to_string()
            .contains("at least one shadow validation run")
    );
}

#[tokio::test]
async fn shadow_cycle_decision_file_validates_without_research_inputs() {
    let root = test_root("shadow-decision-cli");
    let decision_file = root.join("shadow-cycle-decision.json");
    write_json(&decision_file, &shadow_cycle_wait_decision_json());

    let args = parse_args(
        [
            "--shadow-cycle-decision-file".to_owned(),
            decision_file.display().to_string(),
        ]
        .into_iter(),
    )
    .expect("decision args parse")
    .expect("decision args returned");
    let summary = run(args).await.expect("decision validates");

    assert_eq!(summary.shadow_cycle_decisions_validated, 1);
    assert_eq!(
        summary.shadow_cycle_scheduler_action,
        Some(ShadowCycleSchedulerAction::WaitUntilPendingShadowTargetWindowMaterializes)
    );
    assert_eq!(
        summary.shadow_cycle_run_not_before_ms,
        Some(1_779_670_979_756)
    );
    assert_eq!(summary.shadow_cycle_focused_research_manifest_file, None);
    assert_eq!(summary.processed_bundles, 0);
    assert!(summary.output_files.is_empty());
}

#[tokio::test]
async fn shadow_cycle_decision_file_rejects_order_execution_enabled() {
    let root = test_root("shadow-decision-unsafe-cli");
    let decision_file = root.join("shadow-cycle-decision.json");
    let mut decision = shadow_cycle_wait_decision_json();
    decision["safety"]["order_execution_enabled"] = json!(true);
    write_json(&decision_file, &decision);

    let args = parse_args(
        [
            "--shadow-cycle-decision-file".to_owned(),
            decision_file.display().to_string(),
        ]
        .into_iter(),
    )
    .expect("decision args parse")
    .expect("decision args returned");
    let error = run(args)
        .await
        .expect_err("unsafe shadow cycle decision is rejected");

    assert!(error.to_string().contains("paper/live/order execution"));
}

#[tokio::test]
async fn retest_horizon_status_file_validates_without_research_inputs() {
    let root = test_root("retest-status-cli");
    let status_file = root.join("retest-horizon-status.json");
    write_json(&status_file, &retest_horizon_wait_status_json());

    let args = parse_args(
        [
            "--retest-horizon-status-file".to_owned(),
            status_file.display().to_string(),
        ]
        .into_iter(),
    )
    .expect("status args parse")
    .expect("status args returned");
    let summary = run(args).await.expect("status validates");

    assert_eq!(summary.retest_horizon_statuses_validated, 1);
    assert_eq!(
        summary.retest_cycle_scheduler_action,
        Some("WAIT_UNTIL_MARKET_L1_HORIZON_MATERIALIZES".to_owned())
    );
    assert_eq!(
        summary.retest_cycle_run_not_before_ms,
        Some(1_779_719_361_452)
    );
    assert_eq!(summary.processed_bundles, 0);
    assert!(summary.output_files.is_empty());
}

#[tokio::test]
async fn retest_horizon_status_file_rejects_live_enabled() {
    let root = test_root("retest-status-unsafe-cli");
    let status_file = root.join("retest-horizon-status.json");
    let mut status = retest_horizon_wait_status_json();
    status["stage_state"]["live_enabled"] = json!(true);
    write_json(&status_file, &status);

    let args = parse_args(
        [
            "--retest-horizon-status-file".to_owned(),
            status_file.display().to_string(),
        ]
        .into_iter(),
    )
    .expect("status args parse")
    .expect("status args returned");
    let error = run(args)
        .await
        .expect_err("unsafe retest status is rejected");

    assert!(error.to_string().contains("live trading"));
}

#[tokio::test]
async fn build_retest_horizon_status_from_plan_file() {
    let root = test_root("retest-status-build-cli");
    let plan_file = root.join("retest-horizon-plan.json");
    let output_file = root.join("retest-horizon-status.json");
    write_json(&plan_file, &retest_horizon_plan_json());

    let args = parse_args(
        [
            "--build-retest-horizon-status".to_owned(),
            "--retest-horizon-plan-file".to_owned(),
            plan_file.display().to_string(),
            "--retest-horizon-status-output-file".to_owned(),
            output_file.display().to_string(),
            "--now-ms".to_owned(),
            "1779714000000".to_owned(),
        ]
        .into_iter(),
    )
    .expect("status build args parse")
    .expect("status build args returned");
    let summary = run(args).await.expect("status builds");

    assert_eq!(summary.retest_horizon_statuses_validated, 1);
    assert_eq!(
        summary.retest_cycle_scheduler_action,
        Some("RUN_FOCUSED_RETEST_RESEARCH".to_owned())
    );
    assert_eq!(summary.retest_cycle_run_not_before_ms, None);
    assert_eq!(
        summary.output_files,
        vec![output_file.display().to_string()]
    );

    let status: Value =
        serde_json::from_slice(&fs::read(&output_file).expect("status")).expect("status json");
    assert_eq!(
        status["schema_version"],
        json!("research_horizon_status_checkpoint_v1")
    );
    assert_eq!(status["safety"]["checkpoint_s3_write"], json!(false));
    assert_eq!(status["selected_symbols"], json!(["AAVE"]));
    assert_eq!(
        status["by_symbol"][0]["candidates"][1]["horizons"][0]["next_action"],
        json!("wait_for_market_l1_horizon")
    );
}

#[tokio::test]
async fn build_retest_horizon_plan_from_manifest_and_report() {
    let root = test_root("retest-plan-build-cli");
    let bundle = root.join("bundle.json");
    let delta = root.join("delta.json");
    let regime = root.join("regime.json");
    let manifest = root.join("manifest.json");
    let research_output = root.join("research-out");
    let plan_output = root.join("retest-horizon-plan.json");

    write_json(&bundle, &bundle_json_with_gate_inputs(1, 1_300));
    write_json(
        &delta,
        &json!([market_delta_json("delta_001", 1_300, 3_601_300, 0.021)]),
    );
    write_json(
        &regime,
        &json!([market_regime_json("regime_001", 1_300, 3_601_300)]),
    );
    write_json(
        &manifest,
        &json!({
            "schema_version": "research_input_manifest_v1",
            "research_packet_id": "manifest_packet",
            "run_scope": "manifest_batch",
            "candidate_bundle_refs": [{ "uri": bundle.display().to_string() }],
            "market_feature_delta_refs": [{ "uri": delta.display().to_string() }],
            "market_regime_context_refs": [{ "uri": regime.display().to_string() }],
            "runtime_budget_policy": {
                "max_candidate_bundle_count": 10,
                "max_market_artifact_ref_count": 10,
                "max_historical_replay_run_ref_count": 10,
                "max_replay_run_count": 20
            }
        }),
    );

    let research_summary = run(Args {
        input_manifest_file: Some(manifest.clone()),
        output_dir: Some(research_output),
        now_ms: Some(7_300_000),
        ..default_args()
    })
    .await
    .expect("research report builds");
    let report_file = output_file_containing(&research_summary, "research-run-report");

    let args = parse_args(
        [
            "--build-retest-horizon-plan".to_owned(),
            "--input-manifest-file".to_owned(),
            manifest.display().to_string(),
            "--research-report-file".to_owned(),
            report_file.display().to_string(),
            "--retest-horizon-plan-output-file".to_owned(),
            plan_output.display().to_string(),
            "--retest-horizon-latest-l1-as-of-ms".to_owned(),
            "7201300".to_owned(),
            "--now-ms".to_owned(),
            "7400000".to_owned(),
        ]
        .into_iter(),
    )
    .expect("plan build args parse")
    .expect("plan build args returned");
    let summary = run(args).await.expect("plan builds");

    assert_eq!(summary.retest_horizon_plans_created, 1);
    assert_eq!(
        summary.output_files,
        vec![plan_output.display().to_string()]
    );
    let plan: Value =
        serde_json::from_slice(&fs::read(&plan_output).expect("plan")).expect("plan json");
    assert_eq!(
        plan["schema_version"],
        json!("research_retest_horizon_plan_v1")
    );
    assert_eq!(plan["generated_at_ms"], json!(7_400_000));
    assert_eq!(plan["latest_l1_as_of_ms"], json!(7_201_300));
    assert_eq!(plan["summary"]["candidate_count"], json!(1));
    assert_eq!(plan["summary"]["horizon_count"], json!(1));
    assert_eq!(
        plan["horizon_rows"][0]["next_action"],
        json!("accumulate_completed_native_replay_samples")
    );
}

#[tokio::test]
async fn retest_refresh_cycle_waits_without_writing_focused_manifest() {
    let root = test_root("retest-refresh-wait");
    let (manifest, report_file) = write_refresh_cycle_inputs(&root).await;
    let output = root.join("cycle-out");

    let args = parse_args(
        [
            "--run-retest-refresh-cycle".to_owned(),
            "--input-manifest-file".to_owned(),
            manifest.display().to_string(),
            "--research-report-file".to_owned(),
            report_file.display().to_string(),
            "--retest-horizon-latest-l1-as-of-ms".to_owned(),
            "1000".to_owned(),
            "--output-dir".to_owned(),
            output.display().to_string(),
            "--now-ms".to_owned(),
            "2000".to_owned(),
        ]
        .into_iter(),
    )
    .expect("refresh args parse")
    .expect("refresh args returned");
    let summary = run(args).await.expect("refresh cycle waits");

    assert_eq!(summary.retest_horizon_plans_created, 1);
    assert_eq!(summary.retest_horizon_statuses_validated, 1);
    assert_eq!(
        summary.retest_cycle_scheduler_action,
        Some("WAIT_UNTIL_MARKET_L1_HORIZON_MATERIALIZES".to_owned())
    );
    assert_eq!(summary.focused_retest_manifests_created, 0);
    assert!(output.join("retest-horizon-plan.json").exists());
    assert!(output.join("retest-horizon-status.json").exists());
    assert!(!output.join("research-input-manifest.json").exists());
}

#[tokio::test]
async fn retest_refresh_cycle_writes_focused_manifest_for_accumulation_ready_horizon() {
    let root = test_root("retest-refresh-run");
    let (manifest, report_file) = write_refresh_cycle_inputs(&root).await;
    let output = root.join("cycle-out");

    let args = parse_args(
        [
            "--run-retest-refresh-cycle".to_owned(),
            "--input-manifest-file".to_owned(),
            manifest.display().to_string(),
            "--research-report-file".to_owned(),
            report_file.display().to_string(),
            "--retest-horizon-latest-l1-as-of-ms".to_owned(),
            "7201300".to_owned(),
            "--output-dir".to_owned(),
            output.display().to_string(),
            "--research-packet-id".to_owned(),
            "refresh_cycle_focus".to_owned(),
            "--now-ms".to_owned(),
            "7400000".to_owned(),
        ]
        .into_iter(),
    )
    .expect("refresh args parse")
    .expect("refresh args returned");
    let summary = run(args).await.expect("refresh cycle writes focus");

    assert_eq!(summary.retest_horizon_plans_created, 1);
    assert_eq!(summary.retest_horizon_statuses_validated, 1);
    assert_eq!(
        summary.retest_cycle_scheduler_action,
        Some("RUN_FOCUSED_RETEST_RESEARCH".to_owned())
    );
    assert_eq!(summary.focused_retest_manifests_created, 1);
    assert_eq!(summary.focused_retest_candidate_bundle_refs, 1);
    assert!(output.join("retest-horizon-plan.json").exists());
    assert!(output.join("retest-horizon-status.json").exists());
    assert!(output.join("research-input-manifest.json").exists());
    assert!(output.join("research-input-manifest.summary.json").exists());
}

#[test]
fn focused_retest_dispatch_packet_id_is_stable_for_same_refresh_inputs() {
    let source_manifest: crate::model::ResearchInputManifest =
        serde_json::from_value(focused_retest_source_manifest_json())
            .expect("source manifest parses");
    let status = focused_retest_run_now_status_json();
    let mut args = default_args();
    args.input_manifest_s3_bucket = Some("research-bucket".to_owned());
    args.input_manifest_s3_key = Some(
        "research-input-manifest/schema=research_input_manifest_v1/source/manifest.json".to_owned(),
    );
    args.research_report_s3_bucket = Some("research-bucket".to_owned());
    args.research_report_s3_key =
        Some("research-run-report/schema=research_run_report_v1/report.json".to_owned());
    args.run_scope = "focused_retest_local_validation".to_owned();

    let build_a = crate::focused_retest::build_focused_retest_manifest(
        &status,
        &source_manifest,
        &crate::focused_retest::FocusedRetestBuildOptions {
            generated_at_ms: 7_400_000,
            research_packet_id: "research_focus_7400000".to_owned(),
            run_scope: "focused_retest_local_validation".to_owned(),
            next_actions: crate::focused_retest::default_focused_retest_actions(),
            candidate_lifecycle_key_filter: Vec::new(),
            historical_replay_index_ref_mode:
                crate::focused_retest::HistoricalReplayIndexRefMode::Auto,
            s3_write: true,
        },
    )
    .expect("focused build a succeeds");
    let build_b = crate::focused_retest::build_focused_retest_manifest(
        &status,
        &source_manifest,
        &crate::focused_retest::FocusedRetestBuildOptions {
            generated_at_ms: 7_500_000,
            research_packet_id: "research_focus_7500000".to_owned(),
            run_scope: "focused_retest_local_validation".to_owned(),
            next_actions: crate::focused_retest::default_focused_retest_actions(),
            candidate_lifecycle_key_filter: Vec::new(),
            historical_replay_index_ref_mode:
                crate::focused_retest::HistoricalReplayIndexRefMode::Auto,
            s3_write: true,
        },
    )
    .expect("focused build b succeeds");

    let first_id = focused_retest_dispatch_packet_id(&args, Some(7_201_300), &build_a)
        .expect("first dispatch id");
    let second_id = focused_retest_dispatch_packet_id(&args, Some(7_201_300), &build_b)
        .expect("second dispatch id");
    let advanced_l1_id = focused_retest_dispatch_packet_id(&args, Some(7_801_300), &build_b)
        .expect("advanced l1 dispatch id");

    assert_eq!(first_id, second_id);
    assert_ne!(first_id, advanced_l1_id);
    assert!(first_id.starts_with("research_focus_"));
    assert_eq!(
        focused_retest_dispatch_manifest_s3_key(&first_id)
            .expect("dispatch key")
            .as_str(),
        format!(
            "research-input-manifest/schema=research_input_manifest_v1/dedupe_key={first_id}/manifest.json"
        )
    );
}

#[test]
fn shadow_accumulation_dispatch_filters_manifest_to_deficient_lifecycle_keys() {
    let args = default_args();
    let state = retest_cycle_source_state();
    let source_manifest: crate::model::ResearchInputManifest =
        serde_json::from_value(focused_retest_source_manifest_json())
            .expect("source manifest parses");
    let status = focused_retest_run_now_status_json();

    let dispatch = build_shadow_accumulation_manifest_dispatch(
        &args,
        &state,
        &status,
        &source_manifest,
        Some(7_201_300),
        7_400_000,
        vec!["cand_focus:v1".to_owned(), "missing:v1".to_owned()],
    )
    .expect("shadow accumulation dispatch builds")
    .expect("shadow accumulation dispatch is selected");

    assert!(dispatch.key.starts_with(
        "research-input-manifest/schema=research_input_manifest_v1/dedupe_key=research_shadow_accumulation_"
    ));
    assert_eq!(
        dispatch.manifest.run_scope.as_deref(),
        Some("shadow_sample_accumulation_local_validation")
    );
    assert_eq!(dispatch.manifest.candidate_bundle_refs.len(), 1);
    assert!(
        dispatch.manifest.candidate_bundle_refs[0]
            .uri
            .contains("candidate_id=cand_focus")
    );
    assert_eq!(dispatch.manifest.historical_replay_run_index_refs.len(), 1);
    assert_eq!(dispatch.focused_horizon_count, 1);
    assert_eq!(dispatch.focused_candidate_bundle_refs, 1);
    assert_eq!(
        dispatch.deficit_lifecycle_keys,
        vec!["cand_focus:v1".to_owned(), "missing:v1".to_owned()]
    );
}

#[test]
fn shadow_accumulation_dispatch_skips_empty_deficit_keys() {
    let args = default_args();
    let state = retest_cycle_source_state();
    let source_manifest: crate::model::ResearchInputManifest =
        serde_json::from_value(focused_retest_source_manifest_json())
            .expect("source manifest parses");
    let status = focused_retest_run_now_status_json();

    let dispatch = build_shadow_accumulation_manifest_dispatch(
        &args,
        &state,
        &status,
        &source_manifest,
        Some(7_201_300),
        7_400_000,
        Vec::new(),
    )
    .expect("empty deficit keys are valid");

    assert!(dispatch.is_none());
}

fn retest_cycle_source_state() -> RetestCycleSourceState {
    RetestCycleSourceState {
        schema_version: RETEST_CYCLE_SOURCE_STATE_SCHEMA_VERSION.to_owned(),
        generated_at_ms: 7_300_000,
        research_packet_id: "source_packet".to_owned(),
        run_scope: "focused_retest_local_validation".to_owned(),
        source_manifest_s3_bucket: "research-bucket".to_owned(),
        source_manifest_s3_key:
            "research-input-manifest/schema=research_input_manifest_v1/source/manifest.json"
                .to_owned(),
        source_research_report_s3_bucket: "research-bucket".to_owned(),
        source_research_report_s3_key:
            "research-run-report/schema=research_run_report_v1/report.json".to_owned(),
        source_research_report_id: "research_report_source".to_owned(),
        source_candidate_ids: vec!["cand_focus".to_owned()],
        replay_run_id_count: 1,
        summary_findings_count: 1,
        shadow_validation_run_count: 0,
        paper_trade_candidate_count: 0,
        safety: RetestCycleSourceStateSafety {
            dispatcher_prefix: "research-input-manifest/".to_owned(),
            state_s3_write: true,
            ecs_task_started: false,
            shadow_paper_live_enabled: false,
        },
    }
}

async fn write_refresh_cycle_inputs(root: &Path) -> (PathBuf, PathBuf) {
    let bundle =
        root.join("candidate-evidence-bundle/priority=p0/candidate_id=cand_001/part-000001.json");
    let delta = root.join("delta.json");
    let regime = root.join("regime.json");
    let manifest = root.join("manifest.json");
    let research_output = root.join("research-out");

    write_json(&bundle, &bundle_json_with_gate_inputs(1, 1_300));
    write_json(
        &delta,
        &json!([market_delta_json("delta_001", 1_300, 3_601_300, 0.021)]),
    );
    write_json(
        &regime,
        &json!([market_regime_json("regime_001", 1_300, 3_601_300)]),
    );
    write_json(
        &manifest,
        &json!({
            "schema_version": "research_input_manifest_v1",
            "research_packet_id": "manifest_packet",
            "run_scope": "manifest_batch",
            "candidate_bundle_refs": [{ "uri": bundle.display().to_string() }],
            "market_feature_delta_refs": [{ "uri": delta.display().to_string() }],
            "market_regime_context_refs": [{ "uri": regime.display().to_string() }],
            "runtime_budget_policy": {
                "max_candidate_bundle_count": 10,
                "max_market_artifact_ref_count": 10,
                "max_historical_replay_run_ref_count": 10,
                "max_replay_run_count": 20
            }
        }),
    );
    let research_summary = run(Args {
        input_manifest_file: Some(manifest.clone()),
        output_dir: Some(research_output),
        now_ms: Some(7_300_000),
        ..default_args()
    })
    .await
    .expect("research report builds");
    let report_file = output_file_containing(&research_summary, "research-run-report");
    (manifest, report_file)
}

#[tokio::test]
async fn retest_cycle_scheduler_waits_before_not_before() {
    let root = test_root("retest-cycle-scheduler-wait");
    let status_file = root.join("retest-horizon-status.json");
    let source_manifest_file = root.join("research-input-manifest.json");
    let output_file = root.join("focused-retest-manifest.json");
    write_json(&status_file, &focused_retest_status_json());
    write_json(
        &source_manifest_file,
        &focused_retest_source_manifest_json(),
    );

    let args = parse_args(
        [
            "--run-retest-cycle-scheduler".to_owned(),
            "--retest-horizon-status-file".to_owned(),
            status_file.display().to_string(),
            "--input-manifest-file".to_owned(),
            source_manifest_file.display().to_string(),
            "--focused-retest-manifest-output-file".to_owned(),
            output_file.display().to_string(),
            "--now-ms".to_owned(),
            "1779719361451".to_owned(),
        ]
        .into_iter(),
    )
    .expect("scheduler args parse")
    .expect("scheduler args returned");
    let summary = run(args).await.expect("scheduler waits");

    assert_eq!(summary.retest_horizon_statuses_validated, 1);
    assert_eq!(
        summary.retest_cycle_scheduler_action,
        Some("WAIT_UNTIL_MARKET_L1_HORIZON_MATERIALIZES".to_owned())
    );
    assert_eq!(
        summary.retest_cycle_run_not_before_ms,
        Some(1_779_719_361_452)
    );
    assert_eq!(summary.focused_retest_manifests_created, 0);
    assert!(summary.output_files.is_empty());
    assert!(!output_file.exists());
}

#[tokio::test]
async fn retest_cycle_scheduler_requires_fresh_status_after_wait_deadline() {
    let root = test_root("retest-cycle-scheduler-refresh");
    let status_file = root.join("retest-horizon-status.json");
    let source_manifest_file = root.join("research-input-manifest.json");
    let output_file = root.join("focused-retest-manifest.json");
    write_json(&status_file, &focused_retest_status_json());
    write_json(
        &source_manifest_file,
        &focused_retest_source_manifest_json(),
    );

    let args = parse_args(
        [
            "--run-retest-cycle-scheduler".to_owned(),
            "--retest-horizon-status-file".to_owned(),
            status_file.display().to_string(),
            "--input-manifest-file".to_owned(),
            source_manifest_file.display().to_string(),
            "--focused-retest-manifest-output-file".to_owned(),
            output_file.display().to_string(),
            "--now-ms".to_owned(),
            "1779719361452".to_owned(),
        ]
        .into_iter(),
    )
    .expect("scheduler args parse")
    .expect("scheduler args returned");
    let summary = run(args).await.expect("scheduler asks for refresh");

    assert_eq!(
        summary.retest_cycle_scheduler_action,
        Some("REFRESH_RETEST_HORIZON_STATUS_AFTER_WAIT_DEADLINE".to_owned())
    );
    assert_eq!(summary.focused_retest_manifests_created, 0);
    assert!(summary.output_files.is_empty());
    assert!(!output_file.exists());
}

#[tokio::test]
async fn retest_cycle_scheduler_builds_focused_manifest_when_run_now() {
    let root = test_root("retest-cycle-scheduler-run-now");
    let status_file = root.join("retest-horizon-status.json");
    let source_manifest_file = root.join("research-input-manifest.json");
    let output_file = root.join("focused-retest-manifest.json");
    write_json(&status_file, &focused_retest_run_now_status_json());
    write_json(
        &source_manifest_file,
        &focused_retest_source_manifest_json(),
    );

    let args = parse_args(
        [
            "--run-retest-cycle-scheduler".to_owned(),
            "--retest-horizon-status-file".to_owned(),
            status_file.display().to_string(),
            "--input-manifest-file".to_owned(),
            source_manifest_file.display().to_string(),
            "--focused-retest-manifest-output-file".to_owned(),
            output_file.display().to_string(),
            "--research-packet-id".to_owned(),
            "research_focus_scheduler_test".to_owned(),
            "--now-ms".to_owned(),
            "1779719361452".to_owned(),
        ]
        .into_iter(),
    )
    .expect("scheduler args parse")
    .expect("scheduler args returned");
    let summary = run(args).await.expect("scheduler builds focused manifest");

    assert_eq!(
        summary.retest_cycle_scheduler_action,
        Some("RUN_FOCUSED_RETEST_RESEARCH".to_owned())
    );
    assert_eq!(summary.focused_retest_manifests_created, 1);
    assert_eq!(summary.focused_retest_candidate_bundle_refs, 1);
    assert_eq!(summary.output_files.len(), 2);
    assert!(output_file.exists());
}

#[tokio::test]
async fn build_focused_retest_manifest_from_status_and_source_manifest() {
    let root = test_root("focused-retest-manifest-cli");
    let status_file = root.join("retest-horizon-status.json");
    let source_manifest_file = root.join("research-input-manifest.json");
    let output_file = root.join("focused-retest-manifest.json");
    let summary_file = root.join("focused-retest-manifest.summary.json");
    write_json(&status_file, &focused_retest_status_json());
    write_json(
        &source_manifest_file,
        &focused_retest_source_manifest_json(),
    );

    let args = parse_args(
        [
            "--build-focused-retest-manifest".to_owned(),
            "--retest-horizon-status-file".to_owned(),
            status_file.display().to_string(),
            "--input-manifest-file".to_owned(),
            source_manifest_file.display().to_string(),
            "--focused-retest-manifest-output-file".to_owned(),
            output_file.display().to_string(),
            "--focused-retest-summary-output-file".to_owned(),
            summary_file.display().to_string(),
            "--research-packet-id".to_owned(),
            "research_focus_test".to_owned(),
            "--run-scope".to_owned(),
            "focused_retest_local_validation".to_owned(),
            "--now-ms".to_owned(),
            "1779719361452".to_owned(),
        ]
        .into_iter(),
    )
    .expect("focused args parse")
    .expect("focused args returned");
    let summary = run(args).await.expect("focused manifest builds");

    assert_eq!(summary.retest_horizon_statuses_validated, 1);
    assert_eq!(summary.focused_retest_manifests_created, 1);
    assert_eq!(summary.focused_retest_horizon_count, 1);
    assert_eq!(summary.focused_retest_candidate_bundle_refs, 1);
    assert_eq!(summary.output_files.len(), 2);

    let manifest: Value =
        serde_json::from_slice(&fs::read(&output_file).expect("manifest")).expect("manifest json");
    assert_eq!(manifest["research_packet_id"], json!("research_focus_test"));
    assert_eq!(
        manifest["candidate_bundle_refs"].as_array().unwrap().len(),
        1
    );
    assert_eq!(
        manifest["candidate_bundle_refs"][0]["uri"],
        json!(
            "s3://bucket/candidate-evidence-bundle/priority=p0/candidate_id=cand_focus/part-000001.jsonl"
        )
    );
    assert_eq!(
        manifest["historical_replay_run_index_refs"]
            .as_array()
            .unwrap()
            .len(),
        1
    );

    let focus_summary: Value =
        serde_json::from_slice(&fs::read(&summary_file).expect("summary")).expect("summary json");
    assert_eq!(
        focus_summary["schema_version"],
        json!("research_focused_retest_manifest_summary_v1")
    );
    assert_eq!(
        focus_summary["focused"]["selected_candidate_bundle_ref_count"],
        json!(1)
    );
    assert_eq!(focus_summary["safety"]["s3_write"], json!(false));
}

#[tokio::test]
async fn build_focused_retest_manifest_rejects_empty_selection() {
    let root = test_root("focused-retest-empty-cli");
    let status_file = root.join("retest-horizon-status.json");
    let source_manifest_file = root.join("research-input-manifest.json");
    let output_file = root.join("focused-retest-manifest.json");
    write_json(&status_file, &focused_retest_status_json());
    write_json(
        &source_manifest_file,
        &focused_retest_source_manifest_json(),
    );

    let args = parse_args(
        [
            "--build-focused-retest-manifest".to_owned(),
            "--retest-horizon-status-file".to_owned(),
            status_file.display().to_string(),
            "--input-manifest-file".to_owned(),
            source_manifest_file.display().to_string(),
            "--focused-retest-manifest-output-file".to_owned(),
            output_file.display().to_string(),
            "--focused-retest-next-actions".to_owned(),
            "run_research_replay_for_horizon".to_owned(),
        ]
        .into_iter(),
    )
    .expect("focused args parse")
    .expect("focused args returned");
    let error = run(args)
        .await
        .expect_err("empty focused selection is rejected");

    assert!(
        error
            .to_string()
            .contains("selected zero candidate bundle refs")
    );
    assert!(!output_file.exists());
}

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
        market_live_nats_configs_for_candidates(&args, &candidates, "nats://127.0.0.1:4222");

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
            "research-paper-watch-live-ton",
            "research-paper-watch-live-zec"
        ]
    );
    assert!(
        configs
            .iter()
            .all(|config| config.url == "nats://127.0.0.1:4222")
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
        market_live_nats_configs_for_candidates(&args, &candidates, "nats://127.0.0.1:4222");

    assert_eq!(configs.len(), 1);
    assert_eq!(configs[0].subject, "market_live_tick.created.binance.ton");
    assert_eq!(configs[0].consumer, "custom-consumer");
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

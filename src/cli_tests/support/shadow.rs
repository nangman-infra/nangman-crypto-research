use super::*;

pub(in crate::cli::tests) fn oss_adapter_run_json(
    candidate_lifecycle_key: &str,
    verdict: &str,
) -> Value {
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

pub(in crate::cli::tests) fn shadow_cycle_wait_decision_json() -> Value {
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

pub(in crate::cli::tests) fn shadow_validation_run_json(
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

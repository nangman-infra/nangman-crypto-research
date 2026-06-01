use super::*;

pub(in crate::cli::tests) fn market_live_tick_json(
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

pub(in crate::cli::tests) fn paper_watch_candidate_json(id: &str, symbol: &str) -> Value {
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

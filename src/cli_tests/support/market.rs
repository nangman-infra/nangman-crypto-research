use super::*;

pub(in crate::cli::tests) fn market_delta_json(
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

pub(in crate::cli::tests) fn retarget_market_delta_symbol(delta: &mut Value, symbol: &str) {
    delta["symbol_native"] = json!(format!("{symbol}USDT"));
    delta["symbol_canonical"] = json!(symbol);
}

pub(in crate::cli::tests) fn market_liquidity_delta_json(
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

pub(in crate::cli::tests) fn market_liquidity_delta_json_with_value(
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

pub(in crate::cli::tests) fn market_regime_json(
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

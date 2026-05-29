use crate::model::{MarketFeatureDelta, MarketRegimeContext};
use std::collections::BTreeSet;

const HORIZON_MATERIALIZATION_TOLERANCE_MS: i64 = 1_000;

pub(super) fn matching_market_deltas<'a>(
    symbol: &str,
    window_start_ms: i64,
    window_end_ms: i64,
    market_deltas: &'a [MarketFeatureDelta],
) -> Vec<&'a MarketFeatureDelta> {
    market_deltas
        .iter()
        .filter(|delta| {
            delta.symbol_canonical == symbol
                && window_contains(
                    delta.window_start_ms,
                    delta.window_end_ms,
                    window_start_ms,
                    window_end_ms,
                )
                && is_valid_quality(&delta.quality_status)
        })
        .collect()
}

pub(super) fn matching_regime_contexts(
    window_start_ms: i64,
    window_end_ms: i64,
    regime_contexts: &[MarketRegimeContext],
) -> Vec<&MarketRegimeContext> {
    regime_contexts
        .iter()
        .filter(|context| {
            window_contains(
                context.window_start_ms,
                context.window_end_ms,
                window_start_ms,
                window_end_ms,
            ) && is_valid_quality(&context.quality_status)
        })
        .collect()
}

pub(super) fn regime_labels(regime_contexts: &[&MarketRegimeContext]) -> Vec<String> {
    regime_contexts
        .iter()
        .map(|context| context.volatility_regime.trim())
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub(super) fn horizon_is_materialized(matched: &[&MarketFeatureDelta], window_end_ms: i64) -> bool {
    matched
        .iter()
        .map(|delta| delta.window_end_ms)
        .max()
        .is_some_and(|latest_end_ms| {
            latest_end_ms + HORIZON_MATERIALIZATION_TOLERANCE_MS >= window_end_ms
        })
}

pub(super) fn return_bps_values(matched: &[&MarketFeatureDelta]) -> Vec<f64> {
    matched
        .iter()
        .filter_map(|delta| return_pct(delta).map(|value| value * 100.0))
        .collect()
}

pub(super) fn btc_adjustment_bps(regime_contexts: &[&MarketRegimeContext]) -> Option<f64> {
    regime_contexts
        .iter()
        .find_map(|context| context.btc_return_same_window.map(|value| value * 100.0))
}

pub(super) fn average(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

fn return_pct(delta: &MarketFeatureDelta) -> Option<f64> {
    delta
        .price_change_same_window
        .or(delta.change_pct_15m)
        .or(delta.change_pct_1h)
}

fn window_contains(
    actual_start_ms: i64,
    actual_end_ms: i64,
    expected_start_ms: i64,
    expected_end_ms: i64,
) -> bool {
    actual_start_ms >= expected_start_ms && actual_end_ms <= expected_end_ms
}

fn is_valid_quality(quality_status: &str) -> bool {
    !quality_status.eq_ignore_ascii_case("invalid")
}

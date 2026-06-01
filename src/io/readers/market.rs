use crate::error::AppResult;
use crate::model::{MarketFeatureDelta, MarketRegimeContext};
use std::collections::BTreeSet;
use std::path::Path;

use super::super::json::{
    read_json_array_or_jsonl, read_json_array_or_jsonl_bytes, read_json_array_or_jsonl_bytes_filter,
};

pub fn read_market_feature_deltas(path: &Path) -> AppResult<Vec<MarketFeatureDelta>> {
    read_json_array_or_jsonl(path)
}

pub fn read_market_feature_deltas_from_bytes(
    label: &str,
    bytes: &[u8],
) -> AppResult<Vec<MarketFeatureDelta>> {
    read_json_array_or_jsonl_bytes(label, bytes)
}

pub fn read_market_feature_deltas_matching_symbols_from_bytes(
    label: &str,
    bytes: &[u8],
    symbols: &BTreeSet<String>,
) -> AppResult<Vec<MarketFeatureDelta>> {
    if symbols.is_empty() {
        return read_market_feature_deltas_from_bytes(label, bytes);
    }
    read_json_array_or_jsonl_bytes_filter(label, bytes, |delta: &MarketFeatureDelta| {
        symbols.contains(delta.symbol_canonical.as_str())
    })
}

pub fn read_market_regime_contexts(path: &Path) -> AppResult<Vec<MarketRegimeContext>> {
    read_json_array_or_jsonl(path)
}

pub fn read_market_regime_contexts_from_bytes(
    label: &str,
    bytes: &[u8],
) -> AppResult<Vec<MarketRegimeContext>> {
    read_json_array_or_jsonl_bytes(label, bytes)
}

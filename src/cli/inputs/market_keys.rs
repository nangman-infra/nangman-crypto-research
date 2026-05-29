use super::*;

mod artifact_family;
mod key_normalization;
mod report_time;
mod window;

pub(in crate::cli) use artifact_family::{
    market_feature_delta_s3_keys, market_regime_context_s3_keys,
};
pub(in crate::cli) use key_normalization::{
    insert_normalized_s3_key, market_l1_run_id_from_key, normalize_s3_key,
};
pub(in crate::cli) use report_time::deterministic_report_created_at_ms;
pub(in crate::cli) use window::market_l1_replay_window_starts;

pub(in crate::cli) fn should_read_market_s3(args: &Args) -> bool {
    args.input_bundle_s3_bucket.is_some()
        || args.market_l1_s3_bucket.is_some()
        || !args.market_feature_delta_s3_keys.is_empty()
        || !args.market_regime_context_s3_keys.is_empty()
}

pub(in crate::cli) fn bundle_symbol_filter(
    bundles: &[IntelCandidateEvidenceBundle],
) -> BTreeSet<String> {
    bundles
        .iter()
        .flat_map(|bundle| bundle.normalized_symbols.iter().cloned())
        .collect()
}

pub(in crate::cli) fn market_l1_s3_bucket(args: &Args) -> &str {
    args.market_l1_s3_bucket
        .as_deref()
        .unwrap_or(DEFAULT_MARKET_L1_S3_BUCKET)
}

use crate::model::{
    IntelCandidateEvidenceBundle, MarketFeatureDelta, MarketRegimeContext, ReplayResultSummary,
    ReplayRunStatus, ResearchBias,
};

mod matching;
mod outcome;

use matching::{
    average, btc_adjustment_bps, horizon_is_materialized, matching_market_deltas,
    matching_regime_contexts, regime_labels, return_bps_values,
};
use outcome::{completed_summary, incomplete_summary};

pub(super) fn summarize_native_replay(
    bundle: &IntelCandidateEvidenceBundle,
    symbol: &str,
    window_start_ms: i64,
    window_end_ms: i64,
    market_deltas: &[MarketFeatureDelta],
    regime_contexts: &[MarketRegimeContext],
) -> ReplayResultSummary {
    let matched = matching_market_deltas(symbol, window_start_ms, window_end_ms, market_deltas);
    let matched_regimes = matching_regime_contexts(window_start_ms, window_end_ms, regime_contexts);
    let cost_bps = super::cost::estimated_cost_bps(bundle);

    if matched.is_empty() {
        return incomplete_summary(
            bundle,
            &matched,
            ReplayRunStatus::MissingMarketReplayData,
            "missing_native_replay_market_data",
            cost_bps,
            Vec::new(),
        );
    }

    let market_regime_labels = regime_labels(&matched_regimes);
    if !horizon_is_materialized(&matched, window_end_ms) {
        return incomplete_summary(
            bundle,
            &matched,
            ReplayRunStatus::InsufficientEvidence,
            "native_replay_horizon_not_materialized",
            cost_bps,
            market_regime_labels,
        );
    }

    let returns = return_bps_values(&matched);
    if returns.is_empty() {
        return incomplete_summary(
            bundle,
            &matched,
            ReplayRunStatus::InsufficientEvidence,
            "native_replay_return_metric_missing",
            cost_bps,
            market_regime_labels,
        );
    }

    let raw_return_bps = average(&returns);
    let btc_adjustment_bps = btc_adjustment_bps(&matched_regimes);
    let btc_adjusted_return_bps = btc_adjustment_bps
        .map(|btc_return_bps| raw_return_bps - btc_return_bps)
        .unwrap_or(raw_return_bps);
    let net_after_cost_bps = btc_adjusted_return_bps - cost_bps;

    let (bias, mut reasons) = if net_after_cost_bps <= 0.0 {
        (
            ResearchBias::PruneBias,
            vec!["native_replay_net_edge_non_positive".to_owned()],
        )
    } else {
        (
            ResearchBias::RetestBias,
            vec![
                "native_replay_positive_but_survival_not_proven".to_owned(),
                "needs_unseen_window_validation".to_owned(),
            ],
        )
    };
    if btc_adjustment_bps.is_none() {
        reasons.push("market_regime_context_missing".to_owned());
    }
    if market_regime_labels.is_empty() {
        reasons.push("market_regime_label_missing".to_owned());
    }

    completed_summary(outcome::CompletedSummaryInput {
        bundle,
        matched: &matched,
        bias,
        reason_codes: reasons,
        raw_return_bps,
        btc_adjusted_return_bps,
        net_after_cost_bps,
        estimated_cost_bps: cost_bps,
        market_regime_labels,
    })
}

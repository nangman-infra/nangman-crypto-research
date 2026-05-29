use super::super::liquidity::liquidity_filter_summary;
use crate::model::{
    IntelCandidateEvidenceBundle, MarketFeatureDelta, ReplayResultSummary, ReplayRunStatus,
    ResearchBias,
};

pub(super) struct CompletedSummaryInput<'a> {
    pub(super) bundle: &'a IntelCandidateEvidenceBundle,
    pub(super) matched: &'a [&'a MarketFeatureDelta],
    pub(super) bias: ResearchBias,
    pub(super) reason_codes: Vec<String>,
    pub(super) raw_return_bps: f64,
    pub(super) btc_adjusted_return_bps: f64,
    pub(super) net_after_cost_bps: f64,
    pub(super) estimated_cost_bps: f64,
    pub(super) market_regime_labels: Vec<String>,
}

pub(super) fn incomplete_summary(
    bundle: &IntelCandidateEvidenceBundle,
    matched: &[&MarketFeatureDelta],
    status: ReplayRunStatus,
    reason_code: &'static str,
    estimated_cost_bps: f64,
    market_regime_labels: Vec<String>,
) -> ReplayResultSummary {
    ReplayResultSummary {
        status,
        bias: ResearchBias::RetestBias,
        reason_codes: vec![reason_code.to_owned()],
        matched_market_delta_count: matched.len(),
        raw_return_bps: None,
        btc_adjusted_return_bps: None,
        net_after_cost_bps: None,
        estimated_cost_bps,
        market_regime_labels,
        liquidity_filter_summary: liquidity_filter_summary(bundle, matched),
    }
}

pub(super) fn completed_summary(input: CompletedSummaryInput<'_>) -> ReplayResultSummary {
    ReplayResultSummary {
        status: ReplayRunStatus::Completed,
        bias: input.bias,
        reason_codes: input.reason_codes,
        matched_market_delta_count: input.matched.len(),
        raw_return_bps: Some(input.raw_return_bps),
        btc_adjusted_return_bps: Some(input.btc_adjusted_return_bps),
        net_after_cost_bps: Some(input.net_after_cost_bps),
        estimated_cost_bps: input.estimated_cost_bps,
        market_regime_labels: input.market_regime_labels,
        liquidity_filter_summary: liquidity_filter_summary(input.bundle, input.matched),
    }
}

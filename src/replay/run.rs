use super::build::{build_replay_run, first_symbol};
use super::cost::estimated_cost_bps;
use super::liquidity::liquidity_filter_summary;
use super::summary::summarize_native_replay;
use crate::admission::{AdmissionDecision, horizon_ms};
use crate::model::{
    IntelCandidateEvidenceBundle, MarketFeatureDelta, MarketRegimeContext, ReplayResultSummary,
    ReplayRun, ReplayRunStatus, ResearchBias,
};

pub fn build_invalid_replay_run(
    bundle: &IntelCandidateEvidenceBundle,
    admission: &AdmissionDecision,
) -> ReplayRun {
    let symbol = first_symbol(bundle);
    build_replay_run(
        bundle,
        &symbol,
        "invalid_input",
        bundle.decision_available_at_ms,
        bundle.decision_available_at_ms,
        ReplayResultSummary {
            status: ReplayRunStatus::InvalidInput,
            bias: ResearchBias::RetestBias,
            reason_codes: admission.reason_codes.clone(),
            matched_market_delta_count: 0,
            raw_return_bps: None,
            btc_adjusted_return_bps: None,
            net_after_cost_bps: None,
            estimated_cost_bps: estimated_cost_bps(bundle),
            market_regime_labels: Vec::new(),
            liquidity_filter_summary: liquidity_filter_summary(bundle, &[]),
        },
    )
}

pub fn run_native_replay(
    bundle: &IntelCandidateEvidenceBundle,
    market_deltas: &[MarketFeatureDelta],
    regime_contexts: &[MarketRegimeContext],
) -> Vec<ReplayRun> {
    let mut runs = Vec::new();
    for symbol in &bundle.normalized_symbols {
        for horizon in &bundle.allowed_horizons {
            let Some(duration_ms) = horizon_ms(horizon) else {
                continue;
            };
            let window_start_ms = bundle.forbidden_lookahead_boundary_ms;
            let window_end_ms = window_start_ms + duration_ms;
            let summary = summarize_native_replay(
                bundle,
                symbol,
                window_start_ms,
                window_end_ms,
                market_deltas,
                regime_contexts,
            );
            runs.push(build_replay_run(
                bundle,
                symbol,
                horizon,
                window_start_ms,
                window_end_ms,
                summary,
            ));
        }
    }
    runs
}

use super::policy::{
    ALLOCATION_POLICY_VERSION, LIVE_DEFAULT_TOTAL_NOTIONAL_PCT, MAX_CANDIDATE_NOTIONAL_PCT,
    MAX_FAMILY_NOTIONAL_PCT, MAX_SYMBOL_NOTIONAL_PCT,
};
use super::symbols::infer_market_regime;
use crate::hash::stable_id;
use crate::model::{
    CandidateAllocation, PORTFOLIO_ALLOCATION_SNAPSHOT_SCHEMA_VERSION, PortfolioAllocationSnapshot,
    PortfolioReduceOnlySignal, PortfolioRiskRejectEvent, ResearchRunReport,
};
use std::collections::BTreeSet;

pub(super) fn build_snapshot(
    report: &ResearchRunReport,
    allocations: Vec<CandidateAllocation>,
    rejects: &[PortfolioRiskRejectEvent],
    reduce_only_signals: &[PortfolioReduceOnlySignal],
    computed_at_ms: i64,
) -> PortfolioAllocationSnapshot {
    let reason_codes = snapshot_reason_codes(&allocations, rejects, reduce_only_signals);
    let snapshot_id = stable_id(
        "portfolio_allocation_snapshot",
        &[
            &report.research_run_report_id,
            &computed_at_ms.to_string(),
            &allocations.len().to_string(),
            &rejects.len().to_string(),
        ],
    );
    PortfolioAllocationSnapshot {
        portfolio_allocation_snapshot_id: snapshot_id,
        schema_version: PORTFOLIO_ALLOCATION_SNAPSHOT_SCHEMA_VERSION.to_owned(),
        allocation_policy_version: ALLOCATION_POLICY_VERSION.to_owned(),
        computed_at_ms,
        market_regime: infer_market_regime(&report.shadow_validation_runs),
        active_candidate_count: allocations.len(),
        max_total_notional_pct: LIVE_DEFAULT_TOTAL_NOTIONAL_PCT,
        max_symbol_notional_pct: MAX_SYMBOL_NOTIONAL_PCT,
        max_candidate_notional_pct: MAX_CANDIDATE_NOTIONAL_PCT,
        max_regime_notional_pct: MAX_FAMILY_NOTIONAL_PCT,
        candidate_allocations: allocations,
        rejected_candidates: rejects.to_vec(),
        reason_codes,
    }
}

fn snapshot_reason_codes(
    allocations: &[CandidateAllocation],
    rejects: &[PortfolioRiskRejectEvent],
    reduce_only_signals: &[PortfolioReduceOnlySignal],
) -> Vec<String> {
    let mut reasons = BTreeSet::new();
    if allocations.is_empty() {
        reasons.insert("live_default_notional_zero".to_owned());
    } else {
        reasons.insert("portfolio_allocation_computed".to_owned());
        reasons.insert("live_default_notional_zero".to_owned());
    }
    for reject in rejects {
        reasons.insert(reject.reason.clone());
    }
    if !reduce_only_signals.is_empty() {
        reasons.insert("portfolio_reduce_only_signal_created".to_owned());
    }
    reasons.into_iter().collect()
}

use super::events::{reduce_only_signal, reject_event};
use super::policy::{
    MAX_CANDIDATE_NOTIONAL_PCT, MAX_SYMBOL_OPEN_CANDIDATES, MAX_TOTAL_OPEN_CANDIDATES,
};
use super::snapshot::build_snapshot;
use super::symbols::{critical_event_reason, first_symbol};
use crate::model::{
    CandidateAllocation, IntelCandidateEvidenceBundle, PortfolioAllocationSnapshot,
    PortfolioReduceOnlySignal, PortfolioRiskRejectEvent, ResearchBias, ResearchRunReport,
};
use std::collections::BTreeMap;

pub fn build_portfolio_artifacts(
    report: &ResearchRunReport,
    bundles: &[IntelCandidateEvidenceBundle],
    computed_at_ms: i64,
) -> (
    PortfolioAllocationSnapshot,
    Vec<PortfolioRiskRejectEvent>,
    Vec<PortfolioReduceOnlySignal>,
) {
    let bundles_by_key = bundles
        .iter()
        .map(|bundle| (bundle.candidate_lifecycle_key.clone(), bundle))
        .collect::<BTreeMap<_, _>>();
    let shadows_by_key = report
        .shadow_validation_runs
        .iter()
        .map(|run| (run.candidate_lifecycle_key.clone(), run))
        .collect::<BTreeMap<_, _>>();

    let mut allocations = Vec::new();
    let mut rejects = Vec::new();
    let mut reduce_only_signals = Vec::new();
    let mut symbol_counts = BTreeMap::<String, usize>::new();
    let mut family_counts = BTreeMap::<String, usize>::new();

    for finding in report
        .summary_findings
        .iter()
        .filter(|finding| finding.bias == ResearchBias::PromoteToShadowBias)
    {
        let Some(bundle) = bundles_by_key.get(&finding.candidate_lifecycle_key) else {
            continue;
        };
        let Some(shadow) = shadows_by_key.get(&finding.candidate_lifecycle_key) else {
            continue;
        };
        let symbol = first_symbol(bundle, shadow);
        let family = bundle.hypothesis_type.clone();

        if let Some(reason) = critical_event_reason(bundle) {
            rejects.push(reject_event(
                &finding.candidate_lifecycle_key,
                &symbol,
                reason,
                computed_at_ms,
            ));
            reduce_only_signals.push(reduce_only_signal(&symbol, reason, computed_at_ms));
            continue;
        }
        if allocations.len() >= MAX_TOTAL_OPEN_CANDIDATES {
            rejects.push(reject_event(
                &finding.candidate_lifecycle_key,
                &symbol,
                "portfolio_total_candidate_cap",
                computed_at_ms,
            ));
            continue;
        }
        if symbol_counts.get(&symbol).copied().unwrap_or(0) >= MAX_SYMBOL_OPEN_CANDIDATES {
            rejects.push(reject_event(
                &finding.candidate_lifecycle_key,
                &symbol,
                "portfolio_symbol_duplicate_cap",
                computed_at_ms,
            ));
            continue;
        }
        if family_counts.get(&family).copied().unwrap_or(0) >= MAX_TOTAL_OPEN_CANDIDATES {
            rejects.push(reject_event(
                &finding.candidate_lifecycle_key,
                &symbol,
                "portfolio_family_concentration_cap",
                computed_at_ms,
            ));
            continue;
        }

        symbol_counts
            .entry(symbol.clone())
            .and_modify(|count| *count += 1)
            .or_insert(1);
        family_counts
            .entry(family.clone())
            .and_modify(|count| *count += 1)
            .or_insert(1);
        allocations.push(CandidateAllocation {
            candidate_lifecycle_key: finding.candidate_lifecycle_key.clone(),
            symbol_canonical: symbol,
            strategy_id: shadow
                .start_condition_summary
                .research_aggregate_key
                .clone(),
            allocation_weight: 0.0,
            max_notional_pct: MAX_CANDIDATE_NOTIONAL_PCT,
            correlation_bucket: family,
            holding_deadline_ms: shadow.holding_policy.absolute_exit_deadline_ms,
            paper_survival_band: shadow.expected_survival_band.clone(),
        });
    }

    if !allocations.is_empty() {
        let weight = 1.0 / allocations.len() as f64;
        for allocation in &mut allocations {
            allocation.allocation_weight = weight;
        }
    }

    let snapshot = build_snapshot(
        report,
        allocations,
        &rejects,
        &reduce_only_signals,
        computed_at_ms,
    );
    (snapshot, rejects, reduce_only_signals)
}

use crate::hash::stable_id;
use crate::model::{
    CandidateAllocation, IntelCandidateEvidenceBundle,
    PORTFOLIO_ALLOCATION_SNAPSHOT_SCHEMA_VERSION, PORTFOLIO_REDUCE_ONLY_SIGNAL_SCHEMA_VERSION,
    PORTFOLIO_RISK_REJECT_EVENT_SCHEMA_VERSION, PortfolioAllocationSnapshot,
    PortfolioReduceOnlySignal, PortfolioRiskRejectEvent, ResearchBias, ResearchRunReport,
    ShadowValidationRun, SurvivalBand,
};
use std::collections::{BTreeMap, BTreeSet};

const ALLOCATION_POLICY_VERSION: &str = "portfolio_allocation_policy_v1_2026_05_12";
const MAX_TOTAL_OPEN_CANDIDATES: usize = 3;
const MAX_SYMBOL_OPEN_CANDIDATES: usize = 1;
const MAX_CANDIDATE_NOTIONAL_PCT: f64 = 3.0;
const MAX_SYMBOL_NOTIONAL_PCT: f64 = 5.0;
const MAX_FAMILY_NOTIONAL_PCT: f64 = 10.0;
const LIVE_DEFAULT_TOTAL_NOTIONAL_PCT: f64 = 0.0;

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

    let reason_codes = snapshot_reason_codes(&allocations, &rejects, &reduce_only_signals);
    let snapshot_id = stable_id(
        "portfolio_allocation_snapshot",
        &[
            &report.research_run_report_id,
            &computed_at_ms.to_string(),
            &allocations.len().to_string(),
            &rejects.len().to_string(),
        ],
    );
    let snapshot = PortfolioAllocationSnapshot {
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
        rejected_candidates: rejects.clone(),
        reason_codes,
    };

    (snapshot, rejects, reduce_only_signals)
}

fn first_symbol(bundle: &IntelCandidateEvidenceBundle, shadow: &ShadowValidationRun) -> String {
    bundle
        .normalized_symbols
        .first()
        .cloned()
        .unwrap_or_else(|| shadow.symbol_canonical.clone())
}

fn critical_event_reason(bundle: &IntelCandidateEvidenceBundle) -> Option<&'static str> {
    let critical = bundle
        .event_types
        .iter()
        .map(|event_type| event_type.as_str())
        .collect::<BTreeSet<_>>();
    [
        "exchange_delisting",
        "deposit_withdrawal_halt",
        "chain_halt",
        "exploit",
        "regulatory_ban",
        "exchange_operational_event",
        "liquidity_vanish",
        "market_data_integrity_failure",
    ]
    .into_iter()
    .find(|reason| critical.contains(reason))
}

fn reject_event(
    candidate_lifecycle_key: &str,
    symbol: &str,
    reason: &str,
    computed_at_ms: i64,
) -> PortfolioRiskRejectEvent {
    PortfolioRiskRejectEvent {
        portfolio_risk_reject_event_id: stable_id(
            "portfolio_risk_reject",
            &[
                candidate_lifecycle_key,
                symbol,
                reason,
                &computed_at_ms.to_string(),
            ],
        ),
        schema_version: PORTFOLIO_RISK_REJECT_EVENT_SCHEMA_VERSION.to_owned(),
        candidate_lifecycle_key: candidate_lifecycle_key.to_owned(),
        symbol_canonical: symbol.to_owned(),
        reason: reason.to_owned(),
        computed_at_ms,
    }
}

fn reduce_only_signal(
    symbol: &str,
    reason: &str,
    computed_at_ms: i64,
) -> PortfolioReduceOnlySignal {
    PortfolioReduceOnlySignal {
        portfolio_reduce_only_signal_id: stable_id(
            "portfolio_reduce_only",
            &[symbol, reason, &computed_at_ms.to_string()],
        ),
        schema_version: PORTFOLIO_REDUCE_ONLY_SIGNAL_SCHEMA_VERSION.to_owned(),
        symbol_canonical: symbol.to_owned(),
        reason: reason.to_owned(),
        computed_at_ms,
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

fn infer_market_regime(shadow_validation_runs: &[ShadowValidationRun]) -> String {
    let strongest = shadow_validation_runs
        .iter()
        .map(|run| &run.expected_survival_band)
        .max_by_key(|band| match band {
            SurvivalBand::Fragile => 0,
            SurvivalBand::Conditional => 1,
            SurvivalBand::Stable => 2,
            SurvivalBand::Exceptional => 3,
        });
    match strongest {
        Some(SurvivalBand::Exceptional) => "survival_exceptional".to_owned(),
        Some(SurvivalBand::Stable) => "survival_stable".to_owned(),
        Some(SurvivalBand::Conditional) => "survival_conditional".to_owned(),
        Some(SurvivalBand::Fragile) => "survival_fragile".to_owned(),
        None => "no_active_shadow_candidate".to_owned(),
    }
}

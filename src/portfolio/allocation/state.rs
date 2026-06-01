use super::candidate::PortfolioCandidate;
use super::{artifacts::AllocationArtifacts, counts::AllocationCounts};
use crate::model::{CandidateAllocation, PortfolioReduceOnlySignal, PortfolioRiskRejectEvent};

#[derive(Default)]
pub(super) struct AllocationState {
    allocations: Vec<CandidateAllocation>,
    rejects: Vec<PortfolioRiskRejectEvent>,
    reduce_only_signals: Vec<PortfolioReduceOnlySignal>,
    counts: AllocationCounts,
}

impl AllocationState {
    pub(super) fn apply_candidate(
        &mut self,
        candidate: PortfolioCandidate<'_>,
        computed_at_ms: i64,
    ) {
        if let Some(reason) = super::super::symbols::critical_event_reason(candidate.bundle) {
            self.reject(&candidate, reason, computed_at_ms);
            self.reduce_only_signals
                .push(super::super::events::reduce_only_signal(
                    &candidate.symbol,
                    reason,
                    computed_at_ms,
                ));
            return;
        }
        if self.allocations.len() >= super::super::policy::MAX_TOTAL_OPEN_CANDIDATES {
            self.reject(&candidate, "portfolio_total_candidate_cap", computed_at_ms);
            return;
        }
        if self.counts.symbol(&candidate.symbol) >= super::super::policy::MAX_SYMBOL_OPEN_CANDIDATES
        {
            self.reject(&candidate, "portfolio_symbol_duplicate_cap", computed_at_ms);
            return;
        }
        if self.counts.family(&candidate.family) >= super::super::policy::MAX_TOTAL_OPEN_CANDIDATES
        {
            self.reject(
                &candidate,
                "portfolio_family_concentration_cap",
                computed_at_ms,
            );
            return;
        }

        self.accept(candidate);
    }

    pub(super) fn finish(mut self) -> AllocationArtifacts {
        assign_equal_weights(&mut self.allocations);
        AllocationArtifacts {
            allocations: self.allocations,
            rejects: self.rejects,
            reduce_only_signals: self.reduce_only_signals,
        }
    }

    fn accept(&mut self, candidate: PortfolioCandidate<'_>) {
        self.counts.record(&candidate.symbol, &candidate.family);
        self.allocations.push(CandidateAllocation {
            candidate_lifecycle_key: candidate.lifecycle_key.to_owned(),
            symbol_canonical: candidate.symbol,
            strategy_id: candidate
                .shadow
                .start_condition_summary
                .research_aggregate_key
                .clone(),
            allocation_weight: 0.0,
            max_notional_pct: super::super::policy::MAX_CANDIDATE_NOTIONAL_PCT,
            correlation_bucket: candidate.family,
            holding_deadline_ms: candidate.shadow.holding_policy.absolute_exit_deadline_ms,
            paper_survival_band: candidate.shadow.expected_survival_band.clone(),
        });
    }

    fn reject(&mut self, candidate: &PortfolioCandidate<'_>, reason: &str, computed_at_ms: i64) {
        self.rejects.push(super::super::events::reject_event(
            candidate.lifecycle_key,
            &candidate.symbol,
            reason,
            computed_at_ms,
        ));
    }
}

fn assign_equal_weights(allocations: &mut [CandidateAllocation]) {
    if allocations.is_empty() {
        return;
    }
    let weight = 1.0 / allocations.len() as f64;
    for allocation in allocations {
        allocation.allocation_weight = weight;
    }
}

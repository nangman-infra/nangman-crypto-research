use crate::model::{CandidateAllocation, PortfolioReduceOnlySignal, PortfolioRiskRejectEvent};

pub(super) struct AllocationArtifacts {
    pub(super) allocations: Vec<CandidateAllocation>,
    pub(super) rejects: Vec<PortfolioRiskRejectEvent>,
    pub(super) reduce_only_signals: Vec<PortfolioReduceOnlySignal>,
}

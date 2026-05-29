use super::symbol::normalize_symbol;
use crate::model::PaperWatchCandidate;
use std::collections::BTreeMap;

pub(super) fn index_safe_candidates_by_symbol(
    candidates: &[PaperWatchCandidate],
) -> BTreeMap<String, Vec<&PaperWatchCandidate>> {
    let mut candidates_by_symbol: BTreeMap<String, Vec<&PaperWatchCandidate>> = BTreeMap::new();
    for candidate in candidates {
        if !is_safe_paper_watch_candidate(candidate) {
            continue;
        }
        candidates_by_symbol
            .entry(normalize_symbol(&candidate.symbol_canonical))
            .or_default()
            .push(candidate);
    }
    candidates_by_symbol
}

fn is_safe_paper_watch_candidate(candidate: &PaperWatchCandidate) -> bool {
    candidate.safety.paper_only
        && !candidate.safety.live_enabled
        && !candidate.safety.order_execution_enabled
        && !candidate.safety.execution_approval_emitted
}

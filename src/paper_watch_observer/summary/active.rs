use crate::model::PaperWatchCandidate;

pub fn active_candidates(
    candidates: &[PaperWatchCandidate],
    now_ms: i64,
) -> Vec<PaperWatchCandidate> {
    candidates
        .iter()
        .filter(|candidate| {
            candidate.safety.paper_only
                && !candidate.safety.live_enabled
                && !candidate.safety.order_execution_enabled
                && !candidate.safety.execution_approval_emitted
                && now_ms
                    < candidate.created_at_ms
                        + i64::from(candidate.absolute_max_holding_hours) * 60 * 60 * 1000
        })
        .cloned()
        .collect()
}

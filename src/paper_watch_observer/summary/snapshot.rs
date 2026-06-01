use super::super::stats::{count_by, summarize_returns};
use super::super::types::{
    PAPER_WATCH_OBSERVER_SNAPSHOT_SCHEMA_VERSION, PaperWatchObserverSafety,
    PaperWatchObserverSnapshot,
};
use super::active::active_candidates;
use super::candidate::candidate_summary;
use crate::model::{PaperWatchCandidate, PaperWatchLiveMark};
use std::collections::{BTreeMap, BTreeSet};

pub(in crate::paper_watch_observer) fn build_snapshot(
    observer_run_id: &str,
    iteration: usize,
    created_at_ms: i64,
    candidates: &[PaperWatchCandidate],
    new_marks: &[PaperWatchLiveMark],
    marks_by_candidate: &BTreeMap<String, Vec<PaperWatchLiveMark>>,
    seen_live_mark_count: usize,
) -> PaperWatchObserverSnapshot {
    let active_candidates = active_candidates(candidates, created_at_ms);
    let active_symbols = active_candidates
        .iter()
        .map(|candidate| candidate.symbol_canonical.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let all_marks = marks_by_candidate
        .values()
        .flat_map(|marks| marks.iter())
        .collect::<Vec<_>>();

    PaperWatchObserverSnapshot {
        schema_version: PAPER_WATCH_OBSERVER_SNAPSHOT_SCHEMA_VERSION.to_owned(),
        observer_run_id: observer_run_id.to_owned(),
        iteration,
        created_at_ms,
        active_candidate_count: active_candidates.len(),
        active_symbols,
        restored_live_mark_count: seen_live_mark_count.saturating_sub(new_marks.len()),
        new_live_mark_count: new_marks.len(),
        total_live_mark_count: seen_live_mark_count,
        lifecycle_counts: count_by(all_marks.iter().map(|mark| mark.lifecycle_state.as_str())),
        venue_counts: count_by(all_marks.iter().map(|mark| mark.venue.as_str())),
        net_return_bps: summarize_returns(all_marks.iter().map(|mark| mark.net_return_bps)),
        candidate_summaries: active_candidates
            .iter()
            .map(|candidate| candidate_summary(candidate, created_at_ms, marks_by_candidate))
            .collect(),
        safety: PaperWatchObserverSafety {
            paper_only: true,
            live_enabled: false,
            order_execution_enabled: false,
            execution_approval_emitted: false,
        },
    }
}

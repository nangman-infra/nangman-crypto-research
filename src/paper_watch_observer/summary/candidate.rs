use super::super::stats::{finite_max, finite_min, max_drawdown_bps};
use super::super::types::PaperWatchObserverCandidateSummary;
use super::super::verdict::observer_verdict;
use crate::model::{PaperWatchCandidate, PaperWatchLiveMark, PaperWatchSafety};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn candidate_summary(
    candidate: &PaperWatchCandidate,
    now_ms: i64,
    marks_by_candidate: &BTreeMap<String, Vec<PaperWatchLiveMark>>,
) -> PaperWatchObserverCandidateSummary {
    let marks = marks_by_candidate
        .get(&candidate.paper_watch_candidate_id)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let latest = marks.iter().max_by(|left, right| {
        (left.exchange_timestamp_ms, left.ingest_timestamp_ms)
            .cmp(&(right.exchange_timestamp_ms, right.ingest_timestamp_ms))
    });
    let returns = marks
        .iter()
        .map(|mark| mark.net_return_bps)
        .collect::<Vec<_>>();
    let max_return_bps = finite_max(returns.iter().copied());
    let min_return_bps = finite_min(returns.iter().copied());
    let latest_return_bps = latest.map(|mark| mark.net_return_bps);
    let lifecycle_state = latest
        .map(|mark| mark.lifecycle_state.clone())
        .unwrap_or_else(|| "waiting_for_live_tick".to_owned());
    let holding_elapsed_ms = now_ms.saturating_sub(candidate.created_at_ms).max(0);

    PaperWatchObserverCandidateSummary {
        paper_watch_candidate_id: candidate.paper_watch_candidate_id.clone(),
        candidate_id: candidate.candidate_id.clone(),
        candidate_lifecycle_key: candidate.candidate_lifecycle_key.clone(),
        symbol_canonical: candidate.symbol_canonical.clone(),
        source_research_run_id: candidate.source_research_run_id.clone(),
        target_max_holding_hours: candidate.target_max_holding_hours,
        absolute_max_holding_hours: candidate.absolute_max_holding_hours,
        holding_elapsed_ms,
        mark_count: marks.len(),
        venues: marks
            .iter()
            .map(|mark| mark.venue.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect(),
        latest_return_bps,
        min_return_bps,
        max_return_bps,
        max_drawdown_bps: max_drawdown_bps(&returns),
        lifecycle_state: lifecycle_state.clone(),
        observer_verdict: observer_verdict(
            marks.len(),
            latest_return_bps,
            min_return_bps,
            max_return_bps,
            &lifecycle_state,
        ),
        safety: PaperWatchSafety {
            paper_only: true,
            live_enabled: false,
            order_execution_enabled: false,
            execution_approval_emitted: false,
        },
    }
}

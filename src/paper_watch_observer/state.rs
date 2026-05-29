use super::summary::build_snapshot;
use super::types::PaperWatchObserverSnapshot;
use crate::model::{MarketLiveTick, PaperWatchCandidate, PaperWatchLiveMark};
use crate::paper_live::{PaperWatchLiveEntryBook, build_paper_watch_live_marks_with_entry_book};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Default)]
pub struct PaperWatchObserverState {
    entry_book: PaperWatchLiveEntryBook,
    seen_live_mark_ids: BTreeSet<String>,
    marks_by_candidate: BTreeMap<String, Vec<PaperWatchLiveMark>>,
}

impl PaperWatchObserverState {
    pub fn restore_marks(&mut self, marks: &[PaperWatchLiveMark]) {
        let mut ordered_marks = marks.to_vec();
        ordered_marks.sort_by(|left, right| {
            (
                left.exchange_timestamp_ms,
                left.ingest_timestamp_ms,
                left.paper_watch_live_mark_id.as_str(),
            )
                .cmp(&(
                    right.exchange_timestamp_ms,
                    right.ingest_timestamp_ms,
                    right.paper_watch_live_mark_id.as_str(),
                ))
        });
        for mark in ordered_marks {
            self.restore_entry_from_mark(&mark);
            if self
                .seen_live_mark_ids
                .insert(mark.paper_watch_live_mark_id.clone())
            {
                self.marks_by_candidate
                    .entry(mark.paper_watch_candidate_id.clone())
                    .or_default()
                    .push(mark);
            }
        }
    }

    pub fn ingest_ticks(
        &mut self,
        candidates: &[PaperWatchCandidate],
        ticks: &[MarketLiveTick],
    ) -> Vec<PaperWatchLiveMark> {
        let marks =
            build_paper_watch_live_marks_with_entry_book(candidates, ticks, &mut self.entry_book);
        let mut new_marks = Vec::new();
        for mark in marks {
            if !self
                .seen_live_mark_ids
                .insert(mark.paper_watch_live_mark_id.clone())
            {
                continue;
            }
            self.marks_by_candidate
                .entry(mark.paper_watch_candidate_id.clone())
                .or_default()
                .push(mark.clone());
            new_marks.push(mark);
        }
        new_marks
    }

    pub fn snapshot(
        &self,
        observer_run_id: &str,
        iteration: usize,
        created_at_ms: i64,
        candidates: &[PaperWatchCandidate],
        new_marks: &[PaperWatchLiveMark],
    ) -> PaperWatchObserverSnapshot {
        build_snapshot(
            observer_run_id,
            iteration,
            created_at_ms,
            candidates,
            new_marks,
            &self.marks_by_candidate,
            self.seen_live_mark_ids.len(),
        )
    }

    fn restore_entry_from_mark(&mut self, mark: &PaperWatchLiveMark) {
        let quote_asset = mark
            .reason_codes
            .iter()
            .find_map(|reason| reason.strip_prefix("quote_asset="))
            .unwrap_or("");
        self.entry_book.restore_entry(
            &mark.paper_watch_candidate_id,
            &mark.venue,
            quote_asset,
            mark.entry_mark_price,
        );
    }
}

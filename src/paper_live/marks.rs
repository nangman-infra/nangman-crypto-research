use self::entry_book::paper_watch_market_entry_key;
use self::index::index_safe_candidates_by_symbol;
use self::mark::build_mark;
use self::symbol::normalize_symbol;
use crate::model::{
    MARKET_LIVE_TICK_SCHEMA_VERSION, MarketLiveTick, PaperWatchCandidate, PaperWatchLiveMark,
};
use crate::paper_live::price::valid_mark_price;

mod entry_book;
mod index;
mod lifecycle;
mod mark;
mod symbol;

pub use entry_book::PaperWatchLiveEntryBook;

pub fn build_paper_watch_live_marks(
    candidates: &[PaperWatchCandidate],
    ticks: &[MarketLiveTick],
) -> Vec<PaperWatchLiveMark> {
    let mut entry_book = PaperWatchLiveEntryBook::default();
    build_paper_watch_live_marks_with_entry_book(candidates, ticks, &mut entry_book)
}

pub fn build_paper_watch_live_marks_with_entry_book(
    candidates: &[PaperWatchCandidate],
    ticks: &[MarketLiveTick],
    entry_book: &mut PaperWatchLiveEntryBook,
) -> Vec<PaperWatchLiveMark> {
    let candidates_by_symbol = index_safe_candidates_by_symbol(candidates);
    let mut ordered_ticks = ticks.to_vec();
    ordered_ticks.sort_by(|left, right| {
        (
            left.exchange_timestamp_ms,
            left.ingest_timestamp_ms,
            left.event_id.as_str(),
        )
            .cmp(&(
                right.exchange_timestamp_ms,
                right.ingest_timestamp_ms,
                right.event_id.as_str(),
            ))
    });

    let mut marks = Vec::new();
    for tick in &ordered_ticks {
        if tick.schema_version != MARKET_LIVE_TICK_SCHEMA_VERSION {
            continue;
        }
        let Some(current_price) = valid_mark_price(tick.mark_price) else {
            continue;
        };
        let Some(matched_candidates) =
            candidates_by_symbol.get(&normalize_symbol(&tick.symbol_canonical))
        else {
            continue;
        };
        for candidate in matched_candidates {
            let entry_key = paper_watch_market_entry_key(candidate, tick);
            let entry_price = entry_book.entry_price_or_insert(entry_key, current_price);
            marks.push(build_mark(candidate, tick, entry_price, current_price));
        }
    }
    marks
}

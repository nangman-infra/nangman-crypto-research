use crate::error::AppResult;
use crate::model::{MarketLiveTick, PaperWatchCandidate, PaperWatchLiveMark};

use super::super::json::read_json_array_or_jsonl_bytes;

pub fn read_paper_watch_candidates_from_bytes(
    label: &str,
    bytes: &[u8],
) -> AppResult<Vec<PaperWatchCandidate>> {
    read_json_array_or_jsonl_bytes(label, bytes)
}

pub fn read_market_live_ticks_from_bytes(
    label: &str,
    bytes: &[u8],
) -> AppResult<Vec<MarketLiveTick>> {
    read_json_array_or_jsonl_bytes(label, bytes)
}

pub fn read_paper_watch_live_marks_from_bytes(
    label: &str,
    bytes: &[u8],
) -> AppResult<Vec<PaperWatchLiveMark>> {
    read_json_array_or_jsonl_bytes(label, bytes)
}

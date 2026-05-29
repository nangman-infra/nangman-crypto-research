use super::super::price::valid_mark_price;
use crate::model::{MarketLiveTick, PaperWatchCandidate};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PaperWatchLiveEntryBook {
    entry_by_watch_candidate_market: BTreeMap<String, f64>,
}

impl PaperWatchLiveEntryBook {
    pub fn restore_entry(
        &mut self,
        paper_watch_candidate_id: &str,
        venue: &str,
        quote_asset: &str,
        entry_price: f64,
    ) {
        if valid_mark_price(Some(entry_price)).is_none() {
            return;
        }
        let key =
            paper_watch_market_entry_key_from_parts(paper_watch_candidate_id, venue, quote_asset);
        self.entry_by_watch_candidate_market
            .entry(key)
            .or_insert(entry_price);
    }

    pub(super) fn entry_price_or_insert(&mut self, key: String, current_price: f64) -> f64 {
        *self
            .entry_by_watch_candidate_market
            .entry(key)
            .or_insert(current_price)
    }
}

pub(super) fn paper_watch_market_entry_key(
    candidate: &PaperWatchCandidate,
    tick: &MarketLiveTick,
) -> String {
    paper_watch_market_entry_key_from_parts(
        &candidate.paper_watch_candidate_id,
        &tick.venue,
        &tick.quote_asset,
    )
}

fn paper_watch_market_entry_key_from_parts(
    paper_watch_candidate_id: &str,
    venue: &str,
    quote_asset: &str,
) -> String {
    format!(
        "{}:{}:{}",
        paper_watch_candidate_id,
        venue.to_ascii_lowercase(),
        quote_asset.to_ascii_uppercase()
    )
}

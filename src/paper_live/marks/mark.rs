use super::lifecycle::lifecycle_state;
use crate::hash::stable_id;
use crate::model::{
    MarketLiveTick, PAPER_WATCH_LIVE_MARK_SCHEMA_VERSION, PaperWatchCandidate, PaperWatchLiveMark,
    PaperWatchSafety,
};

pub(super) fn build_mark(
    candidate: &PaperWatchCandidate,
    tick: &MarketLiveTick,
    entry_price: f64,
    current_price: f64,
) -> PaperWatchLiveMark {
    let holding_elapsed_ms = tick
        .exchange_timestamp_ms
        .saturating_sub(candidate.created_at_ms)
        .max(0);
    let lifecycle_state = lifecycle_state(candidate, holding_elapsed_ms);
    let net_return_bps = ((current_price / entry_price) - 1.0) * 10_000.0;
    let mut reason_codes = vec![
        "paper_watch_live_mark".to_owned(),
        "paper_only_no_order_execution".to_owned(),
        format!("venue={}", tick.venue),
        format!("quote_asset={}", tick.quote_asset),
        format!("price_source={}", tick.price_source),
    ];
    if lifecycle_state != "watching" {
        reason_codes.push(lifecycle_state.clone());
    }

    PaperWatchLiveMark {
        paper_watch_live_mark_id: stable_id(
            "paper_watch_live_mark",
            &[&candidate.paper_watch_candidate_id, &tick.event_id],
        ),
        paper_watch_candidate_id: candidate.paper_watch_candidate_id.clone(),
        candidate_id: candidate.candidate_id.clone(),
        candidate_lifecycle_key: candidate.candidate_lifecycle_key.clone(),
        symbol_canonical: candidate.symbol_canonical.clone(),
        source_research_run_id: candidate.source_research_run_id.clone(),
        source_market_live_event_id: tick.event_id.clone(),
        venue: tick.venue.clone(),
        mark_source: "market_live_tick".to_owned(),
        marked_at_ms: tick.ingest_timestamp_ms.max(tick.exchange_timestamp_ms),
        exchange_timestamp_ms: tick.exchange_timestamp_ms,
        ingest_timestamp_ms: tick.ingest_timestamp_ms,
        holding_elapsed_ms,
        entry_mark_price: entry_price,
        current_mark_price: current_price,
        net_return_bps,
        target_max_holding_hours: candidate.target_max_holding_hours,
        absolute_max_holding_hours: candidate.absolute_max_holding_hours,
        lifecycle_state,
        reason_codes,
        safety: PaperWatchSafety {
            paper_only: true,
            live_enabled: false,
            order_execution_enabled: false,
            execution_approval_emitted: false,
        },
        schema_version: PAPER_WATCH_LIVE_MARK_SCHEMA_VERSION.to_owned(),
    }
}

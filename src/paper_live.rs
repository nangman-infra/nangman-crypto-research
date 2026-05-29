mod config;
mod io;
mod marks;
mod nats;
mod price;

pub use config::{
    DEFAULT_MARKET_LIVE_NATS_ACK_WAIT_SECS, DEFAULT_MARKET_LIVE_NATS_BATCH_SIZE,
    DEFAULT_MARKET_LIVE_NATS_CONSUMER, DEFAULT_MARKET_LIVE_NATS_DELIVER_POLICY,
    DEFAULT_MARKET_LIVE_NATS_MAX_MESSAGES, DEFAULT_MARKET_LIVE_NATS_STREAM,
    DEFAULT_MARKET_LIVE_NATS_SUBJECT, MarketLiveNatsConfig,
};
pub use io::{read_market_live_ticks, read_paper_watch_candidates};
pub use marks::{
    PaperWatchLiveEntryBook, build_paper_watch_live_marks,
    build_paper_watch_live_marks_with_entry_book,
};
pub use nats::read_market_live_ticks_from_nats;

#[cfg(test)]
mod tests;

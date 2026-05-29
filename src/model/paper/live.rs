use super::watch::PaperWatchSafety;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct MarketLiveTick {
    pub schema_version: String,
    pub event_id: String,
    pub producer_run_id: String,
    pub venue: String,
    pub source_role: String,
    pub market_type: String,
    pub event_type: String,
    pub symbol_native: String,
    pub symbol_canonical: String,
    pub base_asset: String,
    pub quote_asset: String,
    pub exchange_timestamp_ms: i64,
    pub ingest_timestamp_ms: i64,
    pub latency_ms: i64,
    pub sequence_id: String,
    pub sequence_tag: String,
    pub price_source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub best_bid_price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub best_ask_price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mark_price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(alias = "trade_volume")]
    pub quantity: Option<f64>,
    #[serde(alias = "payload_sha256")]
    pub raw_payload_sha256: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PaperWatchLiveMark {
    pub paper_watch_live_mark_id: String,
    pub paper_watch_candidate_id: String,
    pub candidate_id: String,
    pub candidate_lifecycle_key: String,
    pub symbol_canonical: String,
    pub source_research_run_id: String,
    pub source_market_live_event_id: String,
    pub venue: String,
    pub mark_source: String,
    pub marked_at_ms: i64,
    pub exchange_timestamp_ms: i64,
    pub ingest_timestamp_ms: i64,
    pub holding_elapsed_ms: i64,
    pub entry_mark_price: f64,
    pub current_mark_price: f64,
    pub net_return_bps: f64,
    pub target_max_holding_hours: u32,
    pub absolute_max_holding_hours: u32,
    pub lifecycle_state: String,
    pub reason_codes: Vec<String>,
    pub safety: PaperWatchSafety,
    pub schema_version: String,
}

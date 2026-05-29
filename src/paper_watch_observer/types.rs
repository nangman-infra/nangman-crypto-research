use crate::model::PaperWatchSafety;
use serde::Serialize;
use std::collections::BTreeMap;

pub const PAPER_WATCH_OBSERVER_SNAPSHOT_SCHEMA_VERSION: &str = "paper_watch_observer_snapshot_v1";

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PaperWatchObserverSnapshot {
    pub schema_version: String,
    pub observer_run_id: String,
    pub iteration: usize,
    pub created_at_ms: i64,
    pub active_candidate_count: usize,
    pub active_symbols: Vec<String>,
    pub restored_live_mark_count: usize,
    pub new_live_mark_count: usize,
    pub total_live_mark_count: usize,
    pub lifecycle_counts: BTreeMap<String, usize>,
    pub venue_counts: BTreeMap<String, usize>,
    pub net_return_bps: PaperWatchObserverReturnSummary,
    pub candidate_summaries: Vec<PaperWatchObserverCandidateSummary>,
    pub safety: PaperWatchObserverSafety,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PaperWatchObserverCandidateSummary {
    pub paper_watch_candidate_id: String,
    pub candidate_id: String,
    pub candidate_lifecycle_key: String,
    pub symbol_canonical: String,
    pub source_research_run_id: String,
    pub target_max_holding_hours: u32,
    pub absolute_max_holding_hours: u32,
    pub holding_elapsed_ms: i64,
    pub mark_count: usize,
    pub venues: Vec<String>,
    pub latest_return_bps: Option<f64>,
    pub min_return_bps: Option<f64>,
    pub max_return_bps: Option<f64>,
    pub max_drawdown_bps: Option<f64>,
    pub lifecycle_state: String,
    pub observer_verdict: String,
    pub safety: PaperWatchSafety,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PaperWatchObserverReturnSummary {
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub average: Option<f64>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PaperWatchObserverSafety {
    pub paper_only: bool,
    pub live_enabled: bool,
    pub order_execution_enabled: bool,
    pub execution_approval_emitted: bool,
}

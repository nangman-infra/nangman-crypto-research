use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FocusedRetestRow {
    pub candidate_id: String,
    pub candidate_lifecycle_key: Option<String>,
    pub symbol: String,
    pub symbols: Vec<String>,
    pub hypothesis_type: Option<String>,
    pub research_priority: Option<String>,
    pub horizon: String,
    pub next_action: String,
    pub replay_run_count: Option<i64>,
    pub completed_count: Option<i64>,
    pub completed_sample_deficit: Option<i64>,
    pub inferred_unseen_window_count: Option<i64>,
    pub unseen_window_deficit: Option<i64>,
    pub reason_codes: Vec<String>,
}

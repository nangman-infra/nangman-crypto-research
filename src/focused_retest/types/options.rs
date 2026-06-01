use super::replay_index_ref_mode::HistoricalReplayIndexRefMode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusedRetestBuildOptions {
    pub generated_at_ms: i64,
    pub research_packet_id: String,
    pub run_scope: String,
    pub next_actions: Vec<String>,
    pub candidate_lifecycle_key_filter: Vec<String>,
    pub historical_replay_index_ref_mode: HistoricalReplayIndexRefMode,
    pub s3_write: bool,
}

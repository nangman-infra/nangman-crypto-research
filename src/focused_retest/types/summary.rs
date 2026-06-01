use super::row::FocusedRetestRow;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FocusedRetestManifestSummary {
    pub schema_version: String,
    pub generated_at_ms: i64,
    pub focus_next_actions: Vec<String>,
    pub safety: FocusedRetestManifestSafety,
    pub source: FocusedRetestManifestSourceSummary,
    pub focused: FocusedRetestSelectionSummary,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FocusedRetestManifestSafety {
    pub s3_write: bool,
    pub ecs_task_started: bool,
    pub dispatcher_mode_changed: bool,
    pub shadow_paper_live_enabled: bool,
    pub selected_from_existing_retest_status: bool,
    pub historical_replay_run_index_ref_mode: String,
    pub historical_replay_run_index_refs_carried: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FocusedRetestManifestSourceSummary {
    pub research_packet_id: Option<String>,
    pub run_scope: Option<String>,
    pub candidate_bundle_ref_count: usize,
    pub historical_replay_run_index_ref_count: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FocusedRetestSelectionSummary {
    pub focus_horizon_count: usize,
    pub focus_candidate_count: usize,
    pub selected_candidate_bundle_ref_count: usize,
    pub selected_historical_replay_run_index_ref_count: usize,
    pub symbols: Vec<String>,
    pub next_action_counts: Vec<FocusedRetestActionCount>,
    pub horizons: Vec<FocusedRetestHorizonCount>,
    pub selected_candidate_ids: Vec<String>,
    pub missing_candidate_ref_ids: Vec<String>,
    pub rows: Vec<FocusedRetestRow>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FocusedRetestActionCount {
    pub next_action: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FocusedRetestHorizonCount {
    pub horizon: String,
    pub count: usize,
}

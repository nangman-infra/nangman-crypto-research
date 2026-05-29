use crate::error::{AppError, AppResult};
use crate::model::ResearchInputManifest;
use serde::Serialize;

pub const DEFAULT_FOCUSED_RETEST_ACTIONS: &[&str] = &[
    "run_research_replay_for_horizon",
    "accumulate_completed_native_replay_samples",
    "materialize_completed_native_replay_sample",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoricalReplayIndexRefMode {
    Auto,
    Always,
    Never,
}

impl HistoricalReplayIndexRefMode {
    pub fn parse(raw: &str) -> AppResult<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "auto" => Ok(Self::Auto),
            "true" | "always" => Ok(Self::Always),
            "false" | "never" => Ok(Self::Never),
            other => Err(AppError::config(format!(
                "focused retest historical replay index ref mode must be auto, true, or false; got {other}"
            ))),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Always => "true",
            Self::Never => "false",
        }
    }

    pub(super) fn should_carry(self, actions: &[String]) -> bool {
        match self {
            Self::Always => true,
            Self::Never => false,
            Self::Auto => actions
                .iter()
                .any(|action| action == "accumulate_completed_native_replay_samples"),
        }
    }
}

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

#[derive(Debug)]
pub struct FocusedRetestManifestBuild {
    pub manifest: ResearchInputManifest,
    pub summary: FocusedRetestManifestSummary,
}

pub fn default_focused_retest_actions() -> Vec<String> {
    DEFAULT_FOCUSED_RETEST_ACTIONS
        .iter()
        .map(|action| (*action).to_owned())
        .collect()
}

pub fn parse_focused_retest_actions(raw: &str) -> Vec<String> {
    let mut actions = raw
        .split(',')
        .map(str::trim)
        .filter(|action| !action.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    actions.sort();
    actions.dedup();
    actions
}

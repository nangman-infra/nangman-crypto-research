use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ShadowCycleDecision {
    pub schema_version: String,
    pub generated_at: String,
    pub decision_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_cycle_summary_file: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_dir: Option<String>,
    pub scheduler_action: ShadowCycleSchedulerAction,
    pub source_verdict: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_not_before_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_not_before_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_not_before_source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focused_research_manifest_file: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focused_research_summary_file: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_l1_as_of_ms: Option<i64>,
    pub shadow_sample_state: ShadowCycleSampleState,
    #[serde(default)]
    pub safe_next_actions: Vec<String>,
    #[serde(default)]
    pub blocked_actions: Vec<String>,
    pub safety: ShadowCycleDecisionSafety,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ShadowCycleSchedulerAction {
    DiscoverMarketL1Watermark,
    WaitUntilTargetWindowMaterializes,
    WaitUntilPendingShadowTargetWindowMaterializes,
    RunFocusedShadowSampleAccumulationResearch,
    ReviewShadowCompletionEvidence,
    Noop,
    HoldForOperatorReview,
}

impl ShadowCycleSchedulerAction {
    pub fn is_wait_action(&self) -> bool {
        matches!(
            self,
            Self::WaitUntilTargetWindowMaterializes
                | Self::WaitUntilPendingShadowTargetWindowMaterializes
        )
    }

    pub fn requires_focused_research_manifest(&self) -> bool {
        matches!(self, Self::RunFocusedShadowSampleAccumulationResearch)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ShadowCycleSampleState {
    pub shadow_validation_count: usize,
    pub target_window_materialized_count: usize,
    pub candidate_lifecycle_count: usize,
    pub partially_materialized_candidate_count: usize,
    pub pending_target_window_candidate_count: usize,
    pub total_sample_deficit: i64,
    #[serde(default)]
    pub symbols: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ShadowCycleDecisionSafety {
    pub s3_write: bool,
    pub ecs_task_started: bool,
    pub dispatcher_mode_changed: bool,
    pub local_decision_only: bool,
    pub shadow_status_mutated: bool,
    pub paper_live_enabled: bool,
    pub live_enabled: bool,
    pub order_execution_enabled: bool,
}

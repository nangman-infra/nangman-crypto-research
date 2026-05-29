use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ResearchInputManifest {
    pub schema_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub research_packet_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_scope: Option<String>,
    #[serde(default)]
    pub candidate_bundle_refs: Vec<ResearchArtifactRef>,
    #[serde(default)]
    pub market_feature_delta_refs: Vec<ResearchArtifactRef>,
    #[serde(default)]
    pub market_regime_context_refs: Vec<ResearchArtifactRef>,
    #[serde(default)]
    pub shadow_validation_run_refs: Vec<ResearchArtifactRef>,
    #[serde(default)]
    pub hypothesis_harness_result_refs: Vec<ResearchArtifactRef>,
    #[serde(default)]
    pub oss_adapter_run_refs: Vec<ResearchArtifactRef>,
    #[serde(default)]
    pub historical_replay_run_refs: Vec<ResearchArtifactRef>,
    #[serde(default)]
    pub historical_replay_run_index_refs: Vec<ResearchArtifactRef>,
    #[serde(default)]
    pub runtime_budget_policy: ResearchRuntimeBudgetPolicy,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ResearchArtifactRef {
    pub uri: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct RetestCycleSourceState {
    pub schema_version: String,
    pub generated_at_ms: i64,
    pub research_packet_id: String,
    pub run_scope: String,
    pub source_manifest_s3_bucket: String,
    pub source_manifest_s3_key: String,
    pub source_research_report_s3_bucket: String,
    pub source_research_report_s3_key: String,
    pub source_research_report_id: String,
    pub source_candidate_ids: Vec<String>,
    pub replay_run_id_count: usize,
    pub summary_findings_count: usize,
    pub shadow_validation_run_count: usize,
    pub paper_trade_candidate_count: usize,
    pub safety: RetestCycleSourceStateSafety,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct RetestCycleSourceStateSafety {
    pub dispatcher_prefix: String,
    pub state_s3_write: bool,
    pub ecs_task_started: bool,
    pub shadow_paper_live_enabled: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ResearchRuntimeBudgetPolicy {
    #[serde(default = "default_max_candidate_bundle_count")]
    pub max_candidate_bundle_count: usize,
    #[serde(default = "default_max_market_artifact_ref_count")]
    pub max_market_artifact_ref_count: usize,
    #[serde(default = "default_max_shadow_validation_run_ref_count")]
    pub max_shadow_validation_run_ref_count: usize,
    #[serde(default = "default_max_hypothesis_harness_result_ref_count")]
    pub max_hypothesis_harness_result_ref_count: usize,
    #[serde(default = "default_max_oss_adapter_run_ref_count")]
    pub max_oss_adapter_run_ref_count: usize,
    #[serde(default = "default_max_historical_replay_run_ref_count")]
    pub max_historical_replay_run_ref_count: usize,
    #[serde(default = "default_max_replay_run_count")]
    pub max_replay_run_count: usize,
}

impl Default for ResearchRuntimeBudgetPolicy {
    fn default() -> Self {
        Self {
            max_candidate_bundle_count: default_max_candidate_bundle_count(),
            max_market_artifact_ref_count: default_max_market_artifact_ref_count(),
            max_shadow_validation_run_ref_count: default_max_shadow_validation_run_ref_count(),
            max_hypothesis_harness_result_ref_count:
                default_max_hypothesis_harness_result_ref_count(),
            max_oss_adapter_run_ref_count: default_max_oss_adapter_run_ref_count(),
            max_historical_replay_run_ref_count: default_max_historical_replay_run_ref_count(),
            max_replay_run_count: default_max_replay_run_count(),
        }
    }
}

fn default_max_candidate_bundle_count() -> usize {
    500
}

fn default_max_market_artifact_ref_count() -> usize {
    2_000
}

fn default_max_shadow_validation_run_ref_count() -> usize {
    10_000
}

fn default_max_hypothesis_harness_result_ref_count() -> usize {
    10_000
}

fn default_max_oss_adapter_run_ref_count() -> usize {
    10_000
}

fn default_max_historical_replay_run_ref_count() -> usize {
    10_000
}

fn default_max_replay_run_count() -> usize {
    20_000
}

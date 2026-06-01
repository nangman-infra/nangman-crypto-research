use super::{ResearchArtifactRef, ResearchRuntimeBudgetPolicy};
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

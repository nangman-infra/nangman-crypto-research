use serde::{Deserialize, Serialize};

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

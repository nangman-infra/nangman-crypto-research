use super::super::types::FocusedRetestBuildOptions;
use super::candidate_refs::SourceCandidateRef;
use crate::model::{ResearchArtifactRef, ResearchInputManifest};

pub(super) fn build_manifest(
    source_manifest: &ResearchInputManifest,
    options: &FocusedRetestBuildOptions,
    selected_refs: &[SourceCandidateRef],
    carry_historical_index_refs: bool,
) -> ResearchInputManifest {
    let historical_replay_run_index_refs = if carry_historical_index_refs {
        source_manifest.historical_replay_run_index_refs.clone()
    } else {
        Vec::new()
    };

    let mut runtime_budget_policy = source_manifest.runtime_budget_policy.clone();
    runtime_budget_policy.max_candidate_bundle_count = selected_refs.len().max(1);

    ResearchInputManifest {
        schema_version: source_manifest.schema_version.clone(),
        research_packet_id: Some(options.research_packet_id.clone()),
        run_scope: Some(options.run_scope.clone()),
        candidate_bundle_refs: selected_refs
            .iter()
            .map(|candidate_ref| ResearchArtifactRef {
                uri: candidate_ref.uri.clone(),
            })
            .collect(),
        market_feature_delta_refs: Vec::new(),
        market_regime_context_refs: Vec::new(),
        shadow_validation_run_refs: Vec::new(),
        hypothesis_harness_result_refs: Vec::new(),
        oss_adapter_run_refs: Vec::new(),
        historical_replay_run_refs: Vec::new(),
        historical_replay_run_index_refs,
        runtime_budget_policy,
    }
}

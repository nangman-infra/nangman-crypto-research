use crate::model::{
    IntelCandidateEvidenceBundle, OssAdapterRun, OssAdapterVerdictBias, ReplayRun, ResearchBias,
    ResearchPartitionAggregate, ShadowValidationRun, SummaryFinding,
};
use crate::paper::is_completed_passed_shadow;
use std::collections::BTreeSet;

pub(super) fn candidate_findings(
    bundles: &[IntelCandidateEvidenceBundle],
    replay_runs: &[ReplayRun],
    partition_aggregates: &[ResearchPartitionAggregate],
    oss_adapter_runs: &[OssAdapterRun],
    completed_shadow_validation_runs: &[ShadowValidationRun],
) -> Vec<SummaryFinding> {
    let passed_shadow_candidate_keys = completed_shadow_validation_runs
        .iter()
        .filter(|run| is_completed_passed_shadow(run))
        .map(|run| run.candidate_lifecycle_key.clone())
        .collect::<BTreeSet<_>>();
    bundles
        .iter()
        .map(|bundle| {
            let candidate_runs = replay_runs
                .iter()
                .filter(|run| run.source_candidate_id == bundle.candidate_id)
                .collect::<Vec<_>>();
            let candidate_oss_runs = oss_adapter_runs
                .iter()
                .filter(|run| run.candidate_lifecycle_key == bundle.candidate_lifecycle_key)
                .collect::<Vec<_>>();
            let bias = if candidate_oss_runs
                .iter()
                .any(|run| run.normalized_verdict_bias == OssAdapterVerdictBias::PruneBias)
                || candidate_runs
                    .iter()
                    .any(|run| run.result_summary.bias == ResearchBias::PruneBias)
                || candidate_aggregates(bundle, partition_aggregates)
                    .iter()
                    .any(|aggregate| aggregate.gate_bias == ResearchBias::PruneBias)
            {
                ResearchBias::PruneBias
            } else if passed_shadow_candidate_keys.contains(&bundle.candidate_lifecycle_key)
                && candidate_aggregates(bundle, partition_aggregates)
                    .iter()
                    .any(|aggregate| aggregate.gate_bias == ResearchBias::PromoteToShadowBias)
            {
                ResearchBias::PromoteToPaperBias
            } else if candidate_aggregates(bundle, partition_aggregates)
                .iter()
                .any(|aggregate| aggregate.gate_bias == ResearchBias::PromoteToShadowBias)
            {
                ResearchBias::PromoteToShadowBias
            } else {
                ResearchBias::RetestBias
            };
            let mut reason_codes = candidate_runs
                .iter()
                .flat_map(|run| run.result_summary.reason_codes.clone())
                .chain(
                    candidate_aggregates(bundle, partition_aggregates)
                        .into_iter()
                        .flat_map(|aggregate| aggregate.gate_reason_codes.clone()),
                )
                .chain(oss_reason_codes(&candidate_oss_runs))
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            if bias == ResearchBias::PromoteToPaperBias {
                reason_codes.push("shadow_validation_passed_for_paper".to_owned());
            }
            SummaryFinding {
                candidate_id: bundle.candidate_id.clone(),
                candidate_lifecycle_key: bundle.candidate_lifecycle_key.clone(),
                bias,
                reason_codes,
            }
        })
        .collect()
}

fn oss_reason_codes(candidate_oss_runs: &[&OssAdapterRun]) -> Vec<String> {
    let mut reasons = Vec::new();
    for run in candidate_oss_runs {
        match run.normalized_verdict_bias {
            OssAdapterVerdictBias::PruneBias => reasons.push("oss_adapter_prune_bias".to_owned()),
            OssAdapterVerdictBias::RetestBias => reasons.push("oss_adapter_retest_bias".to_owned()),
            OssAdapterVerdictBias::PromoteToReplayBias => {
                reasons.push("oss_adapter_promote_to_replay_bias_requires_native_gate".to_owned())
            }
        }
        reasons.extend(run.adapter_warnings.clone());
    }
    reasons
}

fn candidate_aggregates<'a>(
    bundle: &IntelCandidateEvidenceBundle,
    partition_aggregates: &'a [ResearchPartitionAggregate],
) -> Vec<&'a ResearchPartitionAggregate> {
    partition_aggregates
        .iter()
        .filter(|aggregate| {
            aggregate
                .source_candidate_lifecycle_keys
                .iter()
                .any(|key| key == &bundle.candidate_lifecycle_key)
        })
        .collect()
}

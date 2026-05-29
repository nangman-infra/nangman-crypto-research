use crate::hash::stable_id;
use crate::model::{
    RESEARCH_AGGREGATE_REGISTRY_SCHEMA_VERSION, ResearchAggregateRegistryRecord,
    ResearchAggregateRegistryStage, ResearchBias, ResearchPartitionAggregate, ResearchRunReport,
    ShadowValidationRun,
};
use std::collections::{BTreeMap, BTreeSet};

pub fn build_research_aggregate_registry_records(
    report: &ResearchRunReport,
) -> Vec<ResearchAggregateRegistryRecord> {
    let shadow_run_ids_by_aggregate_key =
        shadow_run_ids_by_aggregate_key(&report.shadow_validation_runs);
    let paper_candidate_keys = paper_candidate_keys(report);

    report
        .partition_aggregates
        .iter()
        .map(|aggregate| {
            let research_aggregate_registry_record_id = stable_id(
                "research_aggregate_registry",
                &[
                    &report.research_run_report_id,
                    &aggregate.research_aggregate_key,
                    aggregate.gate_bias.report_key(),
                ],
            );
            ResearchAggregateRegistryRecord {
                research_aggregate_registry_record_id,
                research_run_report_id: report.research_run_report_id.clone(),
                research_packet_id: report.research_packet_id.clone(),
                run_scope: report.run_scope.clone(),
                research_aggregate_key: aggregate.research_aggregate_key.clone(),
                source_candidate_ids: aggregate.source_candidate_ids.clone(),
                source_candidate_lifecycle_keys: aggregate.source_candidate_lifecycle_keys.clone(),
                symbol_canonical: aggregate.symbol_canonical.clone(),
                hypothesis_type: aggregate.hypothesis_type.clone(),
                validation_adapter: aggregate.validation_adapter.clone(),
                strategy_id_or_family: aggregate.strategy_id_or_family.clone(),
                parameter_variant_id: aggregate.parameter_variant_id.clone(),
                current_research_stage: stage_for_aggregate(
                    &aggregate.gate_bias,
                    aggregate,
                    &paper_candidate_keys,
                ),
                gate_bias: aggregate.gate_bias.clone(),
                survival_band: aggregate.survival_band.clone(),
                replay_run_count: aggregate.replay_run_count,
                active_replay_run_count: aggregate.active_replay_run_count,
                expired_replay_run_count: aggregate.expired_replay_run_count,
                completed_count: aggregate.completed_count,
                effective_completed_sample_weight: aggregate.effective_completed_sample_weight,
                positive_net_count: aggregate.positive_net_count,
                non_positive_net_count: aggregate.non_positive_net_count,
                weighted_win_rate_ppm: aggregate.weighted_win_rate_ppm,
                weighted_mean_net_after_cost_bps: aggregate.weighted_mean_net_after_cost_bps,
                weighted_profit_factor_ppm: aggregate.weighted_profit_factor_ppm,
                cost_stressed_mean_net_after_cost_bps: aggregate
                    .cost_stressed_mean_net_after_cost_bps,
                market_regime_labels: aggregate.market_regime_labels.clone(),
                latest_reason_codes: aggregate.gate_reason_codes.clone(),
                linked_shadow_validation_run_ids: shadow_run_ids_by_aggregate_key
                    .get(&aggregate.research_aggregate_key)
                    .cloned()
                    .unwrap_or_default(),
                created_at_ms: report.created_at_ms,
                schema_version: RESEARCH_AGGREGATE_REGISTRY_SCHEMA_VERSION.to_owned(),
            }
        })
        .collect()
}

fn paper_candidate_keys(report: &ResearchRunReport) -> BTreeSet<String> {
    report
        .summary_findings
        .iter()
        .filter(|finding| finding.bias == ResearchBias::PromoteToPaperBias)
        .map(|finding| finding.candidate_lifecycle_key.clone())
        .collect()
}

fn shadow_run_ids_by_aggregate_key(
    shadow_validation_runs: &[ShadowValidationRun],
) -> BTreeMap<String, Vec<String>> {
    let mut values = BTreeMap::<String, Vec<String>>::new();
    for run in shadow_validation_runs {
        values
            .entry(run.start_condition_summary.research_aggregate_key.clone())
            .or_default()
            .push(run.shadow_validation_run_id.clone());
    }
    values
}

fn stage_for_bias(bias: &ResearchBias) -> ResearchAggregateRegistryStage {
    match bias {
        ResearchBias::PruneBias => ResearchAggregateRegistryStage::Pruned,
        ResearchBias::RetestBias => ResearchAggregateRegistryStage::Retest,
        ResearchBias::PromoteToShadowBias => ResearchAggregateRegistryStage::ShadowCandidate,
        ResearchBias::PromoteToPaperBias => ResearchAggregateRegistryStage::PaperCandidateBias,
    }
}

fn stage_for_aggregate(
    bias: &ResearchBias,
    aggregate: &ResearchPartitionAggregate,
    paper_candidate_keys: &BTreeSet<String>,
) -> ResearchAggregateRegistryStage {
    if aggregate
        .source_candidate_lifecycle_keys
        .iter()
        .any(|key| paper_candidate_keys.contains(key))
    {
        return ResearchAggregateRegistryStage::PaperCandidateBias;
    }
    stage_for_bias(bias)
}

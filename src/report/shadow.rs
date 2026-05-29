use crate::gate::default_research_gate_policy;
use crate::hash::stable_id;
use crate::holding::default_holding_policy;
use crate::model::{
    IntelCandidateEvidenceBundle, PAPER_TRADE_CANDIDATE_SCHEMA_VERSION, ResearchBias,
    ResearchPartitionAggregate, SHADOW_VALIDATION_RUN_SCHEMA_VERSION, ShadowStartConditionSummary,
    ShadowTerminationPolicy, ShadowValidationRun, ShadowValidationStatus, ShadowWatchWindowPolicy,
    SummaryFinding,
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn shadow_validation_run_ids(
    research_packet_id: &str,
    research_run_report_id: &str,
    run_scope: &str,
    partition_aggregates: &[ResearchPartitionAggregate],
    summary_findings: &[SummaryFinding],
    bundles: &[IntelCandidateEvidenceBundle],
) -> Vec<ShadowValidationRun> {
    let promotable_candidate_keys = summary_findings
        .iter()
        .filter(|finding| finding.bias == ResearchBias::PromoteToShadowBias)
        .map(|finding| finding.candidate_lifecycle_key.clone())
        .collect::<BTreeSet<_>>();
    let decision_time_by_candidate_key = bundles
        .iter()
        .map(|bundle| {
            (
                bundle.candidate_lifecycle_key.clone(),
                bundle.decision_available_at_ms,
            )
        })
        .collect::<BTreeMap<_, _>>();

    partition_aggregates
        .iter()
        .filter(|aggregate| aggregate.gate_bias == ResearchBias::PromoteToShadowBias)
        .flat_map(|aggregate| {
            aggregate
                .source_candidate_lifecycle_keys
                .iter()
                .filter(|candidate_lifecycle_key| {
                    promotable_candidate_keys.contains(candidate_lifecycle_key.as_str())
                })
                .map(|candidate_lifecycle_key| {
                    let shadow_validation_run_id = stable_id(
                        "shadow_validation",
                        &[
                            research_packet_id,
                            run_scope,
                            &aggregate.research_aggregate_key,
                            candidate_lifecycle_key,
                        ],
                    );
                    ShadowValidationRun {
                        shadow_validation_run_id,
                        candidate_lifecycle_key: candidate_lifecycle_key.clone(),
                        symbol_canonical: aggregate.symbol_canonical.clone(),
                        trigger_research_run_id: research_run_report_id.to_owned(),
                        start_condition_summary: ShadowStartConditionSummary {
                            research_aggregate_key: aggregate.research_aggregate_key.clone(),
                            gate_policy_version: default_research_gate_policy().policy_version,
                            completed_count: aggregate.completed_count,
                            mean_net_after_cost_bps: aggregate.mean_net_after_cost_bps,
                            win_rate_ppm: aggregate.win_rate_ppm,
                            profit_factor_ppm: aggregate.profit_factor_ppm,
                            gate_reason_codes: aggregate.gate_reason_codes.clone(),
                        },
                        expected_survival_band: aggregate.survival_band.clone(),
                        watch_window_policy: ShadowWatchWindowPolicy {
                            mode: "forward_observation_only".to_owned(),
                            min_shadow_samples: 30,
                            max_shadow_age_days: 30,
                        },
                        termination_policy: ShadowTerminationPolicy {
                            prune_on_non_positive_mean_net: true,
                            prune_on_max_age_without_samples: true,
                            no_order_execution: true,
                        },
                        holding_policy: default_holding_policy(
                            decision_time_by_candidate_key
                                .get(candidate_lifecycle_key)
                                .copied()
                                .unwrap_or(0),
                        ),
                        status: ShadowValidationStatus::Pending,
                        passed: false,
                        paper_trade_candidate_contract_version:
                            PAPER_TRADE_CANDIDATE_SCHEMA_VERSION.to_owned(),
                        schema_version: SHADOW_VALIDATION_RUN_SCHEMA_VERSION.to_owned(),
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

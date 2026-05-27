use crate::gate::{build_partition_aggregates, default_research_gate_policy};
use crate::hash::stable_id;
use crate::holding::default_holding_policy;
use crate::model::{
    HypothesisOutput, IntelCandidateEvidenceBundle, OssAdapterRun, OssAdapterVerdictBias,
    PAPER_TRADE_CANDIDATE_SCHEMA_VERSION, RESEARCH_RUN_REPORT_SCHEMA_VERSION, ReplayRun,
    ReplayRunStatus, ResearchBias, ResearchPartitionAggregate, ResearchRunReport,
    ResearchRunStatus, SHADOW_VALIDATION_RUN_SCHEMA_VERSION, ShadowStartConditionSummary,
    ShadowTerminationPolicy, ShadowValidationRun, ShadowValidationStatus, ShadowWatchWindowPolicy,
    SummaryFinding,
};
use crate::paper::is_completed_passed_shadow;
use crate::portfolio::build_portfolio_artifacts;
use std::collections::{BTreeMap, BTreeSet};

pub fn build_report(
    research_packet_id: &str,
    run_scope: &str,
    created_at_ms: i64,
    bundles: &[IntelCandidateEvidenceBundle],
    replay_runs: &[ReplayRun],
    oss_adapter_runs: &[OssAdapterRun],
    completed_shadow_validation_runs: &[ShadowValidationRun],
) -> ResearchRunReport {
    let candidate_identity = candidate_identity_parts(bundles).join("|");
    let replay_identity = replay_identity_parts(replay_runs).join("|");
    let oss_identity = oss_identity_parts(oss_adapter_runs).join("|");
    let shadow_identity = shadow_identity_parts(completed_shadow_validation_runs).join("|");
    let report_id = stable_id(
        "research_report",
        &[
            research_packet_id,
            run_scope,
            &bundles.len().to_string(),
            &candidate_identity,
            &replay_identity,
            &oss_identity,
            &shadow_identity,
        ],
    );
    let source_candidate_ids = bundles
        .iter()
        .map(|bundle| bundle.candidate_id.clone())
        .collect::<Vec<_>>();
    let top_symbols = bundles
        .iter()
        .flat_map(|bundle| bundle.normalized_symbols.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let top_families = bundles
        .iter()
        .map(|bundle| bundle.hypothesis_type.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let research_gate_policy = default_research_gate_policy();
    let partition_aggregates =
        build_partition_aggregates(bundles, replay_runs, &research_gate_policy, created_at_ms);
    let summary_findings = candidate_findings(
        bundles,
        replay_runs,
        &partition_aggregates,
        oss_adapter_runs,
        completed_shadow_validation_runs,
    );
    let invalid_input_candidate_keys = invalid_input_candidate_keys(bundles, replay_runs);
    let pruned_candidate_keys = summary_findings
        .iter()
        .filter(|finding| finding.bias == ResearchBias::PruneBias)
        .map(|finding| finding.candidate_lifecycle_key.clone())
        .collect::<Vec<_>>();
    let retest_candidate_keys = summary_findings
        .iter()
        .filter(|finding| finding.bias == ResearchBias::RetestBias)
        .map(|finding| finding.candidate_lifecycle_key.clone())
        .collect::<Vec<_>>();
    let surviving_candidate_keys = summary_findings
        .iter()
        .filter(|finding| {
            matches!(
                finding.bias,
                ResearchBias::PromoteToShadowBias | ResearchBias::PromoteToPaperBias
            )
        })
        .map(|finding| finding.candidate_lifecycle_key.clone())
        .collect::<Vec<_>>();
    let shadow_validation_runs = shadow_validation_run_ids(
        research_packet_id,
        &report_id,
        run_scope,
        &partition_aggregates,
        &summary_findings,
        bundles,
    );
    let status = if !invalid_input_candidate_keys.is_empty()
        && replay_runs.len() == invalid_input_candidate_keys.len()
    {
        ResearchRunStatus::InvalidInput
    } else if !invalid_input_candidate_keys.is_empty() {
        ResearchRunStatus::Partial
    } else {
        ResearchRunStatus::Completed
    };

    let mut report = ResearchRunReport {
        research_run_report_id: report_id,
        research_packet_id: research_packet_id.to_owned(),
        source_candidate_ids,
        run_scope: run_scope.to_owned(),
        partition_count: partition_aggregates.len(),
        top_symbols,
        top_families,
        surviving_candidate_keys,
        pruned_candidate_keys,
        retest_candidate_keys,
        shadow_validation_runs,
        paper_watch_candidates: Vec::new(),
        paper_trade_candidates: Vec::new(),
        oss_adapter_run_ids: oss_adapter_runs
            .iter()
            .map(|run| run.oss_adapter_run_id.clone())
            .collect(),
        oss_adapter_reject_count: oss_adapter_runs
            .iter()
            .filter(|run| run.normalized_verdict_bias == OssAdapterVerdictBias::PruneBias)
            .count(),
        portfolio_allocation_snapshot: None,
        portfolio_risk_reject_events: Vec::new(),
        portfolio_reduce_only_signals: Vec::new(),
        hypothesis_outputs: HypothesisOutput::None,
        research_gate_policy,
        partition_aggregates,
        summary_findings,
        research_run_status: status,
        created_at_ms,
        replay_run_ids: replay_runs
            .iter()
            .map(|run| run.replay_run_id.clone())
            .collect(),
        invalid_input_candidate_keys,
        schema_version: RESEARCH_RUN_REPORT_SCHEMA_VERSION.to_owned(),
    };
    let (snapshot, rejects, reduce_only_signals) =
        build_portfolio_artifacts(&report, bundles, created_at_ms);
    report.portfolio_allocation_snapshot = Some(snapshot);
    report.portfolio_risk_reject_events = rejects;
    report.portfolio_reduce_only_signals = reduce_only_signals;
    report
}

fn candidate_identity_parts(bundles: &[IntelCandidateEvidenceBundle]) -> Vec<String> {
    let mut parts = bundles
        .iter()
        .map(|bundle| {
            format!(
                "{}:{}:{}:{}",
                bundle.candidate_id,
                bundle.candidate_lifecycle_key,
                bundle.bundle_key,
                bundle.idempotency_key
            )
        })
        .collect::<Vec<_>>();
    parts.sort();
    parts
}

fn replay_identity_parts(replay_runs: &[ReplayRun]) -> Vec<String> {
    let mut parts = replay_runs
        .iter()
        .map(|run| run.replay_run_id.clone())
        .collect::<Vec<_>>();
    parts.sort();
    parts
}

fn oss_identity_parts(oss_adapter_runs: &[OssAdapterRun]) -> Vec<String> {
    let mut parts = oss_adapter_runs
        .iter()
        .map(|run| {
            format!(
                "{}:{}:{:?}",
                run.oss_adapter_run_id, run.candidate_lifecycle_key, run.normalized_verdict_bias
            )
        })
        .collect::<Vec<_>>();
    parts.sort();
    parts
}

fn shadow_identity_parts(shadow_validation_runs: &[ShadowValidationRun]) -> Vec<String> {
    let mut parts = shadow_validation_runs
        .iter()
        .map(|run| {
            format!(
                "{}:{}:{:?}:{}",
                run.shadow_validation_run_id, run.candidate_lifecycle_key, run.status, run.passed
            )
        })
        .collect::<Vec<_>>();
    parts.sort();
    parts
}

fn invalid_input_candidate_keys(
    bundles: &[IntelCandidateEvidenceBundle],
    replay_runs: &[ReplayRun],
) -> Vec<String> {
    bundles
        .iter()
        .filter(|bundle| {
            let candidate_runs = replay_runs
                .iter()
                .filter(|run| run.source_candidate_id == bundle.candidate_id)
                .collect::<Vec<_>>();
            !candidate_runs.is_empty()
                && candidate_runs
                    .iter()
                    .all(|run| run.result_summary.status == ReplayRunStatus::InvalidInput)
        })
        .map(|bundle| bundle.candidate_lifecycle_key.clone())
        .collect()
}

fn candidate_findings(
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

fn shadow_validation_run_ids(
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

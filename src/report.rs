mod findings;
mod identity;
mod shadow;

use crate::gate::{build_partition_aggregates, default_research_gate_policy};
use crate::hash::stable_id;
use crate::model::{
    HypothesisOutput, IntelCandidateEvidenceBundle, OssAdapterRun, OssAdapterVerdictBias,
    RESEARCH_RUN_REPORT_SCHEMA_VERSION, ReplayRun, ResearchBias, ResearchRunReport,
    ResearchRunStatus, ShadowValidationRun,
};
use crate::portfolio::build_portfolio_artifacts;
use std::collections::BTreeSet;

pub fn build_report(
    research_packet_id: &str,
    run_scope: &str,
    created_at_ms: i64,
    bundles: &[IntelCandidateEvidenceBundle],
    replay_runs: &[ReplayRun],
    oss_adapter_runs: &[OssAdapterRun],
    completed_shadow_validation_runs: &[ShadowValidationRun],
) -> ResearchRunReport {
    let candidate_identity = identity::candidate_identity_parts(bundles).join("|");
    let replay_identity = identity::replay_identity_parts(replay_runs).join("|");
    let oss_identity = identity::oss_identity_parts(oss_adapter_runs).join("|");
    let shadow_identity =
        identity::shadow_identity_parts(completed_shadow_validation_runs).join("|");
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
    let summary_findings = findings::candidate_findings(
        bundles,
        replay_runs,
        &partition_aggregates,
        oss_adapter_runs,
        completed_shadow_validation_runs,
    );
    let invalid_input_candidate_keys = identity::invalid_input_candidate_keys(bundles, replay_runs);
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
    let shadow_validation_runs = shadow::shadow_validation_run_ids(
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

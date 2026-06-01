mod classification;
mod findings;
mod identity;
mod shadow;
mod summary;

use crate::gate::{build_partition_aggregates, default_research_gate_policy};
use crate::model::{
    HypothesisOutput, IntelCandidateEvidenceBundle, OssAdapterRun, OssAdapterVerdictBias,
    RESEARCH_RUN_REPORT_SCHEMA_VERSION, ReplayRun, ResearchRunReport, ShadowValidationRun,
};
use crate::portfolio::build_portfolio_artifacts;

pub fn build_report(
    research_packet_id: &str,
    run_scope: &str,
    created_at_ms: i64,
    bundles: &[IntelCandidateEvidenceBundle],
    replay_runs: &[ReplayRun],
    oss_adapter_runs: &[OssAdapterRun],
    completed_shadow_validation_runs: &[ShadowValidationRun],
) -> ResearchRunReport {
    let report_id = identity::report_id(
        research_packet_id,
        run_scope,
        bundles,
        replay_runs,
        oss_adapter_runs,
        completed_shadow_validation_runs,
    );
    let source_candidate_ids = summary::source_candidate_ids(bundles);
    let top_symbols = summary::top_symbols(bundles);
    let top_families = summary::top_families(bundles);
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
    let pruned_candidate_keys = classification::pruned_candidate_keys(&summary_findings);
    let retest_candidate_keys = classification::retest_candidate_keys(&summary_findings);
    let surviving_candidate_keys = classification::surviving_candidate_keys(&summary_findings);
    let shadow_validation_runs = shadow::shadow_validation_run_ids(
        research_packet_id,
        &report_id,
        run_scope,
        &partition_aggregates,
        &summary_findings,
        bundles,
    );
    let status =
        classification::research_run_status(invalid_input_candidate_keys.len(), bundles.len());

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

use crate::hash::stable_id;
use crate::model::{
    IntelCandidateEvidenceBundle, PAPER_WATCH_CANDIDATE_SCHEMA_VERSION, PaperAccountProfile,
    PaperExpectedCostProfile, PaperExpectedRiskProfile, PaperWatchCandidate,
    PaperWatchReplaySampleSummary, PaperWatchSafety, ResearchPartitionAggregate, ResearchRunReport,
    SummaryFinding,
};
use crate::paper::shared::max_drawdown_band;

pub(super) fn paper_watch_candidate(
    report: &ResearchRunReport,
    finding: &SummaryFinding,
    bundle: &IntelCandidateEvidenceBundle,
    aggregate: &ResearchPartitionAggregate,
    profile: &PaperAccountProfile,
    created_at_ms: i64,
) -> PaperWatchCandidate {
    let paper_watch_candidate_id = stable_id(
        "paper_watch_candidate",
        &[
            &report.research_run_report_id,
            &finding.candidate_lifecycle_key,
            &aggregate.research_aggregate_key,
        ],
    );
    let holding_policy = crate::holding::default_holding_policy(bundle.decision_available_at_ms);

    PaperWatchCandidate {
        paper_watch_candidate_id,
        candidate_id: finding.candidate_id.clone(),
        candidate_lifecycle_key: finding.candidate_lifecycle_key.clone(),
        symbol_canonical: aggregate.symbol_canonical.clone(),
        source_research_run_id: report.research_run_report_id.clone(),
        source_research_packet_id: report.research_packet_id.clone(),
        source_research_bias: finding.bias.clone(),
        historical_survival_band: aggregate.survival_band.clone(),
        admission_reason_codes: admission_reason_codes(),
        blocked_promotion_reason_codes: finding.reason_codes.clone(),
        replay_sample_summary: replay_sample_summary(aggregate),
        expected_cost_profile: expected_cost_profile(profile, aggregate),
        expected_risk_profile: expected_risk_profile(aggregate),
        target_max_holding_hours: holding_policy.target_max_holding_hours,
        absolute_max_holding_hours: holding_policy.absolute_max_holding_hours,
        force_flat_policy: holding_policy.force_flat_policy,
        paper_start_recommendation: "start_forward_paper_watch".to_owned(),
        safety: paper_watch_safety(),
        created_at_ms,
        schema_version: PAPER_WATCH_CANDIDATE_SCHEMA_VERSION.to_owned(),
    }
}

fn admission_reason_codes() -> Vec<String> {
    vec![
        "retest_positive_watch_admitted".to_owned(),
        "paper_only_no_order_execution".to_owned(),
    ]
}

fn replay_sample_summary(aggregate: &ResearchPartitionAggregate) -> PaperWatchReplaySampleSummary {
    PaperWatchReplaySampleSummary {
        research_aggregate_key: aggregate.research_aggregate_key.clone(),
        replay_run_count: aggregate.replay_run_count,
        completed_count: aggregate.completed_count,
        positive_net_count: aggregate.positive_net_count,
        non_positive_net_count: aggregate.non_positive_net_count,
        missing_market_replay_data_count: aggregate.missing_market_replay_data_count,
        insufficient_evidence_count: aggregate.insufficient_evidence_count,
        effective_completed_sample_weight: aggregate.effective_completed_sample_weight,
        weighted_mean_net_after_cost_bps: aggregate.weighted_mean_net_after_cost_bps,
        weighted_profit_factor_ppm: aggregate.weighted_profit_factor_ppm,
    }
}

fn expected_cost_profile(
    profile: &PaperAccountProfile,
    aggregate: &ResearchPartitionAggregate,
) -> PaperExpectedCostProfile {
    PaperExpectedCostProfile {
        fee_model_version: profile.fee_model_version.clone(),
        slippage_model_version: profile.slippage_model_version.clone(),
        estimated_cost_bps: aggregate.estimated_cost_bps,
        cost_stressed_mean_net_after_cost_bps: aggregate.cost_stressed_mean_net_after_cost_bps,
    }
}

fn expected_risk_profile(aggregate: &ResearchPartitionAggregate) -> PaperExpectedRiskProfile {
    PaperExpectedRiskProfile {
        survival_band: aggregate.survival_band.clone(),
        max_drawdown_band: max_drawdown_band(aggregate),
        positive_net_count: aggregate.positive_net_count,
        non_positive_net_count: aggregate.non_positive_net_count,
    }
}

fn paper_watch_safety() -> PaperWatchSafety {
    PaperWatchSafety {
        paper_only: true,
        live_enabled: false,
        order_execution_enabled: false,
        execution_approval_emitted: false,
    }
}

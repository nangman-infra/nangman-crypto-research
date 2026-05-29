use super::artifacts::default_paper_account_profile;
use super::shared::{aggregates_by_candidate_key, has_major_failure_event, max_drawdown_band};
use crate::hash::stable_id;
use crate::model::{
    IntelCandidateEvidenceBundle, PAPER_WATCH_CANDIDATE_SCHEMA_VERSION, PaperExpectedCostProfile,
    PaperExpectedRiskProfile, PaperWatchCandidate, PaperWatchReplaySampleSummary, PaperWatchSafety,
    ResearchBias, ResearchPartitionAggregate, ResearchRunReport,
};
use std::collections::BTreeMap;

pub fn build_paper_watch_candidates(
    report: &ResearchRunReport,
    bundles: &[IntelCandidateEvidenceBundle],
    created_at_ms: i64,
) -> Vec<PaperWatchCandidate> {
    let profile = default_paper_account_profile();
    let bundle_by_key = bundles
        .iter()
        .map(|bundle| (bundle.candidate_lifecycle_key.as_str(), bundle))
        .collect::<BTreeMap<_, _>>();
    let aggregates_by_candidate_key = aggregates_by_candidate_key(&report.partition_aggregates);
    let mut candidates = Vec::new();

    for finding in report
        .summary_findings
        .iter()
        .filter(|finding| finding.bias == ResearchBias::RetestBias)
    {
        let Some(bundle) = bundle_by_key.get(finding.candidate_lifecycle_key.as_str()) else {
            continue;
        };
        if !bundle.approved_universe_symbol || has_major_failure_event(bundle) {
            continue;
        }
        let Some(aggregate) = best_paper_watch_aggregate(
            aggregates_by_candidate_key
                .get(finding.candidate_lifecycle_key.as_str())
                .map(Vec::as_slice)
                .unwrap_or(&[]),
        ) else {
            continue;
        };

        let paper_watch_candidate_id = stable_id(
            "paper_watch_candidate",
            &[
                &report.research_run_report_id,
                &finding.candidate_lifecycle_key,
                &aggregate.research_aggregate_key,
            ],
        );
        let holding_policy =
            crate::holding::default_holding_policy(bundle.decision_available_at_ms);
        let admission_reason_codes = vec![
            "retest_positive_watch_admitted".to_owned(),
            "paper_only_no_order_execution".to_owned(),
        ];

        candidates.push(PaperWatchCandidate {
            paper_watch_candidate_id,
            candidate_id: finding.candidate_id.clone(),
            candidate_lifecycle_key: finding.candidate_lifecycle_key.clone(),
            symbol_canonical: aggregate.symbol_canonical.clone(),
            source_research_run_id: report.research_run_report_id.clone(),
            source_research_packet_id: report.research_packet_id.clone(),
            source_research_bias: finding.bias.clone(),
            historical_survival_band: aggregate.survival_band.clone(),
            admission_reason_codes,
            blocked_promotion_reason_codes: finding.reason_codes.clone(),
            replay_sample_summary: PaperWatchReplaySampleSummary {
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
            },
            expected_cost_profile: PaperExpectedCostProfile {
                fee_model_version: profile.fee_model_version.clone(),
                slippage_model_version: profile.slippage_model_version.clone(),
                estimated_cost_bps: aggregate.estimated_cost_bps,
                cost_stressed_mean_net_after_cost_bps: aggregate
                    .cost_stressed_mean_net_after_cost_bps,
            },
            expected_risk_profile: PaperExpectedRiskProfile {
                survival_band: aggregate.survival_band.clone(),
                max_drawdown_band: max_drawdown_band(aggregate),
                positive_net_count: aggregate.positive_net_count,
                non_positive_net_count: aggregate.non_positive_net_count,
            },
            target_max_holding_hours: holding_policy.target_max_holding_hours,
            absolute_max_holding_hours: holding_policy.absolute_max_holding_hours,
            force_flat_policy: holding_policy.force_flat_policy,
            paper_start_recommendation: "start_forward_paper_watch".to_owned(),
            safety: PaperWatchSafety {
                paper_only: true,
                live_enabled: false,
                order_execution_enabled: false,
                execution_approval_emitted: false,
            },
            created_at_ms,
            schema_version: PAPER_WATCH_CANDIDATE_SCHEMA_VERSION.to_owned(),
        });
    }

    candidates
}

fn best_paper_watch_aggregate<'a>(
    aggregates: &[&'a ResearchPartitionAggregate],
) -> Option<&'a ResearchPartitionAggregate> {
    aggregates
        .iter()
        .copied()
        .filter(|aggregate| paper_watch_eligible_aggregate(aggregate))
        .max_by(|left, right| {
            left.weighted_mean_net_after_cost_bps
                .unwrap_or_default()
                .partial_cmp(&right.weighted_mean_net_after_cost_bps.unwrap_or_default())
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| left.completed_count.cmp(&right.completed_count))
                .then_with(|| left.positive_net_count.cmp(&right.positive_net_count))
        })
}

fn paper_watch_eligible_aggregate(aggregate: &ResearchPartitionAggregate) -> bool {
    let missing_market_data_ratio_ppm = ratio_ppm(
        aggregate.missing_market_replay_data_count,
        aggregate.replay_run_count,
    );
    aggregate.gate_bias == ResearchBias::RetestBias
        && aggregate.positive_net_count > 0
        && aggregate.completed_count > 0
        && aggregate.non_positive_net_count == 0
        && missing_market_data_ratio_ppm <= 500_000
        && aggregate
            .weighted_mean_net_after_cost_bps
            .or(aggregate.mean_net_after_cost_bps)
            .is_some_and(|value| value > 0.0)
        && !aggregate
            .gate_reason_codes
            .iter()
            .any(|reason| reason == "aggregate_net_edge_non_positive")
        && !aggregate
            .gate_reason_codes
            .iter()
            .any(|reason| reason == "native_replay_net_edge_non_positive")
}

fn ratio_ppm(numerator: usize, denominator: usize) -> u64 {
    if denominator == 0 {
        return 1_000_000;
    }
    ((numerator as u128 * 1_000_000) / denominator as u128) as u64
}

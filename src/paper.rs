use crate::hash::stable_id;
use crate::model::{
    DEFAULT_PAPER_ACCOUNT_PROFILE_ID, DEFAULT_PAPER_FEE_MODEL_VERSION,
    DEFAULT_PAPER_SLIPPAGE_MODEL_VERSION, IntelCandidateEvidenceBundle,
    PAPER_ACCOUNT_PROFILE_SCHEMA_VERSION, PAPER_TRADE_CANDIDATE_SCHEMA_VERSION,
    PAPER_TRADE_MARK_SCHEMA_VERSION, PAPER_TRADE_RUN_SCHEMA_VERSION,
    PAPER_TRADE_SUMMARY_SCHEMA_VERSION, PaperAccountProfile, PaperExpectedCostProfile,
    PaperExpectedRiskProfile, PaperShadowSummary, PaperTradeCandidate, PaperTradeMark,
    PaperTradeRun, PaperTradeSummary, ResearchBias, ResearchPartitionAggregate, ResearchRunReport,
    ShadowValidationRun, ShadowValidationStatus, SurvivalBand,
};
use std::collections::{BTreeMap, BTreeSet};

const MS_PER_HOUR: i64 = 60 * 60 * 1000;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PaperArtifacts {
    pub candidates: Vec<PaperTradeCandidate>,
    pub runs: Vec<PaperTradeRun>,
    pub summaries: Vec<PaperTradeSummary>,
    pub marks: Vec<PaperTradeMark>,
}

pub fn default_paper_account_profile() -> PaperAccountProfile {
    PaperAccountProfile {
        paper_account_profile_id: DEFAULT_PAPER_ACCOUNT_PROFILE_ID.to_owned(),
        virtual_starting_balance: 10_000.0,
        max_notional_per_candidate: 100.0,
        fee_model_version: DEFAULT_PAPER_FEE_MODEL_VERSION.to_owned(),
        slippage_model_version: DEFAULT_PAPER_SLIPPAGE_MODEL_VERSION.to_owned(),
        marking_frequency: "hourly".to_owned(),
        target_max_holding_hours: 24,
        absolute_max_holding_hours: 72,
        force_flat_policy: "daily_or_ttl_exit".to_owned(),
        schema_version: PAPER_ACCOUNT_PROFILE_SCHEMA_VERSION.to_owned(),
    }
}

pub fn build_paper_artifacts(
    report: &ResearchRunReport,
    bundles: &[IntelCandidateEvidenceBundle],
    completed_shadow_validation_runs: &[ShadowValidationRun],
    created_at_ms: i64,
) -> PaperArtifacts {
    let profile = default_paper_account_profile();
    let bundle_by_key = bundles
        .iter()
        .map(|bundle| (bundle.candidate_lifecycle_key.as_str(), bundle))
        .collect::<BTreeMap<_, _>>();
    let aggregate_by_candidate_key = aggregate_by_candidate_key(&report.partition_aggregates);
    let passed_shadow_by_candidate_key =
        passed_shadow_by_candidate_key(completed_shadow_validation_runs);
    let paper_candidate_keys = report
        .summary_findings
        .iter()
        .filter(|finding| finding.bias == ResearchBias::PromoteToPaperBias)
        .map(|finding| finding.candidate_lifecycle_key.clone())
        .collect::<BTreeSet<_>>();

    let mut artifacts = PaperArtifacts::default();
    for candidate_lifecycle_key in paper_candidate_keys {
        let Some(bundle) = bundle_by_key.get(candidate_lifecycle_key.as_str()) else {
            continue;
        };
        if has_major_failure_event(bundle) {
            continue;
        }
        let Some(aggregate) = aggregate_by_candidate_key.get(candidate_lifecycle_key.as_str())
        else {
            continue;
        };
        let Some(shadow_run) = passed_shadow_by_candidate_key.get(candidate_lifecycle_key.as_str())
        else {
            continue;
        };
        if shadow_run.holding_policy.target_max_holding_hours > 24
            || shadow_run.holding_policy.absolute_max_holding_hours > 72
        {
            continue;
        }

        let paper_candidate_id = stable_id(
            "paper_trade_candidate",
            &[
                &report.research_run_report_id,
                &candidate_lifecycle_key,
                &shadow_run.shadow_validation_run_id,
            ],
        );
        let candidate = PaperTradeCandidate {
            paper_trade_candidate_id: paper_candidate_id.clone(),
            candidate_lifecycle_key: candidate_lifecycle_key.clone(),
            symbol_canonical: aggregate.symbol_canonical.clone(),
            source_research_run_id: report.research_run_report_id.clone(),
            historical_survival_band: aggregate.survival_band.clone(),
            shadow_summary: PaperShadowSummary {
                shadow_validation_run_id: shadow_run.shadow_validation_run_id.clone(),
                status: shadow_run.status.clone(),
                passed: shadow_run.passed,
                completed_count: shadow_run.start_condition_summary.completed_count,
                mean_net_after_cost_bps: shadow_run.start_condition_summary.mean_net_after_cost_bps,
                win_rate_ppm: shadow_run.start_condition_summary.win_rate_ppm,
                profit_factor_ppm: shadow_run.start_condition_summary.profit_factor_ppm,
                reason_codes: shadow_run.start_condition_summary.gate_reason_codes.clone(),
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
            target_max_holding_hours: shadow_run.holding_policy.target_max_holding_hours,
            absolute_max_holding_hours: shadow_run.holding_policy.absolute_max_holding_hours,
            force_flat_policy: shadow_run.holding_policy.force_flat_policy.clone(),
            paper_start_recommendation: "start_paper_observation".to_owned(),
            schema_version: PAPER_TRADE_CANDIDATE_SCHEMA_VERSION.to_owned(),
        };

        let paper_trade_run_id = stable_id(
            "paper_trade_run",
            &[&paper_candidate_id, &report.research_run_report_id],
        );
        let net_result_band = net_result_band(aggregate);
        let survival_result = survival_result(aggregate);
        let run = PaperTradeRun {
            paper_trade_run_id: paper_trade_run_id.clone(),
            candidate_lifecycle_key: candidate_lifecycle_key.clone(),
            symbol_canonical: aggregate.symbol_canonical.clone(),
            source_research_run_id: report.research_run_report_id.clone(),
            paper_account_profile_id: profile.paper_account_profile_id.clone(),
            started_at_ms: created_at_ms,
            ended_at_ms: created_at_ms
                + i64::from(shadow_run.holding_policy.target_max_holding_hours) * MS_PER_HOUR,
            entry_count: aggregate.completed_count,
            exit_count: aggregate.completed_count,
            max_drawdown_band: max_drawdown_band(aggregate),
            net_result_band: net_result_band.clone(),
            survival_result: survival_result.clone(),
            schema_version: PAPER_TRADE_RUN_SCHEMA_VERSION.to_owned(),
        };
        let summary = PaperTradeSummary {
            paper_trade_summary_id: stable_id("paper_trade_summary", &[&paper_trade_run_id]),
            paper_trade_run_id: paper_trade_run_id.clone(),
            candidate_lifecycle_key: candidate_lifecycle_key.clone(),
            summary_window: format!(
                "target_{}h_absolute_{}h",
                shadow_run.holding_policy.target_max_holding_hours,
                shadow_run.holding_policy.absolute_max_holding_hours
            ),
            entry_behavior_summary: format!(
                "entries_from_completed_replay_windows={}",
                run.entry_count
            ),
            exit_behavior_summary: format!("ttl_exit_policy={}", candidate.force_flat_policy),
            cost_behavior_summary: format!(
                "fee_model={},slippage_model={},estimated_cost_bps={}",
                profile.fee_model_version,
                profile.slippage_model_version,
                aggregate
                    .estimated_cost_bps
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "unknown".to_owned())
            ),
            risk_behavior_summary: format!(
                "survival_band={:?},max_drawdown_band={}",
                aggregate.survival_band, run.max_drawdown_band
            ),
            promote_recommendation: promote_recommendation(&survival_result),
            schema_version: PAPER_TRADE_SUMMARY_SCHEMA_VERSION.to_owned(),
        };
        let mark = PaperTradeMark {
            paper_trade_mark_id: stable_id("paper_trade_mark", &[&paper_trade_run_id]),
            paper_trade_run_id,
            candidate_lifecycle_key,
            symbol_canonical: aggregate.symbol_canonical.clone(),
            marked_at_ms: created_at_ms,
            mark_source: "research_replay_proxy".to_owned(),
            net_result_band,
            survival_result,
            schema_version: PAPER_TRADE_MARK_SCHEMA_VERSION.to_owned(),
        };

        artifacts.candidates.push(candidate);
        artifacts.runs.push(run);
        artifacts.summaries.push(summary);
        artifacts.marks.push(mark);
    }

    artifacts
}

pub fn is_completed_passed_shadow(run: &ShadowValidationRun) -> bool {
    run.status == ShadowValidationStatus::Completed
        && run.passed
        && run.paper_trade_candidate_contract_version == PAPER_TRADE_CANDIDATE_SCHEMA_VERSION
}

fn aggregate_by_candidate_key(
    aggregates: &[ResearchPartitionAggregate],
) -> BTreeMap<&str, &ResearchPartitionAggregate> {
    let mut values = BTreeMap::new();
    for aggregate in aggregates {
        for candidate_lifecycle_key in &aggregate.source_candidate_lifecycle_keys {
            values.insert(candidate_lifecycle_key.as_str(), aggregate);
        }
    }
    values
}

fn passed_shadow_by_candidate_key(
    runs: &[ShadowValidationRun],
) -> BTreeMap<&str, &ShadowValidationRun> {
    runs.iter()
        .filter(|run| is_completed_passed_shadow(run))
        .map(|run| (run.candidate_lifecycle_key.as_str(), run))
        .collect()
}

fn has_major_failure_event(bundle: &IntelCandidateEvidenceBundle) -> bool {
    bundle.event_types.iter().any(|event_type| {
        matches!(
            event_type.as_str(),
            "exchange_delisting" | "exchange_halt" | "security_incident" | "chain_halt"
        )
    })
}

fn max_drawdown_band(aggregate: &ResearchPartitionAggregate) -> String {
    if aggregate.completed_count == 0 {
        return "unknown".to_owned();
    }
    let non_positive_ratio =
        aggregate.non_positive_net_count as f64 / aggregate.completed_count as f64;
    if non_positive_ratio == 0.0 {
        "low".to_owned()
    } else if non_positive_ratio <= 0.2 {
        "controlled".to_owned()
    } else {
        "elevated".to_owned()
    }
}

fn net_result_band(aggregate: &ResearchPartitionAggregate) -> String {
    match aggregate.weighted_mean_net_after_cost_bps {
        Some(value) if value >= 20.0 => "strong_positive".to_owned(),
        Some(value) if value > 0.0 => "positive".to_owned(),
        Some(_) => "non_positive".to_owned(),
        None => "unknown".to_owned(),
    }
}

fn survival_result(aggregate: &ResearchPartitionAggregate) -> String {
    if aggregate
        .weighted_mean_net_after_cost_bps
        .is_none_or(|value| value <= 0.0)
    {
        return "failed_fast".to_owned();
    }
    if aggregate.non_positive_net_count > 0 {
        return "mixed".to_owned();
    }
    if aggregate.survival_band == SurvivalBand::Exceptional {
        "survived_strong".to_owned()
    } else {
        "survived".to_owned()
    }
}

fn promote_recommendation(survival_result: &str) -> String {
    match survival_result {
        "survived_strong" => "approve_execution_review".to_owned(),
        "survived" => "retest".to_owned(),
        _ => "reject".to_owned(),
    }
}

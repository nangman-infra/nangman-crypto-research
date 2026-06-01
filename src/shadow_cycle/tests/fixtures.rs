use super::*;

pub(super) const TARGET_HOURS: u32 = 24;
const ABSOLUTE_HOURS: u32 = 72;

pub(super) fn shadow_run(
    shadow_validation_run_id: &str,
    candidate_lifecycle_key: &str,
    symbol_canonical: &str,
    decision_available_ms: i64,
    min_shadow_samples: usize,
) -> ShadowValidationRun {
    ShadowValidationRun {
        shadow_validation_run_id: shadow_validation_run_id.to_owned(),
        candidate_lifecycle_key: candidate_lifecycle_key.to_owned(),
        symbol_canonical: symbol_canonical.to_owned(),
        trigger_research_run_id: "research_report_test".to_owned(),
        start_condition_summary: ShadowStartConditionSummary {
            research_aggregate_key: "aggregate_test".to_owned(),
            gate_policy_version: "test_gate_policy".to_owned(),
            completed_count: 30,
            mean_net_after_cost_bps: Some(12.0),
            win_rate_ppm: Some(600_000),
            profit_factor_ppm: Some(1_200_000),
            gate_reason_codes: vec!["deterministic_shadow_gate_passed".to_owned()],
        },
        expected_survival_band: SurvivalBand::Stable,
        watch_window_policy: ShadowWatchWindowPolicy {
            mode: "forward_observation_only".to_owned(),
            min_shadow_samples,
            max_shadow_age_days: 30,
        },
        termination_policy: ShadowTerminationPolicy {
            prune_on_non_positive_mean_net: true,
            prune_on_max_age_without_samples: true,
            no_order_execution: true,
        },
        holding_policy: HoldingPolicy {
            target_max_holding_hours: TARGET_HOURS,
            absolute_max_holding_hours: ABSOLUTE_HOURS,
            absolute_exit_deadline_ms: decision_available_ms
                + i64::from(ABSOLUTE_HOURS) * MS_PER_HOUR,
            force_flat_policy: "daily_or_ttl_exit".to_owned(),
            overnight_risk_exception: false,
            holding_policy_version: HOLDING_POLICY_VERSION.to_owned(),
        },
        status: ShadowValidationStatus::Pending,
        passed: false,
        paper_trade_candidate_contract_version: "paper_trade_candidate_v1".to_owned(),
        schema_version: "shadow_validation_run_v1".to_owned(),
    }
}

use super::{
    MS_PER_HOUR, build_shadow_cycle_decision, shadow_sample_deficit_lifecycle_keys,
    validate_shadow_cycle_decision,
};
use crate::model::{
    HOLDING_POLICY_VERSION, HoldingPolicy, ShadowCycleDecision, ShadowCycleSchedulerAction,
    ShadowStartConditionSummary, ShadowTerminationPolicy, ShadowValidationRun,
    ShadowValidationStatus, ShadowWatchWindowPolicy, SurvivalBand,
};

const TARGET_HOURS: u32 = 24;
const ABSOLUTE_HOURS: u32 = 72;

#[test]
fn validates_wait_decision_contract() {
    let decision: ShadowCycleDecision = serde_json::from_str(
            r#"{
              "schema_version": "research_shadow_cycle_decision_v1",
              "generated_at": "2026-05-24T12:16:00Z",
              "decision_id": "shadow_cycle_decision:run:WAIT_FOR_PENDING_SHADOW_TARGET_WINDOW_MATERIALIZATION:1779670979756",
              "source_cycle_summary_file": "/tmp/run/shadow-sample-accumulation-cycle-summary.json",
              "run_dir": "/tmp/run",
              "scheduler_action": "WAIT_UNTIL_PENDING_SHADOW_TARGET_WINDOW_MATERIALIZES",
              "source_verdict": "WAIT_FOR_PENDING_SHADOW_TARGET_WINDOW_MATERIALIZATION",
              "run_not_before_ms": 1779670979756,
              "run_not_before_at": "2026-05-25T01:02:59Z",
              "run_not_before_source": "pending_shadow_target_exit_deadline_ms",
              "focused_research_manifest_file": null,
              "focused_research_summary_file": null,
              "latest_l1_as_of_ms": null,
              "shadow_sample_state": {
                "shadow_validation_count": 24,
                "target_window_materialized_count": 12,
                "candidate_lifecycle_count": 6,
                "partially_materialized_candidate_count": 6,
                "pending_target_window_candidate_count": 6,
                "total_sample_deficit": 168,
                "symbols": ["BTC", "DOGE", "ETH", "SOL", "TON", "ZEC"]
              },
              "safe_next_actions": ["wait_for_pending_shadow_target_window_materialization"],
              "blocked_actions": [
                "do_not_mark_pending_shadow_passed_from_sample_counts_only",
                "do_not_create_paper_without_completed_passed_shadow",
                "do_not_enable_live_from_shadow_sample_gap_manifest"
              ],
              "safety": {
                "s3_write": false,
                "ecs_task_started": false,
                "dispatcher_mode_changed": false,
                "local_decision_only": true,
                "shadow_status_mutated": false,
                "paper_live_enabled": false,
                "live_enabled": false,
                "order_execution_enabled": false
              }
            }"#,
        )
        .expect("wait decision parses");

    validate_shadow_cycle_decision(&decision).expect("wait decision validates");
}

#[test]
fn validates_focused_accumulation_decision_contract() {
    let decision: ShadowCycleDecision = serde_json::from_str(
            r#"{
              "schema_version": "research_shadow_cycle_decision_v1",
              "generated_at": "2026-05-24T12:16:00Z",
              "decision_id": "shadow_cycle_decision:run:ACCUMULATE_SHADOW_SAMPLES_BEFORE_COMPLETION:1779700000000",
              "source_cycle_summary_file": "/tmp/run/shadow-sample-accumulation-cycle-summary.json",
              "run_dir": "/tmp/run",
              "scheduler_action": "RUN_FOCUSED_SHADOW_SAMPLE_ACCUMULATION_RESEARCH",
              "source_verdict": "ACCUMULATE_SHADOW_SAMPLES_BEFORE_COMPLETION",
              "run_not_before_ms": null,
              "run_not_before_at": null,
              "run_not_before_source": null,
              "focused_research_manifest_file": "/tmp/run/shadow-accumulation-input-manifest.next.json",
              "focused_research_summary_file": "/tmp/run/shadow-accumulation-input-manifest.next.summary.json",
              "latest_l1_as_of_ms": 1779700000000,
              "shadow_sample_state": {
                "shadow_validation_count": 24,
                "target_window_materialized_count": 24,
                "candidate_lifecycle_count": 6,
                "partially_materialized_candidate_count": 0,
                "pending_target_window_candidate_count": 0,
                "total_sample_deficit": 156,
                "symbols": ["BTC", "DOGE", "ETH", "SOL", "TON", "ZEC"]
              },
              "safe_next_actions": ["accumulate_shadow_observation_samples"],
              "blocked_actions": [
                "do_not_mark_pending_shadow_passed_from_sample_counts_only",
                "do_not_create_paper_without_completed_passed_shadow",
                "do_not_enable_live_from_shadow_accumulation_manifest",
                "do_not_enable_live_from_shadow_sample_gap_manifest"
              ],
              "safety": {
                "s3_write": false,
                "ecs_task_started": false,
                "dispatcher_mode_changed": false,
                "local_decision_only": true,
                "shadow_status_mutated": false,
                "paper_live_enabled": false,
                "live_enabled": false,
                "order_execution_enabled": false
              }
            }"#,
        )
        .expect("focused decision parses");

    validate_shadow_cycle_decision(&decision).expect("focused decision validates");
}

#[test]
fn rejects_wait_decision_without_not_before_time() {
    let decision: ShadowCycleDecision = serde_json::from_str(
        r#"{
              "schema_version": "research_shadow_cycle_decision_v1",
              "generated_at": "2026-05-24T12:16:00Z",
              "decision_id": "shadow_cycle_decision:run:wait:none",
              "scheduler_action": "WAIT_UNTIL_TARGET_WINDOW_MATERIALIZES",
              "source_verdict": "WAIT_FOR_TARGET_HOLDING_WINDOW",
              "shadow_sample_state": {
                "shadow_validation_count": 1,
                "target_window_materialized_count": 0,
                "candidate_lifecycle_count": 1,
                "partially_materialized_candidate_count": 0,
                "pending_target_window_candidate_count": 1,
                "total_sample_deficit": 30,
                "symbols": ["BTC"]
              },
              "blocked_actions": [
                "do_not_create_paper_without_completed_passed_shadow",
                "do_not_enable_live_from_shadow_sample_gap_manifest"
              ],
              "safety": {
                "s3_write": false,
                "ecs_task_started": false,
                "dispatcher_mode_changed": false,
                "local_decision_only": true,
                "shadow_status_mutated": false,
                "paper_live_enabled": false,
                "live_enabled": false,
                "order_execution_enabled": false
              }
            }"#,
    )
    .expect("invalid wait decision parses");

    let error = validate_shadow_cycle_decision(&decision).expect_err("wait time is required");
    assert!(error.to_string().contains("run_not_before_ms"));
}

#[test]
fn rejects_decision_that_enables_order_execution() {
    let decision: ShadowCycleDecision = serde_json::from_str(
        r#"{
              "schema_version": "research_shadow_cycle_decision_v1",
              "generated_at": "2026-05-24T12:16:00Z",
              "decision_id": "shadow_cycle_decision:run:unsafe",
              "scheduler_action": "NOOP",
              "source_verdict": "NO_SHADOW_SAMPLE_GAP_DETECTED",
              "shadow_sample_state": {
                "shadow_validation_count": 0,
                "target_window_materialized_count": 0,
                "candidate_lifecycle_count": 0,
                "partially_materialized_candidate_count": 0,
                "pending_target_window_candidate_count": 0,
                "total_sample_deficit": 0,
                "symbols": []
              },
              "blocked_actions": [
                "do_not_create_paper_without_completed_passed_shadow",
                "do_not_enable_live_from_shadow_sample_gap_manifest"
              ],
              "safety": {
                "s3_write": false,
                "ecs_task_started": false,
                "dispatcher_mode_changed": false,
                "local_decision_only": true,
                "shadow_status_mutated": false,
                "paper_live_enabled": false,
                "live_enabled": false,
                "order_execution_enabled": true
              }
            }"#,
    )
    .expect("unsafe decision parses");

    let error = validate_shadow_cycle_decision(&decision).expect_err("order execution is rejected");
    assert!(error.to_string().contains("paper/live/order execution"));
}

#[test]
fn builds_wait_decision_until_target_windows_materialize() {
    let decision_available_ms = 1_780_000_000_000;
    let materialized_target_ms = decision_available_ms + i64::from(TARGET_HOURS) * MS_PER_HOUR;
    let later_decision_ms = decision_available_ms + 2 * MS_PER_HOUR;
    let runs = vec![
        shadow_run("shadow_a", "cand_a", "XAUT", decision_available_ms, 30),
        shadow_run("shadow_b", "cand_b", "CHIP", later_decision_ms, 30),
    ];

    let decision =
        build_shadow_cycle_decision(&runs, Some(materialized_target_ms), 1_780_100_000_000);

    assert_eq!(
        decision.scheduler_action,
        ShadowCycleSchedulerAction::WaitUntilTargetWindowMaterializes
    );
    assert_eq!(decision.source_verdict, "WAIT_FOR_TARGET_HOLDING_WINDOW");
    assert_eq!(
        decision.run_not_before_ms,
        Some(later_decision_ms + i64::from(TARGET_HOURS) * MS_PER_HOUR)
    );
    assert_eq!(decision.shadow_sample_state.shadow_validation_count, 2);
    assert_eq!(
        decision
            .shadow_sample_state
            .target_window_materialized_count,
        1
    );
    assert_eq!(
        decision
            .shadow_sample_state
            .pending_target_window_candidate_count,
        1
    );
    validate_shadow_cycle_decision(&decision).expect("generated wait decision validates");
}

#[test]
fn builds_operator_review_decision_when_samples_are_deficient() {
    let decision_available_ms = 1_780_000_000_000;
    let target_ms = decision_available_ms + i64::from(TARGET_HOURS) * MS_PER_HOUR;
    let runs = vec![shadow_run(
        "shadow_a",
        "cand_a",
        "XAUT",
        decision_available_ms,
        30,
    )];

    let decision = build_shadow_cycle_decision(&runs, Some(target_ms), 1_780_100_000_000);

    assert_eq!(
        decision.scheduler_action,
        ShadowCycleSchedulerAction::HoldForOperatorReview
    );
    assert_eq!(
        decision.source_verdict,
        "ACCUMULATE_SHADOW_SAMPLES_BEFORE_COMPLETION"
    );
    assert_eq!(decision.run_not_before_ms, None);
    assert_eq!(decision.shadow_sample_state.total_sample_deficit, 29);
    validate_shadow_cycle_decision(&decision).expect("generated hold decision validates");
}

#[test]
fn exposes_shadow_sample_deficit_lifecycle_keys_after_target_materializes() {
    let decision_available_ms = 1_780_000_000_000;
    let target_ms = decision_available_ms + i64::from(TARGET_HOURS) * MS_PER_HOUR;
    let pending_target_ms = target_ms - 1;
    let runs = vec![
        shadow_run("shadow_a", "cand_a", "XAUT", decision_available_ms, 30),
        shadow_run("shadow_b", "cand_b", "CHIP", decision_available_ms + 1, 30),
    ];

    assert!(shadow_sample_deficit_lifecycle_keys(&runs, Some(pending_target_ms)).is_empty());
    assert_eq!(
        shadow_sample_deficit_lifecycle_keys(&runs, Some(target_ms)),
        vec!["cand_a".to_owned()]
    );
}

fn shadow_run(
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

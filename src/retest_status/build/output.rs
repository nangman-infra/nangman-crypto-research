use crate::model::RETEST_HORIZON_STATUS_SCHEMA_VERSION;
use serde_json::{Value, json};

use super::super::RetestHorizonStatusBuildOptions;
use super::super::status_parts::{
    batch_state, candidate_horizon_matrix_summary, coverage_gaps, iso8601_ms, major50_state,
    research_factory_gap_summary, research_factory_progression,
};
use super::context::StatusBuildContext;
use super::decision::DecisionState;
use super::horizon::HorizonSummary;
use super::schedule::MaterializationSchedule;
use super::stage::StageState;

pub(super) fn build_status_json(
    options: &RetestHorizonStatusBuildOptions,
    context: &StatusBuildContext,
    stage: &StageState,
    horizon: &HorizonSummary,
    schedule: &MaterializationSchedule,
    decision: &DecisionState,
) -> Value {
    json!({
        "schema_version": RETEST_HORIZON_STATUS_SCHEMA_VERSION,
        "generated_at_ms": options.generated_at_ms,
        "generated_at": iso8601_ms(options.generated_at_ms),
        "retest_horizon_plan_file": options.plan_file,
        "driver_summary_file": options.driver_summary_file,
        "safety": {
            "s3_write": false,
            "ecs_task_started": false,
            "dispatcher_mode_changed": false,
            "local_summary_only": true,
            "checkpoint_s3_write": options.checkpoint_s3_write,
            "shadow_paper_live_enabled": false
        },
        "stage_state": stage.value,
        "batch_state": batch_state(&context.driver),
        "horizon_summary": horizon.value,
        "materialization_schedule": schedule.value,
        "by_symbol": horizon.by_symbol,
        "by_horizon": horizon.by_horizon,
        "candidate_horizon_matrix_summary": candidate_horizon_matrix_summary(&horizon.candidate_horizon_matrix),
        "candidate_horizon_matrix": horizon.candidate_horizon_matrix,
        "next_decision": {
            "verdict": decision.verdict,
            "safe_next_actions": decision.safe_next_actions,
            "scheduler_hint": {
                "latest_l1_as_of_ms": context.latest_l1_as_of_ms,
                "latest_l1_as_of_iso": context.latest_l1_as_of_ms.map(iso8601_ms),
                "run_research_after_l1_as_of_ms": schedule.next_wait_due_ms,
                "run_research_after_l1_as_of_iso": schedule.next_wait_due_ms.map(iso8601_ms),
                "wait_deficit_ms": schedule.wait_deficit_ms,
                "run_now_replay_ready": horizon.ready_for_replay_count > 0,
                "promotion_ready_for_review": horizon.promotion_ready_for_review_count > 0
            },
            "blocked_actions": decision.blocked_actions
        },
        "verdict": decision.verdict,
        "selected_symbols": horizon.symbols,
        "next_action_counts": horizon.action_counts,
        "major50_state": major50_state(&context.latest_universe, &context.driver, &context.rows),
        "research_factory_progression": research_factory_progression(
            &context.latest_universe,
            &context.rows,
            stage.promotion_passed,
            stage.shadow_created,
            stage.paper_created
        ),
        "coverage_gaps": coverage_gaps(
            &context.latest_universe,
            &context.driver,
            &context.rows,
            stage.shadow_created
        ),
        "research_factory_gap_summary": research_factory_gap_summary(
            &context.latest_universe,
            &context.driver,
            &context.rows,
            stage.promotion_passed,
            stage.shadow_created,
            stage.paper_created,
            &decision.safe_next_actions
        )
    })
}

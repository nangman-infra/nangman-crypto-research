use crate::error::{AppError, AppResult};
use crate::model::{
    SHADOW_CYCLE_DECISION_SCHEMA_VERSION, ShadowCycleDecision, ShadowCycleSchedulerAction,
};
use std::fs;
use std::path::Path;

pub fn read_shadow_cycle_decision(path: &Path) -> AppResult<ShadowCycleDecision> {
    if !path.is_absolute() {
        return Err(AppError::config(
            "shadow cycle decision file must be an absolute path",
        ));
    }
    let raw = fs::read_to_string(path)?;
    let decision = serde_json::from_str(&raw)?;
    Ok(decision)
}

pub fn validate_shadow_cycle_decision(decision: &ShadowCycleDecision) -> AppResult<()> {
    if decision.schema_version != SHADOW_CYCLE_DECISION_SCHEMA_VERSION {
        return Err(AppError::validation(format!(
            "shadow cycle decision schema_version must be {SHADOW_CYCLE_DECISION_SCHEMA_VERSION}; got {}",
            decision.schema_version
        )));
    }
    if decision.decision_id.trim().is_empty() {
        return Err(AppError::validation(
            "shadow cycle decision decision_id must be non-empty",
        ));
    }
    validate_scheduler_action(decision)?;
    validate_safety(decision)?;
    validate_blocked_actions(decision)?;
    Ok(())
}

fn validate_scheduler_action(decision: &ShadowCycleDecision) -> AppResult<()> {
    if decision.scheduler_action.is_wait_action() {
        if decision.run_not_before_ms.is_none() {
            return Err(AppError::validation(
                "wait shadow cycle decisions must include run_not_before_ms",
            ));
        }
        if decision.focused_research_manifest_file.is_some() {
            return Err(AppError::validation(
                "wait shadow cycle decisions must not include a focused research manifest",
            ));
        }
    }

    if decision
        .scheduler_action
        .requires_focused_research_manifest()
    {
        let Some(manifest_file) = &decision.focused_research_manifest_file else {
            return Err(AppError::validation(
                "focused shadow sample accumulation decisions must include focused_research_manifest_file",
            ));
        };
        if !manifest_file.starts_with('/') {
            return Err(AppError::validation(
                "focused_research_manifest_file must be an absolute local path",
            ));
        }
        if decision.run_not_before_ms.is_some() {
            return Err(AppError::validation(
                "focused shadow sample accumulation decisions must not include run_not_before_ms",
            ));
        }
    }

    if matches!(
        decision.scheduler_action,
        ShadowCycleSchedulerAction::Noop | ShadowCycleSchedulerAction::HoldForOperatorReview
    ) && (decision.run_not_before_ms.is_some()
        || decision.focused_research_manifest_file.is_some())
    {
        return Err(AppError::validation(
            "noop/operator-review shadow cycle decisions must not schedule work",
        ));
    }

    Ok(())
}

fn validate_safety(decision: &ShadowCycleDecision) -> AppResult<()> {
    let safety = &decision.safety;
    if safety.s3_write
        || safety.ecs_task_started
        || safety.dispatcher_mode_changed
        || safety.shadow_status_mutated
        || safety.paper_live_enabled
        || safety.live_enabled
        || safety.order_execution_enabled
    {
        return Err(AppError::validation(
            "shadow cycle decision must be local-only and must not enable paper/live/order execution",
        ));
    }
    if !safety.local_decision_only {
        return Err(AppError::validation(
            "shadow cycle decision must set local_decision_only=true",
        ));
    }
    Ok(())
}

fn validate_blocked_actions(decision: &ShadowCycleDecision) -> AppResult<()> {
    let required_actions = [
        "do_not_create_paper_without_completed_passed_shadow",
        "do_not_enable_live_from_shadow_sample_gap_manifest",
    ];
    for required_action in required_actions {
        if !decision
            .blocked_actions
            .iter()
            .any(|action| action == required_action)
        {
            return Err(AppError::validation(format!(
                "shadow cycle decision missing blocked action: {required_action}"
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ShadowCycleDecision;

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

        let error =
            validate_shadow_cycle_decision(&decision).expect_err("order execution is rejected");
        assert!(error.to_string().contains("paper/live/order execution"));
    }
}

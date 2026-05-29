use crate::error::{AppError, AppResult};
use crate::model::{
    SHADOW_CYCLE_DECISION_SCHEMA_VERSION, ShadowCycleDecision, ShadowCycleSchedulerAction,
};

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
        if !manifest_file.starts_with('/') && !manifest_file.starts_with("s3://") {
            return Err(AppError::validation(
                "focused_research_manifest_file must be an absolute local path or s3:// URI",
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

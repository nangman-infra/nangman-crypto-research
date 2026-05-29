use crate::error::{AppError, AppResult};
use crate::model::RETEST_HORIZON_STATUS_SCHEMA_VERSION;
use serde_json::Value;

pub(super) fn validate_status_safety(status: &Value) -> AppResult<()> {
    validate_schema(status)?;
    validate_local_safety(status)?;
    validate_live_boundaries(status)?;
    validate_blocked_actions(status)
}

fn validate_schema(status: &Value) -> AppResult<()> {
    let schema_version = status
        .get("schema_version")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if schema_version != RETEST_HORIZON_STATUS_SCHEMA_VERSION {
        return Err(AppError::validation(format!(
            "retest horizon status schema_version must be {RETEST_HORIZON_STATUS_SCHEMA_VERSION}; got {schema_version}"
        )));
    }
    Ok(())
}

fn validate_local_safety(status: &Value) -> AppResult<()> {
    let safety = status
        .get("safety")
        .ok_or_else(|| AppError::validation("retest horizon status missing safety block"))?;
    let unsafe_flags = [
        "s3_write",
        "ecs_task_started",
        "dispatcher_mode_changed",
        "shadow_paper_live_enabled",
    ];
    for flag in unsafe_flags {
        if safety.get(flag).and_then(Value::as_bool).unwrap_or(false) {
            return Err(AppError::validation(format!(
                "retest horizon status safety.{flag} must be false"
            )));
        }
    }
    if !safety
        .get("local_summary_only")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return Err(AppError::validation(
            "retest horizon status safety.local_summary_only must be true",
        ));
    }
    Ok(())
}

fn validate_live_boundaries(status: &Value) -> AppResult<()> {
    if status
        .pointer("/stage_state/live_enabled")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return Err(AppError::validation(
            "retest horizon status must not enable live trading",
        ));
    }
    Ok(())
}

fn validate_blocked_actions(status: &Value) -> AppResult<()> {
    let blocked_actions = status
        .pointer("/next_decision/blocked_actions")
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::validation("retest horizon status missing blocked_actions"))?;
    require_blocked_action(blocked_actions, "do_not_create_shadow_without_promotion")?;
    require_any_blocked_action(
        blocked_actions,
        &[
            "do_not_create_paper_without_passed_shadow",
            "do_not_create_paper_without_completed_passed_shadow",
        ],
    )?;
    require_blocked_action(blocked_actions, "do_not_enable_live_from_research_batch")?;
    Ok(())
}

fn require_blocked_action(blocked_actions: &[Value], expected: &str) -> AppResult<()> {
    if blocked_actions
        .iter()
        .any(|value| value.as_str() == Some(expected))
    {
        return Ok(());
    }
    Err(AppError::validation(format!(
        "retest horizon status missing blocked action: {expected}"
    )))
}

fn require_any_blocked_action(blocked_actions: &[Value], expected: &[&str]) -> AppResult<()> {
    if blocked_actions.iter().any(|value| {
        value
            .as_str()
            .is_some_and(|action| expected.contains(&action))
    }) {
        return Ok(());
    }
    Err(AppError::validation(format!(
        "retest horizon status missing one of blocked actions: {}",
        expected.join(", ")
    )))
}

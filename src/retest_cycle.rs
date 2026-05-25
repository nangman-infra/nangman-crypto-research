use crate::error::{AppError, AppResult};
use crate::model::RETEST_HORIZON_STATUS_SCHEMA_VERSION;
use serde::Serialize;
use serde_json::Value;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RetestHorizonStatusValidation {
    pub scheduler_action: String,
    pub run_not_before_ms: Option<i64>,
}

pub fn read_retest_horizon_status(path: &Path) -> AppResult<Value> {
    if !path.is_absolute() {
        return Err(AppError::config(
            "retest horizon status file must be an absolute path",
        ));
    }
    let raw = fs::read_to_string(path)?;
    read_retest_horizon_status_from_bytes(&path.display().to_string(), raw.as_bytes())
}

pub fn read_retest_horizon_status_from_bytes(label: &str, bytes: &[u8]) -> AppResult<Value> {
    let text =
        std::str::from_utf8(bytes).map_err(|error| AppError::Json(format!("{label}: {error}")))?;
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(AppError::validation(format!("{label} must not be empty")));
    }
    Ok(serde_json::from_str(trimmed)?)
}

pub fn validate_retest_horizon_status(status: &Value) -> AppResult<RetestHorizonStatusValidation> {
    let schema_version = status
        .get("schema_version")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if schema_version != RETEST_HORIZON_STATUS_SCHEMA_VERSION {
        return Err(AppError::validation(format!(
            "retest horizon status schema_version must be {RETEST_HORIZON_STATUS_SCHEMA_VERSION}; got {schema_version}"
        )));
    }

    validate_local_safety(status)?;
    validate_live_boundaries(status)?;
    validate_blocked_actions(status)?;

    let verdict = status
        .pointer("/next_decision/verdict")
        .and_then(Value::as_str)
        .or_else(|| status.get("verdict").and_then(Value::as_str))
        .unwrap_or_default();
    let scheduler_hint = status
        .pointer("/next_decision/scheduler_hint")
        .ok_or_else(|| AppError::validation("retest horizon status missing scheduler_hint"))?;

    let run_now_replay_ready = scheduler_hint
        .get("run_now_replay_ready")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let promotion_ready_for_review = scheduler_hint
        .get("promotion_ready_for_review")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let run_not_before_ms = scheduler_hint
        .get("run_research_after_l1_as_of_ms")
        .and_then(Value::as_i64);

    if verdict == "WAIT_FOR_MARKET_L1_HORIZON" {
        let Some(run_not_before_ms) = run_not_before_ms else {
            return Err(AppError::validation(
                "WAIT_FOR_MARKET_L1_HORIZON requires scheduler_hint.run_research_after_l1_as_of_ms",
            ));
        };
        if run_now_replay_ready {
            return Err(AppError::validation(
                "WAIT_FOR_MARKET_L1_HORIZON must not set run_now_replay_ready=true",
            ));
        }
        if promotion_ready_for_review {
            return Err(AppError::validation(
                "WAIT_FOR_MARKET_L1_HORIZON must not set promotion_ready_for_review=true",
            ));
        }
        return Ok(RetestHorizonStatusValidation {
            scheduler_action: "WAIT_UNTIL_MARKET_L1_HORIZON_MATERIALIZES".to_owned(),
            run_not_before_ms: Some(run_not_before_ms),
        });
    }

    if run_now_replay_ready {
        return Ok(RetestHorizonStatusValidation {
            scheduler_action: "RUN_FOCUSED_RETEST_RESEARCH".to_owned(),
            run_not_before_ms: None,
        });
    }

    if promotion_ready_for_review
        || status
            .pointer("/stage_state/promotion_passed")
            .and_then(Value::as_bool)
            .unwrap_or(false)
    {
        return Ok(RetestHorizonStatusValidation {
            scheduler_action: "REVIEW_PROMOTION_EVIDENCE".to_owned(),
            run_not_before_ms: None,
        });
    }

    Ok(RetestHorizonStatusValidation {
        scheduler_action: "HOLD_FOR_OPERATOR_REVIEW".to_owned(),
        run_not_before_ms: None,
    })
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn validates_wait_status_handoff() {
        let status = json!({
            "schema_version": "research_horizon_status_checkpoint_v1",
            "safety": {
                "s3_write": false,
                "ecs_task_started": false,
                "dispatcher_mode_changed": false,
                "local_summary_only": true,
                "shadow_paper_live_enabled": false
            },
            "stage_state": {
                "promotion_passed": false,
                "paper_created": false,
                "live_enabled": false
            },
            "next_decision": {
                "verdict": "WAIT_FOR_MARKET_L1_HORIZON",
                "scheduler_hint": {
                    "latest_l1_as_of_ms": 1779710400000_i64,
                    "run_research_after_l1_as_of_ms": 1779719361452_i64,
                    "run_now_replay_ready": false,
                    "promotion_ready_for_review": false
                },
                "blocked_actions": [
                    "do_not_create_shadow_without_promotion",
                    "do_not_create_paper_without_passed_shadow",
                    "do_not_enable_live_from_research_batch"
                ]
            }
        });

        let summary = validate_retest_horizon_status(&status).expect("status validates");
        assert_eq!(
            summary.scheduler_action,
            "WAIT_UNTIL_MARKET_L1_HORIZON_MATERIALIZES"
        );
        assert_eq!(summary.run_not_before_ms, Some(1779719361452));
    }

    #[test]
    fn rejects_wait_status_without_not_before_time() {
        let status = json!({
            "schema_version": "research_horizon_status_checkpoint_v1",
            "safety": {
                "s3_write": false,
                "ecs_task_started": false,
                "dispatcher_mode_changed": false,
                "local_summary_only": true,
                "shadow_paper_live_enabled": false
            },
            "stage_state": {
                "live_enabled": false
            },
            "next_decision": {
                "verdict": "WAIT_FOR_MARKET_L1_HORIZON",
                "scheduler_hint": {
                    "run_now_replay_ready": false,
                    "promotion_ready_for_review": false
                },
                "blocked_actions": [
                    "do_not_create_shadow_without_promotion",
                    "do_not_create_paper_without_passed_shadow",
                    "do_not_enable_live_from_research_batch"
                ]
            }
        });

        let error = validate_retest_horizon_status(&status).expect_err("wait time is required");
        assert!(error.to_string().contains("run_research_after_l1_as_of_ms"));
    }

    #[test]
    fn rejects_status_that_enables_live() {
        let status = json!({
            "schema_version": "research_horizon_status_checkpoint_v1",
            "safety": {
                "s3_write": false,
                "ecs_task_started": false,
                "dispatcher_mode_changed": false,
                "local_summary_only": true,
                "shadow_paper_live_enabled": false
            },
            "stage_state": {
                "live_enabled": true
            },
            "next_decision": {
                "verdict": "INSPECT_REMAINING_GATE_REASONS",
                "scheduler_hint": {},
                "blocked_actions": [
                    "do_not_create_shadow_without_promotion",
                    "do_not_create_paper_without_passed_shadow",
                    "do_not_enable_live_from_research_batch"
                ]
            }
        });

        let error = validate_retest_horizon_status(&status).expect_err("live is rejected");
        assert!(error.to_string().contains("live trading"));
    }
}

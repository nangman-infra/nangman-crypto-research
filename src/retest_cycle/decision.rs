use crate::error::{AppError, AppResult};
use serde::Serialize;
use serde_json::Value;

use super::safety::validate_status_safety;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RetestHorizonStatusValidation {
    pub scheduler_action: String,
    pub run_not_before_ms: Option<i64>,
}

pub fn validate_retest_horizon_status(status: &Value) -> AppResult<RetestHorizonStatusValidation> {
    validate_status_safety(status)?;

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

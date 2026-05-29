use crate::error::{AppError, AppResult};
use serde_json::Value;

use super::fields::string_field;

pub(in crate::retest_status) fn validate_plan(plan: &Value) -> AppResult<()> {
    let rows = plan
        .get("horizon_rows")
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::validation("retest horizon plan missing horizon_rows"))?;
    for (index, row) in rows.iter().enumerate() {
        for field in ["candidate_id", "horizon", "next_action"] {
            if string_field(row, field).is_none() {
                return Err(AppError::validation(format!(
                    "retest horizon plan horizon_rows[{index}] missing {field}"
                )));
            }
        }
    }
    Ok(())
}

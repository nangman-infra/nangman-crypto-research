use std::fs;
use std::path::Path;

use crate::error::{AppError, AppResult};
use crate::model::ShadowCycleDecision;

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

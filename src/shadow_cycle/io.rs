use std::fs;
use std::path::Path;

use crate::error::AppResult;
use crate::model::ShadowCycleDecision;
use crate::path_validation::validate_config_absolute_path;

pub fn read_shadow_cycle_decision(path: &Path) -> AppResult<ShadowCycleDecision> {
    validate_config_absolute_path(path, "shadow cycle decision file")?;
    let raw = fs::read_to_string(path)?;
    let decision = serde_json::from_str(&raw)?;
    Ok(decision)
}

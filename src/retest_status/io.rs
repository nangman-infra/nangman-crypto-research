use crate::error::{AppError, AppResult};
use crate::path_validation::validate_config_absolute_path;
use serde_json::Value;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetestHorizonStatusBuildOptions {
    pub generated_at_ms: i64,
    pub plan_file: Option<String>,
    pub driver_summary_file: Option<String>,
    pub checkpoint_s3_write: bool,
}

pub fn read_retest_horizon_plan(path: &Path) -> AppResult<Value> {
    validate_config_absolute_path(path, "retest horizon plan file")?;
    let raw = fs::read_to_string(path)?;
    read_retest_horizon_plan_from_bytes(&path.display().to_string(), raw.as_bytes())
}

pub fn read_retest_horizon_plan_from_bytes(label: &str, bytes: &[u8]) -> AppResult<Value> {
    let text =
        std::str::from_utf8(bytes).map_err(|error| AppError::Json(format!("{label}: {error}")))?;
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(AppError::validation(format!("{label} must not be empty")));
    }
    Ok(serde_json::from_str(trimmed)?)
}

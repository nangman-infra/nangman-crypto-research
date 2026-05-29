use crate::error::{AppError, AppResult};
use std::env;
use std::path::PathBuf;

pub(super) fn absolute_path_arg(value: Option<String>, message: &str) -> AppResult<PathBuf> {
    let value = value.ok_or_else(|| AppError::config(message))?;
    let path = PathBuf::from(value);
    if !path.is_absolute() {
        return Err(AppError::config(format!(
            "{message}; got {}",
            path.display()
        )));
    }
    Ok(path)
}

pub(super) fn non_empty_arg(value: Option<String>, message: &str) -> AppResult<String> {
    let value = value.ok_or_else(|| AppError::config(message))?;
    if value.trim().is_empty() {
        return Err(AppError::config(message));
    }
    Ok(value)
}

pub(super) fn env_string(name: &str) -> Option<String> {
    env::var(name).ok().filter(|value| !value.trim().is_empty())
}

pub(super) fn env_bool(name: &str) -> bool {
    env_string(name)
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "y"
            )
        })
        .unwrap_or(false)
}

pub(super) fn env_non_negative_i64(name: &str) -> AppResult<Option<i64>> {
    let Some(raw) = env_string(name) else {
        return Ok(None);
    };
    parse_non_negative_i64(name, &raw).map(Some)
}

pub(super) fn parse_non_negative_i64(name: &str, raw: &str) -> AppResult<i64> {
    let value = raw
        .parse::<i64>()
        .map_err(|_| AppError::config(format!("{name} must be an integer")))?;
    if value < 0 {
        return Err(AppError::config(format!("{name} must be non-negative")));
    }
    Ok(value)
}

pub(super) fn env_usize(name: &str, fallback: usize) -> AppResult<usize> {
    let Some(raw) = env_string(name) else {
        return Ok(fallback);
    };
    parse_positive_usize(name, &raw)
}

pub(super) fn env_usize_allow_zero(name: &str, fallback: usize) -> AppResult<usize> {
    let Some(raw) = env_string(name) else {
        return Ok(fallback);
    };
    parse_non_negative_usize(name, &raw)
}

pub(super) fn env_u64(name: &str, fallback: u64) -> AppResult<u64> {
    let Some(raw) = env_string(name) else {
        return Ok(fallback);
    };
    parse_positive_u64(name, &raw)
}

pub(super) fn parse_positive_usize(name: &str, raw: &str) -> AppResult<usize> {
    let value = raw
        .parse::<usize>()
        .map_err(|_| AppError::config(format!("{name} must be a positive integer")))?;
    if value == 0 {
        return Err(AppError::config(format!(
            "{name} must be greater than zero"
        )));
    }
    Ok(value)
}

pub(super) fn parse_non_negative_usize(name: &str, raw: &str) -> AppResult<usize> {
    raw.parse::<usize>()
        .map_err(|_| AppError::config(format!("{name} must be a non-negative integer")))
}

pub(super) fn parse_positive_u64(name: &str, raw: &str) -> AppResult<u64> {
    let value = raw
        .parse::<u64>()
        .map_err(|_| AppError::config(format!("{name} must be a positive integer")))?;
    if value == 0 {
        return Err(AppError::config(format!(
            "{name} must be greater than zero"
        )));
    }
    Ok(value)
}

pub(super) fn env_list(name: &str) -> Vec<String> {
    env::var(name)
        .ok()
        .map(|value| {
            value
                .split([',', '\n'])
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

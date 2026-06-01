use super::input::trimmed_utf8;
use crate::error::{AppError, AppResult};
use serde::de::DeserializeOwned;

pub(in crate::io) fn read_json_array_or_jsonl_bytes<T>(
    label: &str,
    bytes: &[u8],
) -> AppResult<Vec<T>>
where
    T: DeserializeOwned,
{
    let trimmed = trimmed_utf8(label, bytes)?;
    if trimmed.starts_with('[') {
        return Ok(serde_json::from_str(trimmed)?);
    }
    if trimmed.starts_with('{')
        && let Ok(value) = serde_json::from_str(trimmed)
    {
        return Ok(vec![value]);
    }

    let mut values = Vec::new();
    for (index, line) in trimmed.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        values.push(serde_json::from_str(line).map_err(|error| {
            AppError::Json(format!(
                "{label} line {} is not valid JSON: {error}",
                index + 1
            ))
        })?);
    }
    Ok(values)
}

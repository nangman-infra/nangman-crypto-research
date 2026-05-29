use super::super::partition::partition;
use crate::error::AppResult;

pub(in crate::storage::market_l1) fn l1_index_pointer_key(
    window_start_ms: i64,
) -> AppResult<String> {
    let part = partition(window_start_ms)?;
    Ok(format!(
        "l1_index/window_ms=1000/event_date={}/hour={:02}/window_start_ms={window_start_ms}.json",
        part.date, part.hour
    ))
}

pub(in crate::storage::market_l1) fn l1_manifest_key_from_pointer(
    pointer: &serde_json::Value,
) -> Option<String> {
    string_field(pointer, "canonical_manifest_key")
        .or_else(|| string_field(pointer, "manifest_key"))
        .and_then(normalize_s3_key)
}

pub(in crate::storage::market_l1) fn l1_artifact_key_from_manifest(
    manifest: &serde_json::Value,
    manifest_key_field: &str,
) -> Option<String> {
    string_field(manifest, manifest_key_field).and_then(normalize_s3_key)
}

pub(in crate::storage::market_l1) fn is_success_l1_index_pointer(
    pointer: &serde_json::Value,
) -> bool {
    pointer
        .get("schema_version")
        .and_then(serde_json::Value::as_str)
        == Some("l1_index_pointer_v1")
        && pointer
            .get("status")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|status| status.eq_ignore_ascii_case("success"))
}

pub(in crate::storage::market_l1) fn is_success_l1_manifest(manifest: &serde_json::Value) -> bool {
    manifest
        .get("schema_version")
        .and_then(serde_json::Value::as_str)
        == Some("l1_manifest_v1")
        && manifest
            .get("status")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|status| status.eq_ignore_ascii_case("success"))
}

fn string_field<'a>(value: &'a serde_json::Value, field: &str) -> Option<&'a str> {
    value
        .get(field)
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn normalize_s3_key(value: &str) -> Option<String> {
    let trimmed = value.trim().trim_start_matches('/');
    if trimmed.is_empty() {
        return None;
    }
    if let Some(uri_without_scheme) = trimmed.strip_prefix("s3://") {
        let (_, key) = uri_without_scheme.split_once('/')?;
        let key = key.trim_start_matches('/').trim();
        return (!key.is_empty()).then(|| key.to_owned());
    }
    Some(trimmed.to_owned())
}

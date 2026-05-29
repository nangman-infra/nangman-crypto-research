use super::*;

pub(in crate::cli) fn insert_normalized_s3_key(keys: &mut BTreeSet<String>, value: &str) {
    if let Some(key) = normalize_s3_key(value) {
        keys.insert(key);
    }
}

pub(in crate::cli) fn normalize_s3_key(value: &str) -> Option<String> {
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

pub(in crate::cli) fn market_l1_run_id_from_key(value: &str) -> Option<String> {
    let key = normalize_s3_key(value)?;
    let marker = "run_id=";
    let start = key.find(marker)? + marker.len();
    let remainder = &key[start..];
    let end = remainder.find('/').unwrap_or(remainder.len());
    let run_id = remainder[..end].trim();
    (!run_id.is_empty()).then(|| run_id.to_owned())
}

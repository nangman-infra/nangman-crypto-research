use super::*;

pub(super) enum ReplayRunLocation {
    Local(PathBuf),
    S3 { bucket: String, key: String },
}

pub(super) fn replay_run_location(record: &ReplayRunIndexRecord) -> AppResult<ReplayRunLocation> {
    if let (Some(bucket), Some(key)) = (
        record.replay_run_s3_bucket.as_deref(),
        record.replay_run_s3_key.as_deref(),
    ) {
        return s3_location(bucket, key)
            .map(|location| ReplayRunLocation::S3 {
                bucket: location.bucket,
                key: location.key,
            })
            .ok_or_else(|| {
                AppError::config(format!(
                    "replay_run_index has invalid replay_run_s3_bucket/replay_run_s3_key for replay_run_id={}",
                    record.replay_run_id
                ))
            });
    }
    if record.replay_run_uri.starts_with("s3://") {
        return parse_s3_uri(&record.replay_run_uri)
            .map(|(bucket, key)| ReplayRunLocation::S3 { bucket, key })
            .ok_or_else(|| {
                AppError::config(format!(
                    "replay_run_index replay_run_uri is not a valid S3 URI: {}",
                    record.replay_run_uri
                ))
            });
    }

    let path = PathBuf::from(&record.replay_run_uri);
    crate::path_validation::validate_config_absolute_path(&path, "replay_run_index replay_run_uri")
        .map_err(|error| {
            AppError::config(format!(
                "replay_run_index replay_run_uri must be an absolute path or s3 URI: {}; {error}",
                record.replay_run_uri
            ))
        })?;
    Ok(ReplayRunLocation::Local(path))
}

pub(in crate::cli) fn parse_s3_uri(value: &str) -> Option<(String, String)> {
    let rest = value.strip_prefix("s3://")?;
    let (bucket, key) = rest.split_once('/')?;
    s3_location(bucket, key).map(|location| (location.bucket, location.key))
}

struct S3Location {
    bucket: String,
    key: String,
}

fn s3_location(bucket: &str, key: &str) -> Option<S3Location> {
    if !valid_bucket_name(bucket) || !valid_object_key(key) {
        return None;
    }
    Some(S3Location {
        bucket: bucket.to_owned(),
        key: key.to_owned(),
    })
}

fn valid_bucket_name(bucket: &str) -> bool {
    let bytes = bucket.as_bytes();
    if !(3..=63).contains(&bytes.len()) {
        return false;
    }
    if !bytes[0].is_ascii_alphanumeric() || !bytes[bytes.len() - 1].is_ascii_alphanumeric() {
        return false;
    }
    if bucket.contains("..") || looks_like_ipv4_address(bucket) {
        return false;
    }
    if bucket.starts_with("xn--")
        || bucket.starts_with("sthree-")
        || bucket.starts_with("amzn-s3-demo-")
        || bucket.ends_with("-s3alias")
        || bucket.ends_with("--ol-s3")
        || bucket.ends_with(".mrap")
        || bucket.ends_with("--x-s3")
        || bucket.ends_with("--table-s3")
    {
        return false;
    }
    bytes.iter().all(|byte| {
        byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-')
    })
}

fn valid_object_key(key: &str) -> bool {
    if key.is_empty() || key.len() > 1024 {
        return false;
    }
    if key.chars().any(char::is_control) {
        return false;
    }
    !key.split('/').any(|segment| matches!(segment, "." | ".."))
}

fn looks_like_ipv4_address(value: &str) -> bool {
    let parts = value.split('.').collect::<Vec<_>>();
    parts.len() == 4
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.parse::<u8>().is_ok())
}

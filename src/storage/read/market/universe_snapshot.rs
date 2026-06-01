use super::super::super::{AppError, AppResult, aws_error_detail, s3_client};

pub async fn discover_latest_symbol_universe_snapshot_end_ms_from_s3(
    bucket: &str,
) -> AppResult<Option<i64>> {
    if bucket.trim().is_empty() {
        return Err(AppError::config("market L1 S3 bucket must not be empty"));
    }
    let client = s3_client().await?;
    let prefix = "symbol_universe_snapshot/run_id=";
    let mut latest: Option<(i64, i64, String)> = None;
    let mut continuation_token: Option<String> = None;

    loop {
        let mut request = client.list_objects_v2().bucket(bucket).prefix(prefix);
        if let Some(token) = continuation_token.as_deref() {
            request = request.continuation_token(token);
        }
        let output = request.send().await.map_err(|error| {
            AppError::Aws(format!(
                "s3 list_objects_v2 s3://{bucket}/{prefix}: {}",
                aws_error_detail(&error)
            ))
        })?;

        for object in output.contents() {
            let Some(key) = object.key() else {
                continue;
            };
            let Some(run_end_ms) = parse_l1_run_end_ms(key) else {
                continue;
            };
            let last_modified_ms = object
                .last_modified()
                .and_then(|last_modified| last_modified.to_millis().ok())
                .unwrap_or(0);
            let candidate = (run_end_ms, last_modified_ms, key.to_owned());
            if latest.as_ref().is_none_or(|current| candidate > *current) {
                latest = Some(candidate);
            }
        }

        continuation_token = output.next_continuation_token().map(ToOwned::to_owned);
        if continuation_token.is_none() {
            break;
        }
    }

    Ok(latest.map(|(run_end_ms, _, _)| run_end_ms))
}

fn parse_l1_run_end_ms(key: &str) -> Option<i64> {
    let run_part = key
        .split('/')
        .find_map(|part| part.strip_prefix("run_id=l1_"))?;
    let mut parts = run_part.split('_');
    let _start_ms = parts.next()?;
    parts.next()?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::parse_l1_run_end_ms;

    #[test]
    fn parses_l1_run_end_ms_from_symbol_universe_snapshot_key() {
        let key = "symbol_universe_snapshot/run_id=l1_1700000000000_1700003600000/snapshot.json";

        assert_eq!(parse_l1_run_end_ms(key), Some(1_700_003_600_000));
    }

    #[test]
    fn skips_keys_without_l1_run_end_ms() {
        for key in [
            "symbol_universe_snapshot/run_id=manual_1700000000000/snapshot.json",
            "symbol_universe_snapshot/run_id=l1_1700000000000_bad/snapshot.json",
            "symbol_universe_snapshot/snapshot.json",
        ] {
            assert_eq!(parse_l1_run_end_ms(key), None);
        }
    }
}

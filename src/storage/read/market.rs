use super::super::{
    AppError, AppResult, MarketFeatureDelta, MarketRegimeContext, aws_error_detail,
    discover_latest_market_l1_keys_from_s3, get_object_bytes, is_missing_market_artifact,
    read_market_feature_deltas_matching_symbols_from_bytes, read_market_regime_contexts_from_bytes,
    s3_client,
};
use std::collections::BTreeSet;

pub async fn read_market_feature_deltas_from_s3(
    bucket: &str,
    keys: &[String],
    symbols: &BTreeSet<String>,
) -> AppResult<Vec<MarketFeatureDelta>> {
    let client = s3_client().await?;
    let mut deltas = Vec::new();
    for key in keys {
        let bytes = match get_object_bytes(&client, bucket, key).await {
            Ok(bytes) => bytes,
            Err(error) if is_missing_market_artifact(&error) => continue,
            Err(error) => return Err(error),
        };
        deltas.extend(read_market_feature_deltas_matching_symbols_from_bytes(
            &format!("s3://{bucket}/{key}"),
            bytes.as_ref(),
            symbols,
        )?);
    }
    Ok(deltas)
}

pub async fn discover_latest_market_feature_delta_keys_from_s3(
    bucket: &str,
    window_starts_ms: &[i64],
) -> AppResult<Vec<String>> {
    discover_latest_market_l1_keys_from_s3(
        bucket,
        window_starts_ms,
        "market_feature_delta",
        "/delta.json",
        "market_feature_delta_key",
    )
    .await
}

pub async fn read_market_regime_contexts_from_s3(
    bucket: &str,
    keys: &[String],
) -> AppResult<Vec<MarketRegimeContext>> {
    let client = s3_client().await?;
    let mut contexts = Vec::new();
    for key in keys {
        let bytes = match get_object_bytes(&client, bucket, key).await {
            Ok(bytes) => bytes,
            Err(error) if is_missing_market_artifact(&error) => continue,
            Err(error) => return Err(error),
        };
        contexts.extend(read_market_regime_contexts_from_bytes(
            &format!("s3://{bucket}/{key}"),
            bytes.as_ref(),
        )?);
    }
    Ok(contexts)
}

pub async fn discover_latest_market_regime_context_keys_from_s3(
    bucket: &str,
    window_starts_ms: &[i64],
) -> AppResult<Vec<String>> {
    discover_latest_market_l1_keys_from_s3(
        bucket,
        window_starts_ms,
        "market_regime_context",
        "/context.json",
        "market_regime_context_key",
    )
    .await
}

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

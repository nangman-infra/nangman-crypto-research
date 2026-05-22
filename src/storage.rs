use crate::artifacts::{build_replay_run_index_records, build_research_aggregate_registry_records};
use crate::error::{AppError, AppResult};
use crate::io::{
    read_candidate_bundles_from_bytes, read_market_feature_deltas_from_bytes,
    read_market_regime_contexts_from_bytes, read_oss_adapter_runs_from_bytes,
    read_replay_run_index_records_from_bytes, read_replay_runs_from_bytes,
    read_research_input_manifest_from_bytes,
};
use crate::model::{
    IntelCandidateEvidenceBundle, MarketFeatureDelta, MarketRegimeContext, OssAdapterRun,
    ReplayRun, ReplayRunIndexRecord, ResearchInputManifest, ResearchRunReport, ShadowValidationRun,
};
use aws_config::BehaviorVersion;
use aws_sdk_s3::Client;
use aws_sdk_s3::config::Builder as S3ConfigBuilder;
use aws_types::region::Region;
use chrono::{DateTime, Datelike, Timelike, Utc};
use std::env;

pub async fn read_candidate_bundles_from_s3(
    bucket: &str,
    key: &str,
) -> AppResult<Vec<IntelCandidateEvidenceBundle>> {
    let client = s3_client().await;
    let bytes = get_object_bytes(&client, bucket, key).await?;
    read_candidate_bundles_from_bytes(&format!("s3://{bucket}/{key}"), bytes.as_ref())
}

pub async fn read_research_input_manifest_from_s3(
    bucket: &str,
    key: &str,
) -> AppResult<ResearchInputManifest> {
    let client = s3_client().await;
    let bytes = get_object_bytes(&client, bucket, key).await?;
    read_research_input_manifest_from_bytes(&format!("s3://{bucket}/{key}"), bytes.as_ref())
}

pub async fn read_market_feature_deltas_from_s3(
    bucket: &str,
    keys: &[String],
) -> AppResult<Vec<MarketFeatureDelta>> {
    let client = s3_client().await;
    let mut deltas = Vec::new();
    for key in keys {
        let bytes = get_object_bytes(&client, bucket, key).await?;
        deltas.extend(read_market_feature_deltas_from_bytes(
            &format!("s3://{bucket}/{key}"),
            bytes.as_ref(),
        )?);
    }
    Ok(deltas)
}

pub async fn read_market_regime_contexts_from_s3(
    bucket: &str,
    keys: &[String],
) -> AppResult<Vec<MarketRegimeContext>> {
    let client = s3_client().await;
    let mut contexts = Vec::new();
    for key in keys {
        let bytes = get_object_bytes(&client, bucket, key).await?;
        contexts.extend(read_market_regime_contexts_from_bytes(
            &format!("s3://{bucket}/{key}"),
            bytes.as_ref(),
        )?);
    }
    Ok(contexts)
}

pub async fn read_replay_runs_from_s3(bucket: &str, keys: &[String]) -> AppResult<Vec<ReplayRun>> {
    let client = s3_client().await;
    let mut runs = Vec::new();
    for key in keys {
        let bytes = get_object_bytes(&client, bucket, key).await?;
        runs.extend(read_replay_runs_from_bytes(
            &format!("s3://{bucket}/{key}"),
            bytes.as_ref(),
        )?);
    }
    Ok(runs)
}

pub async fn read_replay_run_index_records_from_s3(
    bucket: &str,
    keys: &[String],
) -> AppResult<Vec<ReplayRunIndexRecord>> {
    let client = s3_client().await;
    let mut records = Vec::new();
    for key in keys {
        let bytes = get_object_bytes(&client, bucket, key).await?;
        records.extend(read_replay_run_index_records_from_bytes(
            &format!("s3://{bucket}/{key}"),
            bytes.as_ref(),
        )?);
    }
    Ok(records)
}

pub async fn read_oss_adapter_runs_from_s3(
    bucket: &str,
    keys: &[String],
) -> AppResult<Vec<OssAdapterRun>> {
    let client = s3_client().await;
    let mut runs = Vec::new();
    for key in keys {
        let bytes = get_object_bytes(&client, bucket, key).await?;
        runs.extend(read_oss_adapter_runs_from_bytes(
            &format!("s3://{bucket}/{key}"),
            bytes.as_ref(),
        )?);
    }
    Ok(runs)
}

pub async fn write_research_outputs_to_s3(
    bucket: &str,
    prefix: &str,
    report: &ResearchRunReport,
    replay_runs: &[ReplayRun],
    shadow_validation_runs: &[ShadowValidationRun],
    output_partition_at_ms: i64,
) -> AppResult<Vec<String>> {
    if bucket.trim().is_empty() {
        return Err(AppError::config(
            "research output S3 bucket must not be empty",
        ));
    }
    let client = s3_client().await;
    let mut written = Vec::new();
    let dt = partition(output_partition_at_ms)?;
    let prefix = normalize_prefix(prefix);
    let report_key = format!(
        "{prefix}research-run-report/schema={}/dt={}/hour={:02}/research_run_report_id={}/report.json",
        report.schema_version, dt.date, dt.hour, report.research_run_report_id
    );
    put_object_json(&client, bucket, &report_key, report).await?;
    written.push(format!("s3://{bucket}/{report_key}"));

    if !replay_runs.is_empty() {
        let replay_key = format!(
            "{prefix}replay-run/schema={}/dt={}/hour={:02}/research_run_report_id={}/part-000001.jsonl",
            replay_runs[0].schema_version, dt.date, dt.hour, report.research_run_report_id
        );
        let mut body = Vec::new();
        for run in replay_runs {
            serde_json::to_writer(&mut body, run)?;
            body.push(b'\n');
        }
        put_object_bytes(&client, bucket, &replay_key, body, "application/x-ndjson").await?;
        written.push(format!("s3://{bucket}/{replay_key}"));

        let replay_run_uri = format!("s3://{bucket}/{replay_key}");
        let replay_run_index_records = build_replay_run_index_records(
            report,
            replay_runs,
            &replay_run_uri,
            Some(bucket),
            Some(&replay_key),
        );
        let replay_index_key = format!(
            "{prefix}replay-run-index/schema={}/dt={}/hour={:02}/research_run_report_id={}/part-000001.jsonl",
            replay_run_index_records[0].schema_version,
            dt.date,
            dt.hour,
            report.research_run_report_id
        );
        let mut replay_index_body = Vec::new();
        for record in &replay_run_index_records {
            serde_json::to_writer(&mut replay_index_body, record)?;
            replay_index_body.push(b'\n');
        }
        put_object_bytes(
            &client,
            bucket,
            &replay_index_key,
            replay_index_body,
            "application/x-ndjson",
        )
        .await?;
        written.push(format!("s3://{bucket}/{replay_index_key}"));
    }

    if !shadow_validation_runs.is_empty() {
        let shadow_key = format!(
            "{prefix}shadow-validation-run/schema={}/dt={}/hour={:02}/research_run_report_id={}/part-000001.jsonl",
            shadow_validation_runs[0].schema_version,
            dt.date,
            dt.hour,
            report.research_run_report_id
        );
        let mut body = Vec::new();
        for run in shadow_validation_runs {
            serde_json::to_writer(&mut body, run)?;
            body.push(b'\n');
        }
        put_object_bytes(&client, bucket, &shadow_key, body, "application/x-ndjson").await?;
        written.push(format!("s3://{bucket}/{shadow_key}"));
    }

    if let Some(snapshot) = report.portfolio_allocation_snapshot.as_ref() {
        let snapshot_key = format!(
            "{prefix}portfolio-allocation-snapshot/schema={}/dt={}/hour={:02}/research_run_report_id={}/snapshot.json",
            snapshot.schema_version, dt.date, dt.hour, report.research_run_report_id
        );
        put_object_json(&client, bucket, &snapshot_key, snapshot).await?;
        written.push(format!("s3://{bucket}/{snapshot_key}"));
    }

    if !report.portfolio_risk_reject_events.is_empty() {
        let reject_key = format!(
            "{prefix}portfolio-risk-reject-event/schema={}/dt={}/hour={:02}/research_run_report_id={}/part-000001.jsonl",
            report.portfolio_risk_reject_events[0].schema_version,
            dt.date,
            dt.hour,
            report.research_run_report_id
        );
        let mut body = Vec::new();
        for record in &report.portfolio_risk_reject_events {
            serde_json::to_writer(&mut body, record)?;
            body.push(b'\n');
        }
        put_object_bytes(&client, bucket, &reject_key, body, "application/x-ndjson").await?;
        written.push(format!("s3://{bucket}/{reject_key}"));
    }

    if !report.portfolio_reduce_only_signals.is_empty() {
        let reduce_key = format!(
            "{prefix}portfolio-reduce-only-signal/schema={}/dt={}/hour={:02}/research_run_report_id={}/part-000001.jsonl",
            report.portfolio_reduce_only_signals[0].schema_version,
            dt.date,
            dt.hour,
            report.research_run_report_id
        );
        let mut body = Vec::new();
        for record in &report.portfolio_reduce_only_signals {
            serde_json::to_writer(&mut body, record)?;
            body.push(b'\n');
        }
        put_object_bytes(&client, bucket, &reduce_key, body, "application/x-ndjson").await?;
        written.push(format!("s3://{bucket}/{reduce_key}"));
    }

    let registry_records = build_research_aggregate_registry_records(report);
    if !registry_records.is_empty() {
        let registry_key = format!(
            "{prefix}research-aggregate-registry/schema={}/dt={}/hour={:02}/research_run_report_id={}/part-000001.jsonl",
            registry_records[0].schema_version, dt.date, dt.hour, report.research_run_report_id
        );
        let mut registry_body = Vec::new();
        for record in &registry_records {
            serde_json::to_writer(&mut registry_body, record)?;
            registry_body.push(b'\n');
        }
        put_object_bytes(
            &client,
            bucket,
            &registry_key,
            registry_body,
            "application/x-ndjson",
        )
        .await?;
        written.push(format!("s3://{bucket}/{registry_key}"));
    }

    Ok(written)
}

async fn get_object_bytes(client: &Client, bucket: &str, key: &str) -> AppResult<Vec<u8>> {
    validate_s3_location(bucket, key, "S3")?;
    let output = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await
        .map_err(|error| {
            AppError::Aws(format!(
                "s3 get_object s3://{bucket}/{key}: {}",
                aws_error_detail(&error)
            ))
        })?;
    let bytes = output
        .body
        .collect()
        .await
        .map_err(|error| {
            AppError::Aws(format!(
                "s3 read body s3://{bucket}/{key}: {}",
                aws_error_detail(&error)
            ))
        })?
        .into_bytes()
        .to_vec();
    Ok(bytes)
}

fn validate_s3_location(bucket: &str, key: &str, label: &str) -> AppResult<()> {
    if bucket.trim().is_empty() {
        return Err(AppError::config(format!(
            "{label} bucket must not be empty"
        )));
    }
    if key.trim().is_empty() {
        return Err(AppError::config(format!("{label} key must not be empty")));
    }
    Ok(())
}

async fn s3_client() -> Client {
    let mut loader = aws_config::defaults(BehaviorVersion::latest());
    if let Some(region) = env_string("AWS_REGION").or_else(|| env_string("AWS_DEFAULT_REGION")) {
        loader = loader.region(Region::new(region));
    }
    if let Some(endpoint) = env_s3_endpoint() {
        loader = loader.endpoint_url(endpoint);
    }
    let config = loader.load().await;
    let s3_config = S3ConfigBuilder::from(&config)
        .force_path_style(
            env_bool("AWS_S3_FORCE_PATH_STYLE") || env_bool("AWS_USE_PATH_STYLE_ENDPOINT"),
        )
        .build();
    Client::from_conf(s3_config)
}

fn env_s3_endpoint() -> Option<String> {
    env::var("AWS_ENDPOINT_URL_S3")
        .ok()
        .or_else(|| env::var("AWS_ENDPOINT_URL").ok())
        .map(|value| value.trim().trim_end_matches('/').to_owned())
        .filter(|value| !value.is_empty())
}

fn env_string(name: &str) -> Option<String> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn env_bool(name: &str) -> bool {
    env::var(name)
        .ok()
        .map(|value| matches!(value.to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
        .unwrap_or(false)
}

async fn put_object_json<T>(client: &Client, bucket: &str, key: &str, value: &T) -> AppResult<()>
where
    T: serde::Serialize,
{
    let body = serde_json::to_vec_pretty(value)?;
    put_object_bytes(client, bucket, key, body, "application/json").await
}

async fn put_object_bytes(
    client: &Client,
    bucket: &str,
    key: &str,
    body: Vec<u8>,
    content_type: &str,
) -> AppResult<()> {
    client
        .put_object()
        .bucket(bucket)
        .key(key)
        .content_type(content_type)
        .body(body.into())
        .send()
        .await
        .map_err(|error| {
            AppError::aws(format!(
                "s3 put_object s3://{bucket}/{key}: {}",
                aws_error_detail(&error)
            ))
        })?;
    Ok(())
}

fn aws_error_detail(error: &(impl std::fmt::Debug + std::fmt::Display)) -> String {
    let display = error.to_string();
    let debug = format!("{error:?}");
    if debug == display {
        display
    } else {
        format!("{display}; debug={debug}")
    }
}

struct Partition {
    date: String,
    hour: u32,
}

fn partition(timestamp_ms: i64) -> AppResult<Partition> {
    let dt = DateTime::<Utc>::from_timestamp_millis(timestamp_ms)
        .ok_or_else(|| AppError::validation(format!("invalid timestamp_ms: {timestamp_ms}")))?;
    Ok(Partition {
        date: format!("{:04}-{:02}-{:02}", dt.year(), dt.month(), dt.day()),
        hour: dt.hour(),
    })
}

fn normalize_prefix(prefix: &str) -> String {
    let trimmed = prefix.trim().trim_start_matches('/');
    if trimmed.is_empty() {
        return String::new();
    }
    format!("{}/", trimmed.trim_end_matches('/'))
}

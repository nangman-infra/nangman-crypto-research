use crate::artifacts::{build_replay_run_index_records, build_research_aggregate_registry_records};
use crate::error::{AppError, AppResult};
use crate::io::{
    ResearchOutputArtifacts, read_candidate_bundles_from_bytes,
    read_market_feature_deltas_matching_symbols_from_bytes, read_market_live_ticks_from_bytes,
    read_market_regime_contexts_from_bytes, read_oss_adapter_runs_from_bytes,
    read_paper_watch_candidates_from_bytes, read_paper_watch_live_marks_from_bytes,
    read_replay_run_index_records_from_bytes, read_replay_runs_from_bytes,
    read_research_input_manifest_from_bytes, read_research_run_report_from_bytes,
    read_shadow_validation_runs_from_bytes,
};
use crate::model::{
    IntelCandidateEvidenceBundle, MarketFeatureDelta, MarketLiveTick, MarketRegimeContext,
    OssAdapterRun, PaperWatchCandidate, PaperWatchLiveMark,
    RETEST_CYCLE_SOURCE_STATE_SCHEMA_VERSION, ReplayRun, ReplayRunIndexRecord,
    ResearchInputManifest, ResearchRunReport, RetestCycleSourceState, ShadowCycleDecision,
    ShadowValidationRun,
};
use crate::retest_cycle::read_retest_horizon_status_from_bytes;
use crate::retest_status::read_retest_horizon_plan_from_bytes;
use aws_config::BehaviorVersion;
use aws_sdk_s3::Client;
use aws_sdk_s3::error::ProvideErrorMetadata;
use aws_types::region::Region;
use chrono::{DateTime, Datelike, Timelike, Utc};
use serde::Serialize;
use std::collections::BTreeSet;
use std::env;

pub async fn read_candidate_bundles_from_s3(
    bucket: &str,
    key: &str,
) -> AppResult<Vec<IntelCandidateEvidenceBundle>> {
    let client = s3_client().await?;
    let bytes = get_object_bytes(&client, bucket, key).await?;
    read_candidate_bundles_from_bytes(&format!("s3://{bucket}/{key}"), bytes.as_ref())
}

pub async fn read_research_input_manifest_from_s3(
    bucket: &str,
    key: &str,
) -> AppResult<ResearchInputManifest> {
    let client = s3_client().await?;
    let bytes = get_object_bytes(&client, bucket, key).await?;
    read_research_input_manifest_from_bytes(&format!("s3://{bucket}/{key}"), bytes.as_ref())
}

pub async fn read_research_run_report_from_s3(
    bucket: &str,
    key: &str,
) -> AppResult<ResearchRunReport> {
    let client = s3_client().await?;
    let bytes = get_object_bytes(&client, bucket, key).await?;
    read_research_run_report_from_bytes(&format!("s3://{bucket}/{key}"), bytes.as_ref())
}

pub async fn read_paper_watch_candidates_from_s3(
    bucket: &str,
    key: &str,
) -> AppResult<Vec<PaperWatchCandidate>> {
    let client = s3_client().await?;
    let bytes = get_object_bytes(&client, bucket, key).await?;
    read_paper_watch_candidates_from_bytes(&format!("s3://{bucket}/{key}"), bytes.as_ref())
}

pub async fn read_market_live_ticks_from_s3(
    bucket: &str,
    key: &str,
) -> AppResult<Vec<MarketLiveTick>> {
    let client = s3_client().await?;
    let bytes = get_object_bytes(&client, bucket, key).await?;
    read_market_live_ticks_from_bytes(&format!("s3://{bucket}/{key}"), bytes.as_ref())
}

pub async fn read_paper_watch_live_marks_from_s3(
    bucket: &str,
    keys: &[String],
) -> AppResult<Vec<PaperWatchLiveMark>> {
    let client = s3_client().await?;
    let mut marks = Vec::new();
    for key in keys {
        let bytes = get_object_bytes(&client, bucket, key).await?;
        marks.extend(read_paper_watch_live_marks_from_bytes(
            &format!("s3://{bucket}/{key}"),
            bytes.as_ref(),
        )?);
    }
    Ok(marks)
}

pub async fn read_latest_retest_cycle_source_state_from_s3(
    bucket: &str,
    prefix: &str,
) -> AppResult<RetestCycleSourceState> {
    if bucket.trim().is_empty() {
        return Err(AppError::config(
            "retest cycle source state S3 bucket must not be empty",
        ));
    }
    let client = s3_client().await?;
    let prefix = normalize_prefix(if prefix.trim().is_empty() {
        "retest-cycle-source-state/schema=research_retest_cycle_source_state_v1"
    } else {
        prefix
    });
    if !prefix.starts_with("retest-cycle-source-state/") {
        return Err(AppError::config(
            "retest cycle source state S3 prefix must start with retest-cycle-source-state/",
        ));
    }
    let keys = list_payload_objects_with_prefix(&client, bucket, &prefix, "/state.json", 1_000)
        .await
        .map(|objects| select_latest_payload_keys(objects, 1))?;
    let key = keys
        .first()
        .ok_or_else(|| AppError::AwsNotFound(format!("s3://{bucket}/{prefix}")))?;
    let bytes = get_object_bytes(&client, bucket, key).await?;
    let state = serde_json::from_slice::<RetestCycleSourceState>(&bytes)?;
    if state.schema_version != RETEST_CYCLE_SOURCE_STATE_SCHEMA_VERSION {
        return Err(AppError::validation(format!(
            "retest cycle source state schema_version must be {RETEST_CYCLE_SOURCE_STATE_SCHEMA_VERSION}; got {}",
            state.schema_version
        )));
    }
    Ok(state)
}

pub async fn read_retest_horizon_status_from_s3(
    bucket: &str,
    key: &str,
) -> AppResult<serde_json::Value> {
    let client = s3_client().await?;
    let bytes = get_object_bytes(&client, bucket, key).await?;
    read_retest_horizon_status_from_bytes(&format!("s3://{bucket}/{key}"), bytes.as_ref())
}

pub async fn read_latest_retest_horizon_status_from_s3(
    bucket: &str,
    prefix: &str,
) -> AppResult<serde_json::Value> {
    if bucket.trim().is_empty() {
        return Err(AppError::config(
            "retest horizon status S3 bucket must not be empty",
        ));
    }
    let client = s3_client().await?;
    let prefix = normalize_prefix(if prefix.trim().is_empty() {
        "retest-horizon-status/schema=research_horizon_status_checkpoint_v1"
    } else {
        prefix
    });
    if !prefix.starts_with("retest-horizon-status/") {
        return Err(AppError::config(
            "retest horizon status S3 prefix must start with retest-horizon-status/",
        ));
    }
    let keys = list_payload_objects_with_prefix(
        &client,
        bucket,
        &prefix,
        "/retest-horizon-status.json",
        1_000,
    )
    .await
    .map(|objects| select_latest_payload_keys(objects, 1))?;
    let key = keys
        .first()
        .ok_or_else(|| AppError::AwsNotFound(format!("s3://{bucket}/{prefix}")))?;
    let bytes = get_object_bytes(&client, bucket, key).await?;
    read_retest_horizon_status_from_bytes(&format!("s3://{bucket}/{key}"), bytes.as_ref())
}

pub async fn read_retest_horizon_plan_from_s3(
    bucket: &str,
    key: &str,
) -> AppResult<serde_json::Value> {
    let client = s3_client().await?;
    let bytes = get_object_bytes(&client, bucket, key).await?;
    read_retest_horizon_plan_from_bytes(&format!("s3://{bucket}/{key}"), bytes.as_ref())
}

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

pub async fn read_replay_runs_from_s3(bucket: &str, keys: &[String]) -> AppResult<Vec<ReplayRun>> {
    let client = s3_client().await?;
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
    let client = s3_client().await?;
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

pub async fn discover_replay_run_index_keys_from_s3(
    bucket: &str,
    prefix: &str,
    read_limit: usize,
    scan_limit: usize,
) -> AppResult<Vec<String>> {
    discover_latest_part_jsonl_keys_from_s3(
        bucket,
        prefix,
        read_limit,
        scan_limit,
        "historical replay-run-index",
    )
    .await
}

pub async fn discover_shadow_validation_run_keys_from_s3(
    bucket: &str,
    prefix: &str,
    read_limit: usize,
    scan_limit: usize,
) -> AppResult<Vec<String>> {
    discover_latest_part_jsonl_keys_from_s3(
        bucket,
        prefix,
        read_limit,
        scan_limit,
        "shadow validation run",
    )
    .await
}

pub async fn discover_paper_watch_candidate_keys_from_s3(
    bucket: &str,
    prefix: &str,
    read_limit: usize,
    scan_limit: usize,
) -> AppResult<Vec<String>> {
    discover_latest_part_jsonl_keys_from_s3(
        bucket,
        prefix,
        read_limit,
        scan_limit,
        "paper-watch candidate",
    )
    .await
}

pub async fn discover_paper_watch_live_mark_keys_from_s3(
    bucket: &str,
    prefix: &str,
    read_limit: usize,
    scan_limit: usize,
) -> AppResult<Vec<String>> {
    discover_latest_part_jsonl_keys_from_s3(
        bucket,
        prefix,
        read_limit,
        scan_limit,
        "paper-watch live mark",
    )
    .await
}

pub fn hourly_partitioned_prefix(prefix: &str, timestamp_ms: i64) -> AppResult<String> {
    let dt = partition(timestamp_ms)?;
    Ok(format!(
        "{}dt={}/hour={:02}/",
        normalize_prefix(prefix),
        dt.date,
        dt.hour
    ))
}

async fn discover_latest_part_jsonl_keys_from_s3(
    bucket: &str,
    prefix: &str,
    read_limit: usize,
    scan_limit: usize,
    artifact_label: &str,
) -> AppResult<Vec<String>> {
    if bucket.trim().is_empty() {
        return Err(AppError::config(format!(
            "{artifact_label} S3 bucket must not be empty"
        )));
    }
    if prefix.trim().is_empty() {
        return Err(AppError::config(format!(
            "{artifact_label} S3 prefix must not be empty"
        )));
    }
    if read_limit == 0 {
        return Err(AppError::config(format!(
            "{artifact_label} S3 read limit must be greater than zero"
        )));
    }
    if scan_limit == 0 {
        return Err(AppError::config(format!(
            "{artifact_label} S3 scan limit must be greater than zero"
        )));
    }

    let client = s3_client().await?;
    let objects =
        list_payload_objects_with_prefix(&client, bucket, prefix, "/part-000001.jsonl", scan_limit)
            .await?;
    Ok(select_latest_payload_keys(objects, read_limit))
}

pub async fn read_oss_adapter_runs_from_s3(
    bucket: &str,
    keys: &[String],
) -> AppResult<Vec<OssAdapterRun>> {
    let client = s3_client().await?;
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

pub async fn read_shadow_validation_runs_from_s3(
    bucket: &str,
    keys: &[String],
) -> AppResult<Vec<ShadowValidationRun>> {
    let client = s3_client().await?;
    let mut runs = Vec::new();
    for key in keys {
        let bytes = get_object_bytes(&client, bucket, key).await?;
        runs.extend(read_shadow_validation_runs_from_bytes(
            &format!("s3://{bucket}/{key}"),
            bytes.as_ref(),
        )?);
    }
    Ok(runs)
}

pub async fn write_research_outputs_to_s3(
    bucket: &str,
    prefix: &str,
    artifacts: &ResearchOutputArtifacts<'_>,
) -> AppResult<Vec<String>> {
    if bucket.trim().is_empty() {
        return Err(AppError::config(
            "research output S3 bucket must not be empty",
        ));
    }
    let client = s3_client().await?;
    let mut written = Vec::new();
    let report = artifacts.report;
    let dt = partition(artifacts.output_partition_at_ms)?;
    let prefix = normalize_prefix(prefix);
    let report_key = format!(
        "{prefix}research-run-report/schema={}/dt={}/hour={:02}/research_run_report_id={}/report.json",
        report.schema_version, dt.date, dt.hour, report.research_run_report_id
    );
    put_object_json(&client, bucket, &report_key, report).await?;
    written.push(format!("s3://{bucket}/{report_key}"));

    if !artifacts.replay_runs.is_empty() {
        let replay_key = format!(
            "{prefix}replay-run/schema={}/dt={}/hour={:02}/research_run_report_id={}/part-000001.jsonl",
            artifacts.replay_runs[0].schema_version,
            dt.date,
            dt.hour,
            report.research_run_report_id
        );
        let mut body = Vec::new();
        for run in artifacts.replay_runs {
            serde_json::to_writer(&mut body, run)?;
            body.push(b'\n');
        }
        put_object_bytes(&client, bucket, &replay_key, body, "application/x-ndjson").await?;
        written.push(format!("s3://{bucket}/{replay_key}"));

        let replay_run_uri = format!("s3://{bucket}/{replay_key}");
        let replay_run_index_records = build_replay_run_index_records(
            report,
            artifacts.replay_runs,
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

    if !artifacts.shadow_validation_runs.is_empty() {
        let shadow_key = format!(
            "{prefix}shadow-validation-run/schema={}/dt={}/hour={:02}/research_run_report_id={}/part-000001.jsonl",
            artifacts.shadow_validation_runs[0].schema_version,
            dt.date,
            dt.hour,
            report.research_run_report_id
        );
        let mut body = Vec::new();
        for run in artifacts.shadow_validation_runs {
            serde_json::to_writer(&mut body, run)?;
            body.push(b'\n');
        }
        put_object_bytes(&client, bucket, &shadow_key, body, "application/x-ndjson").await?;
        written.push(format!("s3://{bucket}/{shadow_key}"));
    }

    if !artifacts.paper_watch_candidates.is_empty() {
        let candidate_key = format!(
            "{prefix}paper-watch-candidate/schema={}/dt={}/hour={:02}/research_run_report_id={}/part-000001.jsonl",
            artifacts.paper_watch_candidates[0].schema_version,
            dt.date,
            dt.hour,
            report.research_run_report_id
        );
        put_jsonl_object(
            &client,
            bucket,
            &candidate_key,
            artifacts.paper_watch_candidates,
        )
        .await?;
        written.push(format!("s3://{bucket}/{candidate_key}"));
    }

    if !artifacts.paper_trade_candidates.is_empty() {
        let candidate_key = format!(
            "{prefix}paper-trade-candidate/schema={}/dt={}/hour={:02}/research_run_report_id={}/part-000001.jsonl",
            artifacts.paper_trade_candidates[0].schema_version,
            dt.date,
            dt.hour,
            report.research_run_report_id
        );
        put_jsonl_object(
            &client,
            bucket,
            &candidate_key,
            artifacts.paper_trade_candidates,
        )
        .await?;
        written.push(format!("s3://{bucket}/{candidate_key}"));
    }

    if !artifacts.paper_trade_runs.is_empty() {
        let run_key = format!(
            "{prefix}paper-trade-run/schema={}/dt={}/hour={:02}/research_run_report_id={}/part-000001.jsonl",
            artifacts.paper_trade_runs[0].schema_version,
            dt.date,
            dt.hour,
            report.research_run_report_id
        );
        put_jsonl_object(&client, bucket, &run_key, artifacts.paper_trade_runs).await?;
        written.push(format!("s3://{bucket}/{run_key}"));
    }

    if !artifacts.paper_trade_summaries.is_empty() {
        let summary_key = format!(
            "{prefix}paper-trade-summary/schema={}/dt={}/hour={:02}/research_run_report_id={}/part-000001.jsonl",
            artifacts.paper_trade_summaries[0].schema_version,
            dt.date,
            dt.hour,
            report.research_run_report_id
        );
        put_jsonl_object(
            &client,
            bucket,
            &summary_key,
            artifacts.paper_trade_summaries,
        )
        .await?;
        written.push(format!("s3://{bucket}/{summary_key}"));
    }

    if !artifacts.paper_trade_marks.is_empty() {
        let mark_key = format!(
            "{prefix}paper-trade-mark/schema={}/dt={}/hour={:02}/research_run_report_id={}/part-000001.jsonl",
            artifacts.paper_trade_marks[0].schema_version,
            dt.date,
            dt.hour,
            report.research_run_report_id
        );
        put_jsonl_object(&client, bucket, &mark_key, artifacts.paper_trade_marks).await?;
        written.push(format!("s3://{bucket}/{mark_key}"));
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

pub async fn write_shadow_cycle_decision_to_s3(
    bucket: &str,
    prefix: &str,
    decision: &ShadowCycleDecision,
    output_partition_at_ms: i64,
) -> AppResult<String> {
    if bucket.trim().is_empty() {
        return Err(AppError::config(
            "shadow cycle decision output S3 bucket must not be empty",
        ));
    }
    let client = s3_client().await?;
    let dt = partition(output_partition_at_ms)?;
    let prefix = normalize_prefix(prefix);
    let key = format!(
        "{prefix}shadow-cycle-decision/schema={}/dt={}/hour={:02}/decision_id={}/decision.json",
        decision.schema_version, dt.date, dt.hour, decision.decision_id
    );
    put_object_json(&client, bucket, &key, decision).await?;
    Ok(format!("s3://{bucket}/{key}"))
}

pub async fn write_paper_watch_live_marks_to_s3(
    bucket: &str,
    prefix: &str,
    marks: &[PaperWatchLiveMark],
    output_partition_at_ms: i64,
) -> AppResult<Vec<String>> {
    if marks.is_empty() {
        return Ok(Vec::new());
    }
    if bucket.trim().is_empty() {
        return Err(AppError::config(
            "paper watch live mark output S3 bucket must not be empty",
        ));
    }
    let client = s3_client().await?;
    let dt = partition(output_partition_at_ms)?;
    let prefix = normalize_prefix(if prefix.trim().is_empty() {
        "paper-watch-live-mark/schema=paper_watch_live_mark_v1"
    } else {
        prefix
    });
    if !prefix.starts_with("paper-watch-live-mark/") {
        return Err(AppError::config(
            "paper watch live mark S3 prefix must start with paper-watch-live-mark/",
        ));
    }
    let key = format!(
        "{prefix}dt={}/hour={:02}/run_id={}/part-000001.jsonl",
        dt.date, dt.hour, output_partition_at_ms
    );
    put_jsonl_object(&client, bucket, &key, marks).await?;
    Ok(vec![format!("s3://{bucket}/{key}")])
}

pub async fn write_paper_watch_observer_snapshot_to_s3<T: Serialize>(
    bucket: &str,
    prefix: &str,
    snapshot: &T,
    output_partition_at_ms: i64,
) -> AppResult<String> {
    if bucket.trim().is_empty() {
        return Err(AppError::config(
            "paper-watch observer snapshot output S3 bucket must not be empty",
        ));
    }
    let client = s3_client().await?;
    let dt = partition(output_partition_at_ms)?;
    let prefix = normalize_prefix(if prefix.trim().is_empty() {
        "paper-watch-observer-state/schema=paper_watch_observer_snapshot_v1"
    } else {
        prefix
    });
    if !prefix.starts_with("paper-watch-observer-state/") {
        return Err(AppError::config(
            "paper-watch observer S3 prefix must start with paper-watch-observer-state/",
        ));
    }
    let key = format!(
        "{prefix}dt={}/hour={:02}/run_id={}/state.json",
        dt.date, dt.hour, output_partition_at_ms
    );
    put_object_json(&client, bucket, &key, snapshot).await?;
    Ok(format!("s3://{bucket}/{key}"))
}

pub async fn write_research_input_manifest_to_s3(
    bucket: &str,
    prefix: &str,
    manifest: &ResearchInputManifest,
    output_partition_at_ms: i64,
) -> AppResult<String> {
    if bucket.trim().is_empty() {
        return Err(AppError::config(
            "research input manifest output S3 bucket must not be empty",
        ));
    }
    let packet_id = manifest
        .research_packet_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            AppError::validation(
                "research input manifest requires research_packet_id for S3 output",
            )
        })?;
    let client = s3_client().await?;
    let dt = partition(output_partition_at_ms)?;
    let prefix = normalize_prefix(if prefix.trim().is_empty() {
        "research-input-manifest/schema=research_input_manifest_v1"
    } else {
        prefix
    });
    if !prefix.starts_with("research-input-manifest/") {
        return Err(AppError::config(
            "focused retest manifest S3 prefix must start with research-input-manifest/",
        ));
    }
    let key = format!(
        "{prefix}dt={}/hour={:02}/research_packet_id={packet_id}/manifest.json",
        dt.date, dt.hour
    );
    put_object_json(&client, bucket, &key, manifest).await?;
    Ok(format!("s3://{bucket}/{key}"))
}

pub async fn write_research_input_manifest_to_exact_s3_key_if_absent(
    bucket: &str,
    key: &str,
    manifest: &ResearchInputManifest,
) -> AppResult<Option<String>> {
    validate_s3_location(bucket, key, "research input manifest output")?;
    validate_research_input_manifest_s3_key(key)?;
    let client = s3_client().await?;
    let body = serde_json::to_vec_pretty(manifest)?;
    match put_object_bytes_if_absent(&client, bucket, key, body, "application/json").await? {
        PutIfAbsentResult::Created => Ok(Some(format!("s3://{bucket}/{key}"))),
        PutIfAbsentResult::AlreadyExists => Ok(None),
    }
}

pub async fn write_retest_cycle_source_state_to_s3(
    bucket: &str,
    prefix: &str,
    state: &RetestCycleSourceState,
    output_partition_at_ms: i64,
) -> AppResult<String> {
    if bucket.trim().is_empty() {
        return Err(AppError::config(
            "retest cycle source state output S3 bucket must not be empty",
        ));
    }
    let client = s3_client().await?;
    let dt = partition(output_partition_at_ms)?;
    let prefix = normalize_prefix(if prefix.trim().is_empty() {
        "retest-cycle-source-state/schema=research_retest_cycle_source_state_v1"
    } else {
        prefix
    });
    if !prefix.starts_with("retest-cycle-source-state/") {
        return Err(AppError::config(
            "retest cycle source state S3 prefix must start with retest-cycle-source-state/",
        ));
    }
    let key = format!(
        "{prefix}dt={}/hour={:02}/research_packet_id={}/research_run_report_id={}/state.json",
        dt.date, dt.hour, state.research_packet_id, state.source_research_report_id
    );
    put_object_json(&client, bucket, &key, state).await?;
    Ok(format!("s3://{bucket}/{key}"))
}

pub async fn write_retest_horizon_plan_to_s3(
    bucket: &str,
    prefix: &str,
    plan: &serde_json::Value,
    output_partition_at_ms: i64,
) -> AppResult<String> {
    if bucket.trim().is_empty() {
        return Err(AppError::config(
            "retest horizon plan output S3 bucket must not be empty",
        ));
    }
    let client = s3_client().await?;
    let dt = partition(output_partition_at_ms)?;
    let prefix = normalize_prefix(if prefix.trim().is_empty() {
        "retest-horizon-plan/schema=research_retest_horizon_plan_v1"
    } else {
        prefix
    });
    if !prefix.starts_with("retest-horizon-plan/") {
        return Err(AppError::config(
            "retest horizon plan S3 prefix must start with retest-horizon-plan/",
        ));
    }
    let generated_at_ms = plan
        .get("generated_at_ms")
        .and_then(serde_json::Value::as_i64)
        .unwrap_or(output_partition_at_ms);
    let key = format!(
        "{prefix}dt={}/hour={:02}/generated_at_ms={generated_at_ms}/retest-horizon-plan.json",
        dt.date, dt.hour
    );
    put_object_json(&client, bucket, &key, plan).await?;
    Ok(format!("s3://{bucket}/{key}"))
}

pub async fn write_retest_horizon_status_to_s3(
    bucket: &str,
    prefix: &str,
    status: &serde_json::Value,
    output_partition_at_ms: i64,
) -> AppResult<String> {
    if bucket.trim().is_empty() {
        return Err(AppError::config(
            "retest horizon status output S3 bucket must not be empty",
        ));
    }
    let client = s3_client().await?;
    let dt = partition(output_partition_at_ms)?;
    let prefix = normalize_prefix(if prefix.trim().is_empty() {
        "retest-horizon-status/schema=research_horizon_status_checkpoint_v1"
    } else {
        prefix
    });
    if !prefix.starts_with("retest-horizon-status/") {
        return Err(AppError::config(
            "retest horizon status S3 prefix must start with retest-horizon-status/",
        ));
    }
    let generated_at_ms = status
        .get("generated_at_ms")
        .and_then(serde_json::Value::as_i64)
        .unwrap_or(output_partition_at_ms);
    let key = format!(
        "{prefix}dt={}/hour={:02}/generated_at_ms={generated_at_ms}/retest-horizon-status.json",
        dt.date, dt.hour
    );
    put_object_json(&client, bucket, &key, status).await?;
    Ok(format!("s3://{bucket}/{key}"))
}

async fn discover_latest_market_l1_keys_from_s3(
    bucket: &str,
    window_starts_ms: &[i64],
    family_prefix: &str,
    file_suffix: &str,
    manifest_key_field: &str,
) -> AppResult<Vec<String>> {
    if window_starts_ms.is_empty() {
        return Ok(Vec::new());
    }
    if bucket.trim().is_empty() {
        return Err(AppError::config("market L1 S3 bucket must not be empty"));
    }
    let client = s3_client().await?;
    let mut keys = Vec::new();
    for window_start_ms in window_starts_ms {
        let prefix = format!("{family_prefix}/run_id=l1_{window_start_ms}_");
        if let Some(key) = latest_key_with_prefix(&client, bucket, &prefix, file_suffix).await? {
            keys.push(key);
        }
        if let Some(key) =
            latest_key_from_l1_index(&client, bucket, *window_start_ms, manifest_key_field).await?
        {
            keys.push(key);
        }
    }
    keys.sort();
    keys.dedup();
    Ok(keys)
}

async fn latest_key_from_l1_index(
    client: &Client,
    bucket: &str,
    window_start_ms: i64,
    manifest_key_field: &str,
) -> AppResult<Option<String>> {
    let pointer_key = l1_index_pointer_key(window_start_ms)?;
    let pointer_bytes = match get_object_bytes(client, bucket, &pointer_key).await {
        Ok(bytes) => bytes,
        Err(error) if is_missing_market_artifact(&error) => return Ok(None),
        Err(error) => return Err(error),
    };
    let pointer = serde_json::from_slice::<serde_json::Value>(&pointer_bytes).map_err(|error| {
        AppError::validation(format!(
            "invalid Market-L1 index pointer s3://{bucket}/{pointer_key}: {error}"
        ))
    })?;
    if !is_success_l1_index_pointer(&pointer) {
        return Ok(None);
    }
    let manifest_key = l1_manifest_key_from_pointer(&pointer).ok_or_else(|| {
        AppError::validation(format!(
            "Market-L1 index pointer missing canonical manifest key: s3://{bucket}/{pointer_key}"
        ))
    })?;
    let manifest_bytes = match get_object_bytes(client, bucket, &manifest_key).await {
        Ok(bytes) => bytes,
        Err(error) if is_missing_market_artifact(&error) => return Ok(None),
        Err(error) => return Err(error),
    };
    let manifest =
        serde_json::from_slice::<serde_json::Value>(&manifest_bytes).map_err(|error| {
            AppError::validation(format!(
                "invalid Market-L1 manifest s3://{bucket}/{manifest_key}: {error}"
            ))
        })?;
    if !is_success_l1_manifest(&manifest) {
        return Ok(None);
    }
    Ok(l1_artifact_key_from_manifest(&manifest, manifest_key_field))
}

fn l1_index_pointer_key(window_start_ms: i64) -> AppResult<String> {
    let part = partition(window_start_ms)?;
    Ok(format!(
        "l1_index/window_ms=1000/event_date={}/hour={:02}/window_start_ms={window_start_ms}.json",
        part.date, part.hour
    ))
}

fn l1_manifest_key_from_pointer(pointer: &serde_json::Value) -> Option<String> {
    string_field(pointer, "canonical_manifest_key")
        .or_else(|| string_field(pointer, "manifest_key"))
        .and_then(normalize_s3_key)
}

fn l1_artifact_key_from_manifest(
    manifest: &serde_json::Value,
    manifest_key_field: &str,
) -> Option<String> {
    string_field(manifest, manifest_key_field).and_then(normalize_s3_key)
}

fn is_success_l1_index_pointer(pointer: &serde_json::Value) -> bool {
    pointer
        .get("schema_version")
        .and_then(serde_json::Value::as_str)
        == Some("l1_index_pointer_v1")
        && pointer
            .get("status")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|status| status.eq_ignore_ascii_case("success"))
}

fn is_success_l1_manifest(manifest: &serde_json::Value) -> bool {
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

async fn latest_key_with_prefix(
    client: &Client,
    bucket: &str,
    prefix: &str,
    file_suffix: &str,
) -> AppResult<Option<String>> {
    let mut latest: Option<String> = None;
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
            if !key.ends_with(file_suffix) {
                continue;
            }
            if latest
                .as_deref()
                .is_none_or(|current_latest| key > current_latest)
            {
                latest = Some(key.to_owned());
            }
        }

        continuation_token = output.next_continuation_token().map(ToOwned::to_owned);
        if continuation_token.is_none() {
            break;
        }
    }

    Ok(latest)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ListedPayloadObject {
    key: String,
    last_modified_ms: i64,
}

async fn list_payload_objects_with_prefix(
    client: &Client,
    bucket: &str,
    prefix: &str,
    file_suffix: &str,
    scan_limit: usize,
) -> AppResult<Vec<ListedPayloadObject>> {
    let mut objects = Vec::new();
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
            if !key.ends_with(file_suffix) {
                continue;
            }
            objects.push(ListedPayloadObject {
                key: key.to_owned(),
                last_modified_ms: object
                    .last_modified()
                    .and_then(|last_modified| last_modified.to_millis().ok())
                    .unwrap_or(0),
            });
            if objects.len() > scan_limit {
                return Err(AppError::validation(format!(
                    "historical replay-run-index S3 scan limit exceeded for s3://{bucket}/{prefix}: limit={scan_limit}; narrow the prefix"
                )));
            }
        }

        continuation_token = output.next_continuation_token().map(ToOwned::to_owned);
        if continuation_token.is_none() {
            break;
        }
    }

    Ok(objects)
}

fn select_latest_payload_keys(
    mut objects: Vec<ListedPayloadObject>,
    read_limit: usize,
) -> Vec<String> {
    objects.sort_by(|left, right| {
        right
            .last_modified_ms
            .cmp(&left.last_modified_ms)
            .then_with(|| right.key.cmp(&left.key))
    });
    objects
        .into_iter()
        .take(read_limit)
        .map(|object| object.key)
        .collect()
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
            if let Some(service_error) = error.as_service_error()
                && service_error.code() == Some("NoSuchKey")
            {
                return AppError::AwsNotFound(format!("s3://{bucket}/{key}"));
            }
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

fn is_missing_market_artifact(error: &AppError) -> bool {
    matches!(error, AppError::AwsNotFound(_))
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

fn validate_research_input_manifest_s3_key(key: &str) -> AppResult<()> {
    let trimmed = key.trim().trim_start_matches('/');
    if !trimmed.starts_with("research-input-manifest/") {
        return Err(AppError::config(
            "research input manifest S3 key must start with research-input-manifest/",
        ));
    }
    if !(trimmed.ends_with(".json") || trimmed.ends_with(".jsonl")) {
        return Err(AppError::config(
            "research input manifest S3 key must end with .json or .jsonl",
        ));
    }
    Ok(())
}

async fn s3_client() -> AppResult<Client> {
    let mut loader = aws_config::defaults(BehaviorVersion::latest());
    if let Some(region) = env_string("AWS_REGION").or_else(|| env_string("AWS_DEFAULT_REGION")) {
        loader = loader.region(Region::new(region));
    }
    let config = loader.load().await;
    Ok(Client::new(&config))
}

fn env_string(name: &str) -> Option<String> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

async fn put_object_json<T>(client: &Client, bucket: &str, key: &str, value: &T) -> AppResult<()>
where
    T: serde::Serialize,
{
    let body = serde_json::to_vec_pretty(value)?;
    put_object_bytes(client, bucket, key, body, "application/json").await
}

async fn put_jsonl_object<T>(
    client: &Client,
    bucket: &str,
    key: &str,
    values: &[T],
) -> AppResult<()>
where
    T: serde::Serialize,
{
    let mut body = Vec::new();
    for value in values {
        serde_json::to_writer(&mut body, value)?;
        body.push(b'\n');
    }
    put_object_bytes(client, bucket, key, body, "application/x-ndjson").await
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

enum PutIfAbsentResult {
    Created,
    AlreadyExists,
}

async fn put_object_bytes_if_absent(
    client: &Client,
    bucket: &str,
    key: &str,
    body: Vec<u8>,
    content_type: &str,
) -> AppResult<PutIfAbsentResult> {
    client
        .put_object()
        .bucket(bucket)
        .key(key)
        .content_type(content_type)
        .if_none_match("*")
        .body(body.into())
        .send()
        .await
        .map(|_| PutIfAbsentResult::Created)
        .or_else(|error| {
            if let Some(service_error) = error.as_service_error()
                && matches!(
                    service_error.code(),
                    Some("PreconditionFailed" | "ConditionalRequestConflict")
                )
            {
                return Ok(PutIfAbsentResult::AlreadyExists);
            }
            Err(AppError::Aws(format!(
                "s3 put_object if_absent s3://{bucket}/{key}: {}",
                aws_error_detail(&error)
            )))
        })
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn l1_index_pointer_key_matches_market_ingest_partition_contract() {
        assert_eq!(
            l1_index_pointer_key(1_778_387_400_000).expect("valid timestamp"),
            "l1_index/window_ms=1000/event_date=2026-05-10/hour=04/window_start_ms=1778387400000.json"
        );
    }

    #[test]
    fn extracts_manifest_and_artifact_keys_from_l1_index_contract() {
        let pointer = json!({
            "schema_version": "l1_index_pointer_v1",
            "canonical_manifest_key": "s3://bucket/runs/run_id=l1_1_2_3/manifest.json",
            "status": "success"
        });
        let manifest = json!({
            "schema_version": "l1_manifest_v1",
            "status": "success",
            "market_feature_delta_key": "s3://bucket/market_feature_delta/run_id=l1_1_2_3/delta.json",
            "market_regime_context_key": "market_regime_context/run_id=l1_1_2_3/context.json"
        });

        assert!(is_success_l1_index_pointer(&pointer));
        assert_eq!(
            l1_manifest_key_from_pointer(&pointer),
            Some("runs/run_id=l1_1_2_3/manifest.json".to_owned())
        );
        assert!(is_success_l1_manifest(&manifest));
        assert_eq!(
            l1_artifact_key_from_manifest(&manifest, "market_feature_delta_key"),
            Some("market_feature_delta/run_id=l1_1_2_3/delta.json".to_owned())
        );
        assert_eq!(
            l1_artifact_key_from_manifest(&manifest, "market_regime_context_key"),
            Some("market_regime_context/run_id=l1_1_2_3/context.json".to_owned())
        );
    }

    #[test]
    fn ignores_non_success_l1_index_or_manifest() {
        let pointer = json!({
            "schema_version": "l1_index_pointer_v1",
            "canonical_manifest_key": "runs/run_id=l1_1_2_3/manifest.json",
            "status": "failed"
        });
        let manifest = json!({
            "schema_version": "l1_manifest_v1",
            "status": "failed",
            "market_feature_delta_key": "market_feature_delta/run_id=l1_1_2_3/delta.json"
        });

        assert!(!is_success_l1_index_pointer(&pointer));
        assert!(!is_success_l1_manifest(&manifest));
    }

    #[test]
    fn missing_market_artifact_errors_are_skippable() {
        let error = AppError::AwsNotFound("s3://bucket/missing.json".to_owned());

        assert!(is_missing_market_artifact(&error));
    }

    #[test]
    fn non_not_found_aws_errors_are_not_skippable() {
        let error = AppError::Aws("AccessDenied".to_owned());

        assert!(!is_missing_market_artifact(&error));
    }

    #[test]
    fn latest_payload_key_selection_prefers_recent_jsonl_parts() {
        let keys = select_latest_payload_keys(
            vec![
                ListedPayloadObject {
                    key: "replay-run-index/schema=x/dt=2026-05-22/part-000001.jsonl".to_owned(),
                    last_modified_ms: 100,
                },
                ListedPayloadObject {
                    key: "replay-run-index/schema=x/dt=2026-05-23/part-000001.jsonl".to_owned(),
                    last_modified_ms: 300,
                },
                ListedPayloadObject {
                    key: "replay-run-index/schema=x/dt=2026-05-21/part-000001.jsonl".to_owned(),
                    last_modified_ms: 200,
                },
            ],
            2,
        );

        assert_eq!(
            keys,
            vec![
                "replay-run-index/schema=x/dt=2026-05-23/part-000001.jsonl",
                "replay-run-index/schema=x/dt=2026-05-21/part-000001.jsonl",
            ]
        );
    }

    #[test]
    fn hourly_partitioned_prefix_narrows_observer_restore_scan() {
        assert_eq!(
            hourly_partitioned_prefix(
                "paper-watch-live-mark/schema=paper_watch_live_mark_v1",
                1_779_935_219_010,
            )
            .expect("valid prefix"),
            "paper-watch-live-mark/schema=paper_watch_live_mark_v1/dt=2026-05-28/hour=02/"
        );
    }
}

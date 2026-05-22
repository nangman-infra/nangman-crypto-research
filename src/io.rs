use crate::artifacts::{build_replay_run_index_records, build_research_aggregate_registry_records};
use crate::error::{AppError, AppResult};
use crate::model::{
    IntelCandidateEvidenceBundle, MarketFeatureDelta, MarketRegimeContext, OssAdapterRun,
    PortfolioAllocationSnapshot, PortfolioReduceOnlySignal, PortfolioRiskRejectEvent, ReplayRun,
    ReplayRunIndexRecord, ResearchInputManifest, ResearchRunReport, ShadowValidationRun,
};
use chrono::{DateTime, Datelike, Timelike, Utc};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

pub type PortfolioOutputBodies = (Option<Vec<u8>>, Vec<u8>, Vec<u8>);

pub fn read_candidate_bundles(path: &Path) -> AppResult<Vec<IntelCandidateEvidenceBundle>> {
    read_json_array_or_jsonl(path)
}

pub fn read_research_input_manifest(path: &Path) -> AppResult<ResearchInputManifest> {
    let bytes = fs::read(path)?;
    read_research_input_manifest_from_bytes(&path.display().to_string(), &bytes)
}

pub fn read_research_input_manifest_from_bytes(
    label: &str,
    bytes: &[u8],
) -> AppResult<ResearchInputManifest> {
    let text =
        std::str::from_utf8(bytes).map_err(|error| AppError::Json(format!("{label}: {error}")))?;
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(AppError::validation(format!("{label} must not be empty")));
    }
    serde_json::from_str(trimmed).map_err(Into::into)
}

pub fn read_candidate_bundles_from_bytes(
    label: &str,
    bytes: &[u8],
) -> AppResult<Vec<IntelCandidateEvidenceBundle>> {
    read_json_array_or_jsonl_bytes(label, bytes)
}

pub fn read_market_feature_deltas(path: &Path) -> AppResult<Vec<MarketFeatureDelta>> {
    read_json_array_or_jsonl(path)
}

pub fn read_market_feature_deltas_from_bytes(
    label: &str,
    bytes: &[u8],
) -> AppResult<Vec<MarketFeatureDelta>> {
    read_json_array_or_jsonl_bytes(label, bytes)
}

pub fn read_market_regime_contexts(path: &Path) -> AppResult<Vec<MarketRegimeContext>> {
    read_json_array_or_jsonl(path)
}

pub fn read_replay_runs(path: &Path) -> AppResult<Vec<ReplayRun>> {
    read_json_array_or_jsonl(path)
}

pub fn read_replay_run_index_records(path: &Path) -> AppResult<Vec<ReplayRunIndexRecord>> {
    read_json_array_or_jsonl(path)
}

pub fn read_oss_adapter_runs(path: &Path) -> AppResult<Vec<OssAdapterRun>> {
    read_json_array_or_jsonl(path)
}

pub fn read_market_regime_contexts_from_bytes(
    label: &str,
    bytes: &[u8],
) -> AppResult<Vec<MarketRegimeContext>> {
    read_json_array_or_jsonl_bytes(label, bytes)
}

pub fn read_replay_runs_from_bytes(label: &str, bytes: &[u8]) -> AppResult<Vec<ReplayRun>> {
    read_json_array_or_jsonl_bytes(label, bytes)
}

pub fn read_replay_run_index_records_from_bytes(
    label: &str,
    bytes: &[u8],
) -> AppResult<Vec<ReplayRunIndexRecord>> {
    read_json_array_or_jsonl_bytes(label, bytes)
}

pub fn read_oss_adapter_runs_from_bytes(
    label: &str,
    bytes: &[u8],
) -> AppResult<Vec<OssAdapterRun>> {
    read_json_array_or_jsonl_bytes(label, bytes)
}

pub fn write_research_outputs(
    output_dir: &Path,
    report: &ResearchRunReport,
    replay_runs: &[ReplayRun],
    shadow_validation_runs: &[ShadowValidationRun],
    output_partition_at_ms: i64,
) -> AppResult<Vec<PathBuf>> {
    let mut written = Vec::new();
    let dt = partition(output_partition_at_ms)?;
    let report_key = format!(
        "research-run-report/schema={}/dt={}/hour={:02}/research_run_report_id={}/report.json",
        report.schema_version, dt.date, dt.hour, report.research_run_report_id
    );
    written.push(write_pretty_json(output_dir, &report_key, report)?);

    if !replay_runs.is_empty() {
        let replay_key = format!(
            "replay-run/schema={}/dt={}/hour={:02}/research_run_report_id={}/part-000001.jsonl",
            replay_runs[0].schema_version, dt.date, dt.hour, report.research_run_report_id
        );
        let replay_run_uri = output_dir.join(&replay_key).display().to_string();
        let replay_run_index_records =
            build_replay_run_index_records(report, replay_runs, &replay_run_uri, None, None);
        written.push(write_jsonl(output_dir, &replay_key, replay_runs)?);
        let replay_index_key = format!(
            "replay-run-index/schema={}/dt={}/hour={:02}/research_run_report_id={}/part-000001.jsonl",
            replay_run_index_records[0].schema_version,
            dt.date,
            dt.hour,
            report.research_run_report_id
        );
        written.push(write_jsonl(
            output_dir,
            &replay_index_key,
            &replay_run_index_records,
        )?);
    }

    if !shadow_validation_runs.is_empty() {
        let shadow_key = format!(
            "shadow-validation-run/schema={}/dt={}/hour={:02}/research_run_report_id={}/part-000001.jsonl",
            shadow_validation_runs[0].schema_version,
            dt.date,
            dt.hour,
            report.research_run_report_id
        );
        written.push(write_jsonl(
            output_dir,
            &shadow_key,
            shadow_validation_runs,
        )?);
    }

    if let Some(snapshot) = report.portfolio_allocation_snapshot.as_ref() {
        let snapshot_key = format!(
            "portfolio-allocation-snapshot/schema={}/dt={}/hour={:02}/research_run_report_id={}/snapshot.json",
            snapshot.schema_version, dt.date, dt.hour, report.research_run_report_id
        );
        written.push(write_pretty_json(output_dir, &snapshot_key, snapshot)?);
    }

    if !report.portfolio_risk_reject_events.is_empty() {
        let reject_key = format!(
            "portfolio-risk-reject-event/schema={}/dt={}/hour={:02}/research_run_report_id={}/part-000001.jsonl",
            report.portfolio_risk_reject_events[0].schema_version,
            dt.date,
            dt.hour,
            report.research_run_report_id
        );
        written.push(write_jsonl(
            output_dir,
            &reject_key,
            &report.portfolio_risk_reject_events,
        )?);
    }

    if !report.portfolio_reduce_only_signals.is_empty() {
        let reduce_only_key = format!(
            "portfolio-reduce-only-signal/schema={}/dt={}/hour={:02}/research_run_report_id={}/part-000001.jsonl",
            report.portfolio_reduce_only_signals[0].schema_version,
            dt.date,
            dt.hour,
            report.research_run_report_id
        );
        written.push(write_jsonl(
            output_dir,
            &reduce_only_key,
            &report.portfolio_reduce_only_signals,
        )?);
    }

    let registry_records = build_research_aggregate_registry_records(report);
    if !registry_records.is_empty() {
        let registry_key = format!(
            "research-aggregate-registry/schema={}/dt={}/hour={:02}/research_run_report_id={}/part-000001.jsonl",
            registry_records[0].schema_version, dt.date, dt.hour, report.research_run_report_id
        );
        written.push(write_jsonl(output_dir, &registry_key, &registry_records)?);
    }

    Ok(written)
}

pub fn write_portfolio_outputs_to_body(
    snapshot: &Option<PortfolioAllocationSnapshot>,
    rejects: &[PortfolioRiskRejectEvent],
    reduce_only_signals: &[PortfolioReduceOnlySignal],
) -> AppResult<PortfolioOutputBodies> {
    let snapshot_body = snapshot
        .as_ref()
        .map(serde_json::to_vec_pretty)
        .transpose()?;
    let mut reject_body = Vec::new();
    for record in rejects {
        serde_json::to_writer(&mut reject_body, record)?;
        reject_body.push(b'\n');
    }
    let mut reduce_only_body = Vec::new();
    for record in reduce_only_signals {
        serde_json::to_writer(&mut reduce_only_body, record)?;
        reduce_only_body.push(b'\n');
    }
    Ok((snapshot_body, reject_body, reduce_only_body))
}

fn read_json_array_or_jsonl<T>(path: &Path) -> AppResult<Vec<T>>
where
    T: DeserializeOwned,
{
    let bytes = fs::read(path)?;
    read_json_array_or_jsonl_bytes(&path.display().to_string(), &bytes)
}

fn read_json_array_or_jsonl_bytes<T>(label: &str, bytes: &[u8]) -> AppResult<Vec<T>>
where
    T: DeserializeOwned,
{
    let text =
        std::str::from_utf8(bytes).map_err(|error| AppError::Json(format!("{label}: {error}")))?;
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(AppError::validation(format!("{label} must not be empty")));
    }
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

fn write_pretty_json<T>(output_dir: &Path, key: &str, record: &T) -> AppResult<PathBuf>
where
    T: Serialize,
{
    let path = output_dir.join(key);
    let parent = path.parent().ok_or_else(|| {
        AppError::validation(format!("output path has no parent: {}", path.display()))
    })?;
    fs::create_dir_all(parent)?;
    let mut file = File::create(&path)?;
    serde_json::to_writer_pretty(&mut file, record)?;
    file.write_all(b"\n")?;
    Ok(path)
}

fn write_jsonl<T>(output_dir: &Path, key: &str, records: &[T]) -> AppResult<PathBuf>
where
    T: Serialize,
{
    let path = output_dir.join(key);
    let parent = path.parent().ok_or_else(|| {
        AppError::validation(format!("output path has no parent: {}", path.display()))
    })?;
    fs::create_dir_all(parent)?;
    let mut file = File::create(&path)?;
    for record in records {
        serde_json::to_writer(&mut file, record)?;
        file.write_all(b"\n")?;
    }
    Ok(path)
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

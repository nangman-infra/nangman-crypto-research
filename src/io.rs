use crate::artifacts::{build_replay_run_index_records, build_research_aggregate_registry_records};
use crate::error::{AppError, AppResult};
use crate::model::{
    IntelCandidateEvidenceBundle, MarketFeatureDelta, MarketRegimeContext, OssAdapterRun,
    PaperTradeCandidate, PaperTradeMark, PaperTradeRun, PaperTradeSummary, PaperWatchCandidate,
    PortfolioAllocationSnapshot, PortfolioReduceOnlySignal, PortfolioRiskRejectEvent, ReplayRun,
    ReplayRunIndexRecord, ResearchInputManifest, ResearchRunReport, ShadowCycleDecision,
    ShadowValidationRun,
};
use chrono::{DateTime, Datelike, Timelike, Utc};
use serde::de::{DeserializeOwned, DeserializeSeed, SeqAccess, Visitor};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::Write;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

pub type PortfolioOutputBodies = (Option<Vec<u8>>, Vec<u8>, Vec<u8>);

pub struct ResearchOutputArtifacts<'a> {
    pub report: &'a ResearchRunReport,
    pub replay_runs: &'a [ReplayRun],
    pub shadow_validation_runs: &'a [ShadowValidationRun],
    pub paper_watch_candidates: &'a [PaperWatchCandidate],
    pub paper_trade_candidates: &'a [PaperTradeCandidate],
    pub paper_trade_runs: &'a [PaperTradeRun],
    pub paper_trade_summaries: &'a [PaperTradeSummary],
    pub paper_trade_marks: &'a [PaperTradeMark],
    pub output_partition_at_ms: i64,
}

pub fn read_candidate_bundles(path: &Path) -> AppResult<Vec<IntelCandidateEvidenceBundle>> {
    read_json_array_or_jsonl(path)
}

pub fn read_research_input_manifest(path: &Path) -> AppResult<ResearchInputManifest> {
    let bytes = fs::read(path)?;
    read_research_input_manifest_from_bytes(&path.display().to_string(), &bytes)
}

pub fn read_research_run_report(path: &Path) -> AppResult<ResearchRunReport> {
    let bytes = fs::read(path)?;
    read_research_run_report_from_bytes(&path.display().to_string(), &bytes)
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

pub fn read_research_run_report_from_bytes(
    label: &str,
    bytes: &[u8],
) -> AppResult<ResearchRunReport> {
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

pub fn read_market_feature_deltas_matching_symbols_from_bytes(
    label: &str,
    bytes: &[u8],
    symbols: &BTreeSet<String>,
) -> AppResult<Vec<MarketFeatureDelta>> {
    if symbols.is_empty() {
        return read_market_feature_deltas_from_bytes(label, bytes);
    }
    read_json_array_or_jsonl_bytes_filter(label, bytes, |delta: &MarketFeatureDelta| {
        symbols.contains(delta.symbol_canonical.as_str())
    })
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

pub fn read_shadow_validation_runs(path: &Path) -> AppResult<Vec<ShadowValidationRun>> {
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

pub fn read_shadow_validation_runs_from_bytes(
    label: &str,
    bytes: &[u8],
) -> AppResult<Vec<ShadowValidationRun>> {
    read_json_array_or_jsonl_bytes(label, bytes)
}

pub fn write_research_outputs(
    output_dir: &Path,
    artifacts: &ResearchOutputArtifacts<'_>,
) -> AppResult<Vec<PathBuf>> {
    let mut written = Vec::new();
    let report = artifacts.report;
    let dt = partition(artifacts.output_partition_at_ms)?;
    let report_key = format!(
        "research-run-report/schema={}/dt={}/hour={:02}/research_run_report_id={}/report.json",
        report.schema_version, dt.date, dt.hour, report.research_run_report_id
    );
    written.push(write_pretty_json(output_dir, &report_key, report)?);

    if !artifacts.replay_runs.is_empty() {
        let replay_key = format!(
            "replay-run/schema={}/dt={}/hour={:02}/research_run_report_id={}/part-000001.jsonl",
            artifacts.replay_runs[0].schema_version,
            dt.date,
            dt.hour,
            report.research_run_report_id
        );
        let replay_run_uri = output_dir.join(&replay_key).display().to_string();
        let replay_run_index_records = build_replay_run_index_records(
            report,
            artifacts.replay_runs,
            &replay_run_uri,
            None,
            None,
        );
        written.push(write_jsonl(output_dir, &replay_key, artifacts.replay_runs)?);
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

    if !artifacts.shadow_validation_runs.is_empty() {
        let shadow_key = format!(
            "shadow-validation-run/schema={}/dt={}/hour={:02}/research_run_report_id={}/part-000001.jsonl",
            artifacts.shadow_validation_runs[0].schema_version,
            dt.date,
            dt.hour,
            report.research_run_report_id
        );
        written.push(write_jsonl(
            output_dir,
            &shadow_key,
            artifacts.shadow_validation_runs,
        )?);
    }

    if !artifacts.paper_watch_candidates.is_empty() {
        let candidate_key = format!(
            "paper-watch-candidate/schema={}/dt={}/hour={:02}/research_run_report_id={}/part-000001.jsonl",
            artifacts.paper_watch_candidates[0].schema_version,
            dt.date,
            dt.hour,
            report.research_run_report_id
        );
        written.push(write_jsonl(
            output_dir,
            &candidate_key,
            artifacts.paper_watch_candidates,
        )?);
    }

    if !artifacts.paper_trade_candidates.is_empty() {
        let candidate_key = format!(
            "paper-trade-candidate/schema={}/dt={}/hour={:02}/research_run_report_id={}/part-000001.jsonl",
            artifacts.paper_trade_candidates[0].schema_version,
            dt.date,
            dt.hour,
            report.research_run_report_id
        );
        written.push(write_jsonl(
            output_dir,
            &candidate_key,
            artifacts.paper_trade_candidates,
        )?);
    }

    if !artifacts.paper_trade_runs.is_empty() {
        let run_key = format!(
            "paper-trade-run/schema={}/dt={}/hour={:02}/research_run_report_id={}/part-000001.jsonl",
            artifacts.paper_trade_runs[0].schema_version,
            dt.date,
            dt.hour,
            report.research_run_report_id
        );
        written.push(write_jsonl(
            output_dir,
            &run_key,
            artifacts.paper_trade_runs,
        )?);
    }

    if !artifacts.paper_trade_summaries.is_empty() {
        let summary_key = format!(
            "paper-trade-summary/schema={}/dt={}/hour={:02}/research_run_report_id={}/part-000001.jsonl",
            artifacts.paper_trade_summaries[0].schema_version,
            dt.date,
            dt.hour,
            report.research_run_report_id
        );
        written.push(write_jsonl(
            output_dir,
            &summary_key,
            artifacts.paper_trade_summaries,
        )?);
    }

    if !artifacts.paper_trade_marks.is_empty() {
        let mark_key = format!(
            "paper-trade-mark/schema={}/dt={}/hour={:02}/research_run_report_id={}/part-000001.jsonl",
            artifacts.paper_trade_marks[0].schema_version,
            dt.date,
            dt.hour,
            report.research_run_report_id
        );
        written.push(write_jsonl(
            output_dir,
            &mark_key,
            artifacts.paper_trade_marks,
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

pub fn write_shadow_cycle_decision(
    output_file: &Path,
    decision: &ShadowCycleDecision,
) -> AppResult<PathBuf> {
    if !output_file.is_absolute() {
        return Err(AppError::config(
            "shadow cycle decision output file must be an absolute path",
        ));
    }
    let parent = output_file.parent().ok_or_else(|| {
        AppError::validation(format!(
            "shadow cycle decision output path has no parent: {}",
            output_file.display()
        ))
    })?;
    fs::create_dir_all(parent)?;
    let mut file = File::create(output_file)?;
    serde_json::to_writer_pretty(&mut file, decision)?;
    file.write_all(b"\n")?;
    Ok(output_file.to_path_buf())
}

pub fn write_research_input_manifest(
    output_file: &Path,
    manifest: &ResearchInputManifest,
) -> AppResult<PathBuf> {
    if !output_file.is_absolute() {
        return Err(AppError::config(
            "research input manifest output file must be an absolute path",
        ));
    }
    let parent = output_file.parent().ok_or_else(|| {
        AppError::validation(format!(
            "research input manifest output path has no parent: {}",
            output_file.display()
        ))
    })?;
    fs::create_dir_all(parent)?;
    let mut file = File::create(output_file)?;
    serde_json::to_writer_pretty(&mut file, manifest)?;
    file.write_all(b"\n")?;
    Ok(output_file.to_path_buf())
}

pub fn write_pretty_json_file<T>(output_file: &Path, value: &T) -> AppResult<PathBuf>
where
    T: Serialize,
{
    if !output_file.is_absolute() {
        return Err(AppError::config("output file must be an absolute path"));
    }
    let parent = output_file.parent().ok_or_else(|| {
        AppError::validation(format!(
            "output file path has no parent: {}",
            output_file.display()
        ))
    })?;
    fs::create_dir_all(parent)?;
    let mut file = File::create(output_file)?;
    serde_json::to_writer_pretty(&mut file, value)?;
    file.write_all(b"\n")?;
    Ok(output_file.to_path_buf())
}

pub fn write_shadow_cycle_decision_to_dir(
    output_dir: &Path,
    decision: &ShadowCycleDecision,
    output_partition_at_ms: i64,
) -> AppResult<PathBuf> {
    let dt = partition(output_partition_at_ms)?;
    let key = format!(
        "shadow-cycle-decision/schema={}/dt={}/hour={:02}/decision_id={}/decision.json",
        decision.schema_version, dt.date, dt.hour, decision.decision_id
    );
    write_pretty_json(output_dir, &key, decision)
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

fn read_json_array_or_jsonl_bytes_filter<T, F>(
    label: &str,
    bytes: &[u8],
    mut keep: F,
) -> AppResult<Vec<T>>
where
    T: DeserializeOwned,
    F: FnMut(&T) -> bool,
{
    let text =
        std::str::from_utf8(bytes).map_err(|error| AppError::Json(format!("{label}: {error}")))?;
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(AppError::validation(format!("{label} must not be empty")));
    }
    if trimmed.starts_with('[') {
        let mut deserializer = serde_json::Deserializer::from_str(trimmed);
        let values = FilteredSeqSeed::<T, F> {
            keep: &mut keep,
            marker: PhantomData,
        }
        .deserialize(&mut deserializer)?;
        deserializer.end()?;
        return Ok(values);
    }
    if trimmed.starts_with('{')
        && let Ok(value) = serde_json::from_str(trimmed)
    {
        return Ok(if keep(&value) {
            vec![value]
        } else {
            Vec::new()
        });
    }

    let mut values = Vec::new();
    for (index, line) in trimmed.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let value = serde_json::from_str(line).map_err(|error| {
            AppError::Json(format!(
                "{label} line {} is not valid JSON: {error}",
                index + 1
            ))
        })?;
        if keep(&value) {
            values.push(value);
        }
    }
    Ok(values)
}

struct FilteredSeqSeed<'a, T, F> {
    keep: &'a mut F,
    marker: PhantomData<T>,
}

impl<'de, T, F> DeserializeSeed<'de> for FilteredSeqSeed<'_, T, F>
where
    T: Deserialize<'de>,
    F: FnMut(&T) -> bool,
{
    type Value = Vec<T>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(FilteredSeqVisitor {
            keep: self.keep,
            marker: PhantomData,
        })
    }
}

struct FilteredSeqVisitor<'a, T, F> {
    keep: &'a mut F,
    marker: PhantomData<T>,
}

impl<'de, T, F> Visitor<'de> for FilteredSeqVisitor<'_, T, F>
where
    T: Deserialize<'de>,
    F: FnMut(&T) -> bool,
{
    type Value = Vec<T>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a JSON array")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(value) = seq.next_element::<T>()? {
            if (self.keep)(&value) {
                values.push(value);
            }
        }
        Ok(values)
    }
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

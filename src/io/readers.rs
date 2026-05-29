use crate::error::{AppError, AppResult};
use crate::model::{
    IntelCandidateEvidenceBundle, MarketFeatureDelta, MarketLiveTick, MarketRegimeContext,
    OssAdapterRun, PaperWatchCandidate, PaperWatchLiveMark, ReplayRun, ReplayRunIndexRecord,
    ResearchInputManifest, ResearchRunReport, ShadowValidationRun,
};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use super::json::{
    read_json_array_or_jsonl, read_json_array_or_jsonl_bytes, read_json_array_or_jsonl_bytes_filter,
};

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

pub fn read_paper_watch_candidates_from_bytes(
    label: &str,
    bytes: &[u8],
) -> AppResult<Vec<PaperWatchCandidate>> {
    read_json_array_or_jsonl_bytes(label, bytes)
}

pub fn read_market_live_ticks_from_bytes(
    label: &str,
    bytes: &[u8],
) -> AppResult<Vec<MarketLiveTick>> {
    read_json_array_or_jsonl_bytes(label, bytes)
}

pub fn read_paper_watch_live_marks_from_bytes(
    label: &str,
    bytes: &[u8],
) -> AppResult<Vec<PaperWatchLiveMark>> {
    read_json_array_or_jsonl_bytes(label, bytes)
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

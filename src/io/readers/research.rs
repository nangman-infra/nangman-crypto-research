use crate::error::{AppError, AppResult};
use crate::model::{IntelCandidateEvidenceBundle, ResearchInputManifest, ResearchRunReport};
use serde::de::DeserializeOwned;
use std::fs;
use std::path::Path;

use super::super::json::{read_json_array_or_jsonl, read_json_array_or_jsonl_bytes};

pub fn read_candidate_bundles(path: &Path) -> AppResult<Vec<IntelCandidateEvidenceBundle>> {
    read_json_array_or_jsonl(path)
}

pub fn read_candidate_bundles_from_bytes(
    label: &str,
    bytes: &[u8],
) -> AppResult<Vec<IntelCandidateEvidenceBundle>> {
    read_json_array_or_jsonl_bytes(label, bytes)
}

pub fn read_research_input_manifest(path: &Path) -> AppResult<ResearchInputManifest> {
    let bytes = fs::read(path)?;
    read_research_input_manifest_from_bytes(&path.display().to_string(), &bytes)
}

pub fn read_research_input_manifest_from_bytes(
    label: &str,
    bytes: &[u8],
) -> AppResult<ResearchInputManifest> {
    read_single_json_object_from_bytes(label, bytes)
}

pub fn read_research_run_report(path: &Path) -> AppResult<ResearchRunReport> {
    let bytes = fs::read(path)?;
    read_research_run_report_from_bytes(&path.display().to_string(), &bytes)
}

pub fn read_research_run_report_from_bytes(
    label: &str,
    bytes: &[u8],
) -> AppResult<ResearchRunReport> {
    read_single_json_object_from_bytes(label, bytes)
}

fn read_single_json_object_from_bytes<T>(label: &str, bytes: &[u8]) -> AppResult<T>
where
    T: DeserializeOwned,
{
    let text =
        std::str::from_utf8(bytes).map_err(|error| AppError::Json(format!("{label}: {error}")))?;
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(AppError::validation(format!("{label} must not be empty")));
    }
    serde_json::from_str(trimmed).map_err(Into::into)
}

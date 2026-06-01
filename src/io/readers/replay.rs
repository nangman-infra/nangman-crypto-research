use crate::error::AppResult;
use crate::model::{OssAdapterRun, ReplayRun, ReplayRunIndexRecord, ShadowValidationRun};
use std::path::Path;

use super::super::json::{read_json_array_or_jsonl, read_json_array_or_jsonl_bytes};

pub fn read_replay_runs(path: &Path) -> AppResult<Vec<ReplayRun>> {
    read_json_array_or_jsonl(path)
}

pub fn read_replay_runs_from_bytes(label: &str, bytes: &[u8]) -> AppResult<Vec<ReplayRun>> {
    read_json_array_or_jsonl_bytes(label, bytes)
}

pub fn read_replay_run_index_records(path: &Path) -> AppResult<Vec<ReplayRunIndexRecord>> {
    read_json_array_or_jsonl(path)
}

pub fn read_replay_run_index_records_from_bytes(
    label: &str,
    bytes: &[u8],
) -> AppResult<Vec<ReplayRunIndexRecord>> {
    read_json_array_or_jsonl_bytes(label, bytes)
}

pub fn read_oss_adapter_runs(path: &Path) -> AppResult<Vec<OssAdapterRun>> {
    read_json_array_or_jsonl(path)
}

pub fn read_oss_adapter_runs_from_bytes(
    label: &str,
    bytes: &[u8],
) -> AppResult<Vec<OssAdapterRun>> {
    read_json_array_or_jsonl_bytes(label, bytes)
}

pub fn read_shadow_validation_runs(path: &Path) -> AppResult<Vec<ShadowValidationRun>> {
    read_json_array_or_jsonl(path)
}

pub fn read_shadow_validation_runs_from_bytes(
    label: &str,
    bytes: &[u8],
) -> AppResult<Vec<ShadowValidationRun>> {
    read_json_array_or_jsonl_bytes(label, bytes)
}

use super::plain::read_json_array_or_jsonl_bytes;
use crate::error::AppResult;
use serde::de::DeserializeOwned;
use std::fs;
use std::path::Path;

pub(in crate::io) fn read_json_array_or_jsonl<T>(path: &Path) -> AppResult<Vec<T>>
where
    T: DeserializeOwned,
{
    let bytes = fs::read(path)?;
    read_json_array_or_jsonl_bytes(&path.display().to_string(), &bytes)
}

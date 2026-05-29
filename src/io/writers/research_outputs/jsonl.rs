use crate::error::AppResult;
use std::path::{Path, PathBuf};

use super::super::file::write_jsonl;
use super::keys::ResearchOutputKeys;

pub(super) fn write_jsonl_dataset<T>(
    output_dir: &Path,
    keys: &ResearchOutputKeys,
    dataset: &str,
    schema_version: &str,
    records: &[T],
    written: &mut Vec<PathBuf>,
) -> AppResult<()>
where
    T: serde::Serialize,
{
    let key = keys.jsonl_dataset(dataset, schema_version);
    written.push(write_jsonl(output_dir, &key, records)?);
    Ok(())
}

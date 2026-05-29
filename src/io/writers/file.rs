use crate::error::{AppError, AppResult};
use serde::Serialize;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use super::validation::output_path;

pub(super) fn write_pretty_json<T>(output_dir: &Path, key: &str, record: &T) -> AppResult<PathBuf>
where
    T: Serialize,
{
    let path = output_path(output_dir, key)?;
    let parent = output_parent(&path)?;
    fs::create_dir_all(parent)?;
    let mut file = File::create(&path)?;
    serde_json::to_writer_pretty(&mut file, record)?;
    file.write_all(b"\n")?;
    Ok(path)
}

pub(super) fn write_jsonl<T>(output_dir: &Path, key: &str, records: &[T]) -> AppResult<PathBuf>
where
    T: Serialize,
{
    let path = output_path(output_dir, key)?;
    let parent = output_parent(&path)?;
    fs::create_dir_all(parent)?;
    let mut file = File::create(&path)?;
    for record in records {
        serde_json::to_writer(&mut file, record)?;
        file.write_all(b"\n")?;
    }
    Ok(path)
}

fn output_parent(path: &Path) -> AppResult<&Path> {
    path.parent().ok_or_else(|| {
        AppError::validation(format!("output path has no parent: {}", path.display()))
    })
}

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
    let mut file = create_output_file(&path)?;
    serde_json::to_writer_pretty(&mut file, record)?;
    file.write_all(b"\n")?;
    Ok(path)
}

pub(super) fn write_jsonl<T>(output_dir: &Path, key: &str, records: &[T]) -> AppResult<PathBuf>
where
    T: Serialize,
{
    let path = output_path(output_dir, key)?;
    let mut file = create_output_file(&path)?;
    for record in records {
        serde_json::to_writer(&mut file, record)?;
        file.write_all(b"\n")?;
    }
    Ok(path)
}

pub(super) fn create_output_file(path: &Path) -> AppResult<File> {
    let parent = output_parent(path)?;
    create_output_parent_dirs(parent)?;
    if fs::symlink_metadata(path).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
        return Err(AppError::validation(format!(
            "output path must not be a symlink: {}",
            path.display()
        )));
    }
    Ok(File::create(path)?)
}

fn create_output_parent_dirs(parent: &Path) -> AppResult<()> {
    match fs::symlink_metadata(parent) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            return Err(AppError::validation(format!(
                "output parent directory must not be a symlink: {}",
                parent.display()
            )));
        }
        Ok(metadata) if metadata.is_dir() => return Ok(()),
        Ok(_) => {
            return Err(AppError::validation(format!(
                "output parent path must be a directory: {}",
                parent.display()
            )));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }

    let ancestor = output_parent(parent)?;
    create_output_parent_dirs(ancestor)?;
    match fs::create_dir(parent) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(error) => return Err(error.into()),
    }
    ensure_output_directory(parent)
}

fn ensure_output_directory(path: &Path) -> AppResult<()> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        return Err(AppError::validation(format!(
            "output parent directory must not be a symlink: {}",
            path.display()
        )));
    }
    if !metadata.is_dir() {
        return Err(AppError::validation(format!(
            "output parent path must be a directory: {}",
            path.display()
        )));
    }
    Ok(())
}

fn output_parent(path: &Path) -> AppResult<&Path> {
    path.parent().ok_or_else(|| {
        AppError::validation(format!("output path has no parent: {}", path.display()))
    })
}

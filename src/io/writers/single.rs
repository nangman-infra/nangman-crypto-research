use crate::error::{AppError, AppResult};
use crate::model::{PaperWatchLiveMark, ResearchInputManifest, ShadowCycleDecision};
use serde::Serialize;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use super::super::partition::partition;
use super::file::{write_jsonl, write_pretty_json};

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

pub fn write_paper_watch_live_marks(
    output_dir: &Path,
    marks: &[PaperWatchLiveMark],
    output_partition_at_ms: i64,
) -> AppResult<Vec<PathBuf>> {
    if marks.is_empty() {
        return Ok(Vec::new());
    }
    let dt = partition(output_partition_at_ms)?;
    let key = format!(
        "paper-watch-live-mark/schema={}/dt={}/hour={:02}/run_id={}/part-000001.jsonl",
        marks[0].schema_version, dt.date, dt.hour, output_partition_at_ms
    );
    Ok(vec![write_jsonl(output_dir, &key, marks)?])
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

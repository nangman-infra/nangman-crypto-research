use crate::error::AppResult;
use crate::model::{PaperWatchLiveMark, ResearchInputManifest, ShadowCycleDecision};
use crate::path_validation::validate_config_absolute_path;
use serde::Serialize;
use std::io::Write;
use std::path::{Path, PathBuf};

use super::super::partition::partition;
use super::file::{create_output_file, write_jsonl, write_pretty_json};

pub fn write_shadow_cycle_decision(
    output_file: &Path,
    decision: &ShadowCycleDecision,
) -> AppResult<PathBuf> {
    write_pretty_json_file_with_labels(output_file, decision, "shadow cycle decision output file")
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
    write_pretty_json_file_with_labels(output_file, manifest, "research input manifest output file")
}

pub fn write_pretty_json_file<T>(output_file: &Path, value: &T) -> AppResult<PathBuf>
where
    T: Serialize,
{
    write_pretty_json_file_with_labels(output_file, value, "output file")
}

fn write_pretty_json_file_with_labels<T>(
    output_file: &Path,
    value: &T,
    file_label: &str,
) -> AppResult<PathBuf>
where
    T: Serialize,
{
    validate_config_absolute_path(output_file, file_label)?;
    let mut file = create_output_file(output_file)?;
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

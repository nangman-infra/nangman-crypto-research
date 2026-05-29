use crate::artifacts::build_replay_run_index_records;
use crate::error::AppResult;
use crate::io::ResearchOutputArtifacts;
use std::path::{Path, PathBuf};

use super::jsonl::write_jsonl_dataset;
use super::keys::ResearchOutputKeys;
use crate::io::writers::validation::output_path;

pub(super) fn write_replay_outputs(
    output_dir: &Path,
    keys: &ResearchOutputKeys,
    artifacts: &ResearchOutputArtifacts<'_>,
    written: &mut Vec<PathBuf>,
) -> AppResult<()> {
    if artifacts.replay_runs.is_empty() {
        return Ok(());
    }

    let replay_key = keys.jsonl_dataset("replay-run", &artifacts.replay_runs[0].schema_version);
    let replay_run_uri = output_path(output_dir, &replay_key)?.display().to_string();
    let replay_run_index_records = build_replay_run_index_records(
        artifacts.report,
        artifacts.replay_runs,
        &replay_run_uri,
        None,
        None,
    );

    write_jsonl_dataset(
        output_dir,
        keys,
        "replay-run",
        &artifacts.replay_runs[0].schema_version,
        artifacts.replay_runs,
        written,
    )?;
    write_jsonl_dataset(
        output_dir,
        keys,
        "replay-run-index",
        &replay_run_index_records[0].schema_version,
        &replay_run_index_records,
        written,
    )
}

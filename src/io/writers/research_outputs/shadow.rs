use crate::error::AppResult;
use crate::io::ResearchOutputArtifacts;
use std::path::{Path, PathBuf};

use super::jsonl::write_jsonl_dataset;
use super::keys::ResearchOutputKeys;

pub(super) fn write_shadow_outputs(
    output_dir: &Path,
    keys: &ResearchOutputKeys,
    artifacts: &ResearchOutputArtifacts<'_>,
    written: &mut Vec<PathBuf>,
) -> AppResult<()> {
    if artifacts.shadow_validation_runs.is_empty() {
        return Ok(());
    }

    write_jsonl_dataset(
        output_dir,
        keys,
        "shadow-validation-run",
        &artifacts.shadow_validation_runs[0].schema_version,
        artifacts.shadow_validation_runs,
        written,
    )
}

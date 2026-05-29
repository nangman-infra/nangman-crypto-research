use crate::error::AppResult;
use std::path::{Path, PathBuf};

mod jsonl;
mod keys;
mod paper;
mod portfolio;
mod registry;
mod replay;
mod report;
mod shadow;

use super::super::types::ResearchOutputArtifacts;
use keys::ResearchOutputKeys;
use paper::write_paper_outputs;
use portfolio::write_portfolio_outputs;
use registry::write_registry_outputs;
use replay::write_replay_outputs;
use report::write_report_output;
use shadow::write_shadow_outputs;

pub fn write_research_outputs(
    output_dir: &Path,
    artifacts: &ResearchOutputArtifacts<'_>,
) -> AppResult<Vec<PathBuf>> {
    let mut written = Vec::new();
    let report = artifacts.report;
    let keys = ResearchOutputKeys::new(
        artifacts.output_partition_at_ms,
        &report.research_run_report_id,
    )?;
    write_report_output(output_dir, &keys, report, &mut written)?;
    write_replay_outputs(output_dir, &keys, artifacts, &mut written)?;
    write_shadow_outputs(output_dir, &keys, artifacts, &mut written)?;
    write_paper_outputs(output_dir, &keys, artifacts, &mut written)?;
    write_portfolio_outputs(output_dir, &keys, report, &mut written)?;
    write_registry_outputs(output_dir, &keys, report, &mut written)?;

    Ok(written)
}

use crate::error::AppResult;
use crate::io::ResearchOutputArtifacts;
use std::path::{Path, PathBuf};

use super::jsonl::write_jsonl_dataset;
use super::keys::ResearchOutputKeys;

pub(super) fn write_paper_outputs(
    output_dir: &Path,
    keys: &ResearchOutputKeys,
    artifacts: &ResearchOutputArtifacts<'_>,
    written: &mut Vec<PathBuf>,
) -> AppResult<()> {
    if !artifacts.paper_watch_candidates.is_empty() {
        write_jsonl_dataset(
            output_dir,
            keys,
            "paper-watch-candidate",
            &artifacts.paper_watch_candidates[0].schema_version,
            artifacts.paper_watch_candidates,
            written,
        )?;
    }

    if !artifacts.paper_trade_candidates.is_empty() {
        write_jsonl_dataset(
            output_dir,
            keys,
            "paper-trade-candidate",
            &artifacts.paper_trade_candidates[0].schema_version,
            artifacts.paper_trade_candidates,
            written,
        )?;
    }

    if !artifacts.paper_trade_runs.is_empty() {
        write_jsonl_dataset(
            output_dir,
            keys,
            "paper-trade-run",
            &artifacts.paper_trade_runs[0].schema_version,
            artifacts.paper_trade_runs,
            written,
        )?;
    }

    if !artifacts.paper_trade_summaries.is_empty() {
        write_jsonl_dataset(
            output_dir,
            keys,
            "paper-trade-summary",
            &artifacts.paper_trade_summaries[0].schema_version,
            artifacts.paper_trade_summaries,
            written,
        )?;
    }

    if !artifacts.paper_trade_marks.is_empty() {
        write_jsonl_dataset(
            output_dir,
            keys,
            "paper-trade-mark",
            &artifacts.paper_trade_marks[0].schema_version,
            artifacts.paper_trade_marks,
            written,
        )?;
    }

    Ok(())
}

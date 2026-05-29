use crate::error::AppResult;
use crate::model::ResearchRunReport;
use std::path::{Path, PathBuf};

use super::super::file::write_pretty_json;
use super::jsonl::write_jsonl_dataset;
use super::keys::ResearchOutputKeys;

pub(super) fn write_portfolio_outputs(
    output_dir: &Path,
    keys: &ResearchOutputKeys,
    report: &ResearchRunReport,
    written: &mut Vec<PathBuf>,
) -> AppResult<()> {
    if let Some(snapshot) = report.portfolio_allocation_snapshot.as_ref() {
        let snapshot_key = keys.json_object(
            "portfolio-allocation-snapshot",
            &snapshot.schema_version,
            "snapshot.json",
        );
        written.push(write_pretty_json(output_dir, &snapshot_key, snapshot)?);
    }

    if !report.portfolio_risk_reject_events.is_empty() {
        write_jsonl_dataset(
            output_dir,
            keys,
            "portfolio-risk-reject-event",
            &report.portfolio_risk_reject_events[0].schema_version,
            &report.portfolio_risk_reject_events,
            written,
        )?;
    }

    if !report.portfolio_reduce_only_signals.is_empty() {
        write_jsonl_dataset(
            output_dir,
            keys,
            "portfolio-reduce-only-signal",
            &report.portfolio_reduce_only_signals[0].schema_version,
            &report.portfolio_reduce_only_signals,
            written,
        )?;
    }

    Ok(())
}

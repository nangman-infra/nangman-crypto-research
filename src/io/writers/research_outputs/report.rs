use crate::error::AppResult;
use crate::model::ResearchRunReport;
use std::path::{Path, PathBuf};

use super::super::file::write_pretty_json;
use super::keys::ResearchOutputKeys;

pub(super) fn write_report_output(
    output_dir: &Path,
    keys: &ResearchOutputKeys,
    report: &ResearchRunReport,
    written: &mut Vec<PathBuf>,
) -> AppResult<()> {
    let report_key = keys.json_object("research-run-report", &report.schema_version, "report.json");
    written.push(write_pretty_json(output_dir, &report_key, report)?);
    Ok(())
}

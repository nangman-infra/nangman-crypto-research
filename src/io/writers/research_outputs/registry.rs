use crate::artifacts::build_research_aggregate_registry_records;
use crate::error::AppResult;
use crate::model::ResearchRunReport;
use std::path::{Path, PathBuf};

use super::jsonl::write_jsonl_dataset;
use super::keys::ResearchOutputKeys;

pub(super) fn write_registry_outputs(
    output_dir: &Path,
    keys: &ResearchOutputKeys,
    report: &ResearchRunReport,
    written: &mut Vec<PathBuf>,
) -> AppResult<()> {
    let registry_records = build_research_aggregate_registry_records(report);
    if registry_records.is_empty() {
        return Ok(());
    }

    write_jsonl_dataset(
        output_dir,
        keys,
        "research-aggregate-registry",
        &registry_records[0].schema_version,
        &registry_records,
        written,
    )
}

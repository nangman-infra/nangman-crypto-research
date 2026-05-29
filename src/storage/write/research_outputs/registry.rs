use aws_sdk_s3::Client;

use crate::artifacts::build_research_aggregate_registry_records;
use crate::error::AppResult;
use crate::model::ResearchRunReport;

use super::jsonl::write_jsonl_dataset;
use super::keys::ResearchOutputS3Keys;

pub(super) async fn write_registry_outputs(
    client: &Client,
    bucket: &str,
    keys: &ResearchOutputS3Keys,
    report: &ResearchRunReport,
    written: &mut Vec<String>,
) -> AppResult<()> {
    let registry_records = build_research_aggregate_registry_records(report);
    if registry_records.is_empty() {
        return Ok(());
    }

    write_jsonl_dataset(
        client,
        bucket,
        keys,
        "research-aggregate-registry",
        &registry_records[0].schema_version,
        &registry_records,
        written,
    )
    .await
}

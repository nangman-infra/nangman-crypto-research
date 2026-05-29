use aws_sdk_s3::Client;

use crate::error::AppResult;
use crate::io::ResearchOutputArtifacts;

use super::jsonl::write_jsonl_dataset;
use super::keys::ResearchOutputS3Keys;

pub(super) async fn write_shadow_outputs(
    client: &Client,
    bucket: &str,
    keys: &ResearchOutputS3Keys,
    artifacts: &ResearchOutputArtifacts<'_>,
    written: &mut Vec<String>,
) -> AppResult<()> {
    if artifacts.shadow_validation_runs.is_empty() {
        return Ok(());
    }

    write_jsonl_dataset(
        client,
        bucket,
        keys,
        "shadow-validation-run",
        &artifacts.shadow_validation_runs[0].schema_version,
        artifacts.shadow_validation_runs,
        written,
    )
    .await
}

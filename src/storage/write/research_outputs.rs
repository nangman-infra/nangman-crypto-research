mod jsonl;
mod keys;
mod paper;
mod portfolio;
mod registry;
mod replay;
mod shadow;

use crate::error::{AppError, AppResult};
use crate::io::ResearchOutputArtifacts;
use crate::storage::client::s3_client;
use crate::storage::objects::put_object_json;

use keys::{ResearchOutputS3Keys, s3_uri};

pub async fn write_research_outputs_to_s3(
    bucket: &str,
    prefix: &str,
    artifacts: &ResearchOutputArtifacts<'_>,
) -> AppResult<Vec<String>> {
    if bucket.trim().is_empty() {
        return Err(AppError::config(
            "research output S3 bucket must not be empty",
        ));
    }
    let client = s3_client().await?;
    let mut written = Vec::new();
    let report = artifacts.report;
    let keys = ResearchOutputS3Keys::new(
        prefix,
        artifacts.output_partition_at_ms,
        &report.research_run_report_id,
    )?;
    let report_key = keys.json_object("research-run-report", &report.schema_version, "report.json");
    put_object_json(&client, bucket, &report_key, report).await?;
    written.push(s3_uri(bucket, &report_key));

    replay::write_replay_outputs(&client, bucket, &keys, artifacts, &mut written).await?;
    shadow::write_shadow_outputs(&client, bucket, &keys, artifacts, &mut written).await?;
    paper::write_paper_outputs(&client, bucket, &keys, artifacts, &mut written).await?;
    portfolio::write_portfolio_outputs(&client, bucket, &keys, report, &mut written).await?;
    registry::write_registry_outputs(&client, bucket, &keys, report, &mut written).await?;

    Ok(written)
}

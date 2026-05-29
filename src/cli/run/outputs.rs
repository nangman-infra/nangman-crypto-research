use super::*;
use crate::model::ResearchRunReport;

pub(super) async fn write_research_pipeline_outputs(
    args: &Args,
    report: &ResearchRunReport,
    output_artifacts: &ResearchOutputArtifacts<'_>,
    output_partition_at_ms: i64,
) -> AppResult<Vec<String>> {
    let mut output_files = write_primary_outputs(args, report, output_artifacts).await?;
    output_files.extend(
        write_retest_cycle_source_state_output(args, report, &output_files, output_partition_at_ms)
            .await?,
    );
    Ok(output_files)
}

async fn write_primary_outputs(
    args: &Args,
    report: &ResearchRunReport,
    output_artifacts: &ResearchOutputArtifacts<'_>,
) -> AppResult<Vec<String>> {
    if let Some(output_dir) = args.output_dir.as_deref() {
        return Ok(write_research_outputs(output_dir, output_artifacts)?
            .into_iter()
            .map(|path| path.display().to_string())
            .collect());
    }
    if let Some(output_bucket) = args.output_s3_bucket.as_deref() {
        return write_research_outputs_to_s3(
            output_bucket,
            args.output_s3_prefix.as_deref().unwrap_or(""),
            output_artifacts,
        )
        .await;
    }
    println!("{}", serde_json::to_string_pretty(report)?);
    Ok(Vec::new())
}

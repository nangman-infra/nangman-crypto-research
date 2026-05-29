use super::super::*;
use super::identity::{
    focused_retest_dispatch_manifest_s3_key, focused_retest_summary_output_path,
};

pub(in crate::cli) async fn write_retest_refresh_cycle_focused_manifest_output(
    args: &Args,
    build: &FocusedRetestManifestBuild,
) -> AppResult<FocusedRetestManifestWriteResult> {
    if let Some(output_dir) = args.output_dir.as_deref() {
        let manifest_path = output_dir.join("research-input-manifest.json");
        let summary_path = output_dir.join("research-input-manifest.summary.json");
        return Ok(FocusedRetestManifestWriteResult {
            created: true,
            output_files: vec![
                write_research_input_manifest(&manifest_path, &build.manifest)?
                    .display()
                    .to_string(),
                write_pretty_json_file(&summary_path, &build.summary)?
                    .display()
                    .to_string(),
            ],
        });
    }
    let Some(bucket) = args.output_s3_bucket.as_deref() else {
        return Err(AppError::config(
            "--run-retest-refresh-cycle requires --output-dir or --output-s3-bucket",
        ));
    };
    let packet_id = build
        .manifest
        .research_packet_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            AppError::validation("focused retest manifest missing research_packet_id")
        })?;
    let key = focused_retest_dispatch_manifest_s3_key(packet_id)?;
    match write_research_input_manifest_to_exact_s3_key_if_absent(bucket, &key, &build.manifest)
        .await?
    {
        Some(uri) => Ok(FocusedRetestManifestWriteResult {
            created: true,
            output_files: vec![uri],
        }),
        None => Ok(FocusedRetestManifestWriteResult {
            created: false,
            output_files: vec![format!("s3://{bucket}/{key}")],
        }),
    }
}

pub(in crate::cli) struct FocusedRetestManifestWriteResult {
    pub(in crate::cli) created: bool,
    pub(in crate::cli) output_files: Vec<String>,
}

pub(in crate::cli) async fn write_focused_retest_manifest_outputs(
    args: &Args,
    build: &FocusedRetestManifestBuild,
    output_partition_at_ms: i64,
) -> AppResult<Vec<String>> {
    if let Some(path) = args.focused_retest_manifest_output_file.as_deref() {
        let mut output_files = Vec::new();
        output_files.push(
            write_research_input_manifest(path, &build.manifest)?
                .display()
                .to_string(),
        );
        let summary_path = focused_retest_summary_output_path(args, path);
        output_files.push(
            write_pretty_json_file(&summary_path, &build.summary)?
                .display()
                .to_string(),
        );
        return Ok(output_files);
    }
    let Some(bucket) = args.output_s3_bucket.as_deref() else {
        return Err(AppError::config(
            "--build-focused-retest-manifest requires --focused-retest-manifest-output-file or --output-s3-bucket",
        ));
    };
    let uri = write_research_input_manifest_to_s3(
        bucket,
        args.output_s3_prefix.as_deref().unwrap_or(""),
        &build.manifest,
        output_partition_at_ms,
    )
    .await?;
    Ok(vec![uri])
}

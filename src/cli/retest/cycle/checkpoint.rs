use super::inputs::RetestRefreshCycleInputs;
use super::*;

pub(super) struct RetestRefreshCycleCheckpoint {
    pub(super) validation: crate::retest_cycle::RetestHorizonStatusValidation,
    pub(super) status: serde_json::Value,
    pub(super) output_files: Vec<String>,
}

pub(super) async fn build_retest_refresh_cycle_checkpoint(
    args: &Args,
    inputs: &RetestRefreshCycleInputs,
) -> AppResult<RetestRefreshCycleCheckpoint> {
    let plan = build_retest_horizon_plan(
        &inputs.bundles,
        &inputs.report,
        &RetestHorizonPlanBuildOptions {
            generated_at_ms: inputs.output_partition_at_ms,
            manifest_label: input_manifest_label(args),
            report_label: research_report_label(args),
            latest_l1_as_of_ms: inputs.latest_l1_as_of_ms,
        },
    )?;
    let mut output_files =
        write_retest_refresh_cycle_plan_output(args, &plan, inputs.output_partition_at_ms).await?;

    let status = build_retest_horizon_status(
        &plan,
        None,
        &RetestHorizonStatusBuildOptions {
            generated_at_ms: inputs.output_partition_at_ms,
            plan_file: output_files.first().cloned(),
            driver_summary_file: None,
            checkpoint_s3_write: args.output_s3_bucket.is_some(),
        },
    )?;
    let validation = validate_retest_horizon_status(&status)?;
    output_files.extend(
        write_retest_refresh_cycle_status_output(args, &status, inputs.output_partition_at_ms)
            .await?,
    );

    Ok(RetestRefreshCycleCheckpoint {
        validation,
        status,
        output_files,
    })
}

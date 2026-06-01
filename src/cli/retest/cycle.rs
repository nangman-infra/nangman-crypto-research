use super::*;

mod checkpoint;
mod derived_args;
mod focused;
mod inputs;
mod summary;

use checkpoint::build_retest_refresh_cycle_checkpoint;
use derived_args::derive_latest_state_refresh_args;
use focused::maybe_write_focused_retest_manifest;
use inputs::load_retest_refresh_cycle_inputs;
use summary::retest_refresh_cycle_summary;

pub(in crate::cli) async fn run_retest_refresh_cycle_mode(args: &Args) -> AppResult<RunSummary> {
    let inputs = load_retest_refresh_cycle_inputs(args).await?;
    let mut checkpoint = build_retest_refresh_cycle_checkpoint(args, &inputs).await?;
    let mut focused =
        maybe_write_focused_retest_manifest(args, &inputs, &checkpoint.status).await?;
    checkpoint.output_files.append(&mut focused.output_files);
    Ok(retest_refresh_cycle_summary(checkpoint, focused))
}

pub(in crate::cli) async fn run_retest_refresh_cycle_from_latest_state_mode(
    args: &Args,
) -> AppResult<RunSummary> {
    let Some(bucket) = args.output_s3_bucket.as_deref() else {
        return Err(AppError::config(
            "--run-retest-refresh-cycle-from-latest-state requires --output-s3-bucket",
        ));
    };
    let state = read_latest_retest_cycle_source_state_from_s3(bucket, "").await?;
    let derived_args = derive_latest_state_refresh_args(args, state);
    run_retest_refresh_cycle_mode(&derived_args).await
}

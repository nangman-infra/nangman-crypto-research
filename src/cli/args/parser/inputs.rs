use super::*;

pub(super) fn apply_research_input_arg<I>(
    args: &mut Args,
    arg: &str,
    values: &mut I,
) -> AppResult<bool>
where
    I: Iterator<Item = String>,
{
    match arg {
        "--input-manifest-file" => {
            args.input_manifest_file = Some(absolute_path_arg(
                values.next(),
                "--input-manifest-file requires an absolute path",
            )?);
        }
        "--input-manifest-s3-bucket" => {
            args.input_manifest_s3_bucket = Some(non_empty_arg(
                values.next(),
                "--input-manifest-s3-bucket requires a value",
            )?);
        }
        "--input-manifest-s3-key" => {
            args.input_manifest_s3_key = Some(non_empty_arg(
                values.next(),
                "--input-manifest-s3-key requires a value",
            )?);
        }
        "--research-report-file" => {
            args.research_report_file = Some(absolute_path_arg(
                values.next(),
                "--research-report-file requires an absolute path",
            )?);
        }
        "--research-report-s3-bucket" => {
            args.research_report_s3_bucket = Some(non_empty_arg(
                values.next(),
                "--research-report-s3-bucket requires a value",
            )?);
        }
        "--research-report-s3-key" => {
            args.research_report_s3_key = Some(non_empty_arg(
                values.next(),
                "--research-report-s3-key requires a value",
            )?);
        }
        "--input-bundle-file" => {
            args.input_bundle_file = Some(absolute_path_arg(
                values.next(),
                "--input-bundle-file requires an absolute path",
            )?);
        }
        "--input-bundle-s3-bucket" => {
            args.input_bundle_s3_bucket = Some(non_empty_arg(
                values.next(),
                "--input-bundle-s3-bucket requires a value",
            )?);
        }
        "--input-bundle-s3-key" => {
            args.input_bundle_s3_key = Some(non_empty_arg(
                values.next(),
                "--input-bundle-s3-key requires a value",
            )?);
        }
        "--market-feature-delta-file" => {
            args.market_feature_delta_file = Some(absolute_path_arg(
                values.next(),
                "--market-feature-delta-file requires an absolute path",
            )?);
        }
        "--market-regime-context-file" => {
            args.market_regime_context_file = Some(absolute_path_arg(
                values.next(),
                "--market-regime-context-file requires an absolute path",
            )?);
        }
        "--market-l1-s3-bucket" => {
            args.market_l1_s3_bucket = Some(non_empty_arg(
                values.next(),
                "--market-l1-s3-bucket requires a value",
            )?);
        }
        "--market-feature-delta-s3-key" => {
            args.market_feature_delta_s3_keys.push(non_empty_arg(
                values.next(),
                "--market-feature-delta-s3-key requires a value",
            )?);
        }
        "--market-regime-context-s3-key" => {
            args.market_regime_context_s3_keys.push(non_empty_arg(
                values.next(),
                "--market-regime-context-s3-key requires a value",
            )?);
        }
        "--historical-replay-run-file" => {
            args.historical_replay_run_files.push(absolute_path_arg(
                values.next(),
                "--historical-replay-run-file requires an absolute path",
            )?);
        }
        "--historical-replay-run-index-file" => {
            args.historical_replay_run_index_files
                .push(absolute_path_arg(
                    values.next(),
                    "--historical-replay-run-index-file requires an absolute path",
                )?);
        }
        "--oss-adapter-run-file" => {
            args.oss_adapter_run_files.push(absolute_path_arg(
                values.next(),
                "--oss-adapter-run-file requires an absolute path",
            )?);
        }
        "--shadow-validation-run-file" => {
            args.shadow_validation_run_files.push(absolute_path_arg(
                values.next(),
                "--shadow-validation-run-file requires an absolute path",
            )?);
        }
        "--oss-adapter-run-s3-bucket" => {
            args.oss_adapter_run_s3_bucket = Some(non_empty_arg(
                values.next(),
                "--oss-adapter-run-s3-bucket requires a value",
            )?);
        }
        "--oss-adapter-run-s3-key" => {
            args.oss_adapter_run_s3_keys.push(non_empty_arg(
                values.next(),
                "--oss-adapter-run-s3-key requires a value",
            )?);
        }
        "--shadow-validation-run-s3-bucket" => {
            args.shadow_validation_run_s3_bucket = Some(non_empty_arg(
                values.next(),
                "--shadow-validation-run-s3-bucket requires a value",
            )?);
        }
        "--shadow-validation-run-s3-key" => {
            args.shadow_validation_run_s3_keys.push(non_empty_arg(
                values.next(),
                "--shadow-validation-run-s3-key requires a value",
            )?);
        }
        _ => return Ok(false),
    }
    Ok(true)
}

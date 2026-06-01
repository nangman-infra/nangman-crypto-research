use super::*;

pub(super) fn apply_retest_status_arg<I>(
    args: &mut Args,
    arg: &str,
    values: &mut I,
) -> AppResult<bool>
where
    I: Iterator<Item = String>,
{
    match arg {
        "--retest-horizon-status-output-file" => {
            args.retest_horizon_status_output_file = Some(absolute_path_arg(
                values.next(),
                "--retest-horizon-status-output-file requires an absolute path",
            )?);
        }
        "--retest-driver-summary-file" => {
            args.retest_driver_summary_file = Some(absolute_path_arg(
                values.next(),
                "--retest-driver-summary-file requires an absolute path",
            )?);
        }
        "--retest-horizon-status-file" => {
            args.retest_horizon_status_file = Some(absolute_path_arg(
                values.next(),
                "--retest-horizon-status-file requires an absolute path",
            )?);
        }
        "--retest-horizon-status-s3-bucket" => {
            args.retest_horizon_status_s3_bucket = Some(non_empty_arg(
                values.next(),
                "--retest-horizon-status-s3-bucket requires a value",
            )?);
        }
        "--retest-horizon-status-s3-key" => {
            args.retest_horizon_status_s3_key = Some(non_empty_arg(
                values.next(),
                "--retest-horizon-status-s3-key requires a value",
            )?);
        }
        _ => return Ok(false),
    }

    Ok(true)
}

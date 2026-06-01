use super::*;

pub(super) fn apply_retest_plan_arg<I>(
    args: &mut Args,
    arg: &str,
    values: &mut I,
) -> AppResult<bool>
where
    I: Iterator<Item = String>,
{
    match arg {
        "--retest-horizon-plan-file" => {
            args.retest_horizon_plan_file = Some(absolute_path_arg(
                values.next(),
                "--retest-horizon-plan-file requires an absolute path",
            )?);
        }
        "--retest-horizon-plan-s3-bucket" => {
            args.retest_horizon_plan_s3_bucket = Some(non_empty_arg(
                values.next(),
                "--retest-horizon-plan-s3-bucket requires a value",
            )?);
        }
        "--retest-horizon-plan-s3-key" => {
            args.retest_horizon_plan_s3_key = Some(non_empty_arg(
                values.next(),
                "--retest-horizon-plan-s3-key requires a value",
            )?);
        }
        "--retest-horizon-plan-output-file" => {
            args.retest_horizon_plan_output_file = Some(absolute_path_arg(
                values.next(),
                "--retest-horizon-plan-output-file requires an absolute path",
            )?);
        }
        "--retest-horizon-latest-l1-as-of-ms" => {
            let raw = values.next().ok_or_else(|| {
                AppError::config("--retest-horizon-latest-l1-as-of-ms requires a number")
            })?;
            args.retest_horizon_latest_l1_as_of_ms = Some(parse_non_negative_i64(
                "--retest-horizon-latest-l1-as-of-ms",
                &raw,
            )?);
        }
        _ => return Ok(false),
    }

    Ok(true)
}

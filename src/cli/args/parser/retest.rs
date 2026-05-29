use super::*;

pub(super) fn apply_retest_arg<I>(args: &mut Args, arg: &str, values: &mut I) -> AppResult<bool>
where
    I: Iterator<Item = String>,
{
    match arg {
        "--shadow-cycle-decision-file" => {
            args.shadow_cycle_decision_file = Some(absolute_path_arg(
                values.next(),
                "--shadow-cycle-decision-file requires an absolute path",
            )?);
        }
        "--shadow-cycle-decision-output-file" => {
            args.shadow_cycle_decision_output_file = Some(absolute_path_arg(
                values.next(),
                "--shadow-cycle-decision-output-file requires an absolute path",
            )?);
        }
        "--shadow-cycle-latest-l1-as-of-ms" => {
            let raw = values.next().ok_or_else(|| {
                AppError::config("--shadow-cycle-latest-l1-as-of-ms requires a number")
            })?;
            args.shadow_cycle_latest_l1_as_of_ms = Some(parse_non_negative_i64(
                "--shadow-cycle-latest-l1-as-of-ms",
                &raw,
            )?);
        }
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
        "--focused-retest-manifest-output-file" => {
            args.focused_retest_manifest_output_file = Some(absolute_path_arg(
                values.next(),
                "--focused-retest-manifest-output-file requires an absolute path",
            )?);
        }
        "--focused-retest-summary-output-file" => {
            args.focused_retest_summary_output_file = Some(absolute_path_arg(
                values.next(),
                "--focused-retest-summary-output-file requires an absolute path",
            )?);
        }
        "--focused-retest-next-actions" => {
            let raw = non_empty_arg(
                values.next(),
                "--focused-retest-next-actions requires a comma-separated value",
            )?;
            let actions = parse_focused_retest_actions(&raw);
            if actions.is_empty() {
                return Err(AppError::config(
                    "--focused-retest-next-actions must contain at least one action",
                ));
            }
            args.focused_retest_next_actions = actions;
        }
        "--focused-retest-include-historical-index-refs" => {
            let raw = non_empty_arg(
                values.next(),
                "--focused-retest-include-historical-index-refs requires auto, true, or false",
            )?;
            args.focused_retest_historical_replay_index_ref_mode =
                HistoricalReplayIndexRefMode::parse(&raw)?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}

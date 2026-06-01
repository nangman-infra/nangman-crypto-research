use super::*;

pub(super) fn apply_focused_retest_arg<I>(
    args: &mut Args,
    arg: &str,
    values: &mut I,
) -> AppResult<bool>
where
    I: Iterator<Item = String>,
{
    match arg {
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

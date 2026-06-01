use super::*;

pub(super) fn apply_shadow_cycle_arg<I>(
    args: &mut Args,
    arg: &str,
    values: &mut I,
) -> AppResult<bool>
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
        _ => return Ok(false),
    }

    Ok(true)
}

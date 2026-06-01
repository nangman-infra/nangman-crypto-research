use super::super::*;

pub(super) fn apply_replay_arg<I>(args: &mut Args, arg: &str, values: &mut I) -> AppResult<bool>
where
    I: Iterator<Item = String>,
{
    match arg {
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
        _ => return Ok(false),
    }
    Ok(true)
}

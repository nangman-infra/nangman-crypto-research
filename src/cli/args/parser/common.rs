use super::*;

pub(super) fn apply_common_arg<I>(args: &mut Args, arg: &str, values: &mut I) -> AppResult<bool>
where
    I: Iterator<Item = String>,
{
    match arg {
        "--historical-replay-run-s3-bucket" => {
            args.historical_replay_run_s3_bucket = Some(non_empty_arg(
                values.next(),
                "--historical-replay-run-s3-bucket requires a value",
            )?);
        }
        "--historical-replay-run-s3-key" => {
            args.historical_replay_run_s3_keys.push(non_empty_arg(
                values.next(),
                "--historical-replay-run-s3-key requires a value",
            )?);
        }
        "--historical-replay-run-index-s3-bucket" => {
            args.historical_replay_run_index_s3_bucket = Some(non_empty_arg(
                values.next(),
                "--historical-replay-run-index-s3-bucket requires a value",
            )?);
        }
        "--historical-replay-run-index-s3-key" => {
            args.historical_replay_run_index_s3_keys.push(non_empty_arg(
                values.next(),
                "--historical-replay-run-index-s3-key requires a value",
            )?);
        }
        "--output-dir" => {
            args.output_dir = Some(absolute_path_arg(
                values.next(),
                "--output-dir requires an absolute path",
            )?);
        }
        "--output-s3-bucket" => {
            args.output_s3_bucket = Some(non_empty_arg(
                values.next(),
                "--output-s3-bucket requires a value",
            )?);
        }
        "--output-s3-prefix" => {
            args.output_s3_prefix = Some(non_empty_arg(
                values.next(),
                "--output-s3-prefix requires a value",
            )?);
        }
        "--research-packet-id" => {
            args.research_packet_id =
                non_empty_arg(values.next(), "--research-packet-id requires a value")?;
        }
        "--run-scope" => {
            args.run_scope = non_empty_arg(values.next(), "--run-scope requires a value")?;
        }
        "--now-ms" => {
            let raw = values
                .next()
                .ok_or_else(|| AppError::config("--now-ms requires a number"))?;
            let value = raw
                .parse::<i64>()
                .map_err(|_| AppError::config("--now-ms must be an integer"))?;
            if value < 0 {
                return Err(AppError::config("--now-ms must be non-negative"));
            }
            args.now_ms = Some(value);
        }
        _ => return Ok(false),
    }
    Ok(true)
}

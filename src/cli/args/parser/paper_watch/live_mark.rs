use super::super::*;

pub(super) fn apply_live_mark_arg<I>(args: &mut Args, arg: &str, values: &mut I) -> AppResult<bool>
where
    I: Iterator<Item = String>,
{
    match arg {
        "--paper-watch-live-mark-s3-prefix" => {
            args.paper_watch_live_mark_s3_prefix = non_empty_arg(
                values.next(),
                "--paper-watch-live-mark-s3-prefix requires a value",
            )?;
        }
        "--paper-watch-live-mark-read-limit" => {
            let raw = non_empty_arg(
                values.next(),
                "--paper-watch-live-mark-read-limit requires a positive integer",
            )?;
            args.paper_watch_live_mark_read_limit =
                parse_positive_usize("--paper-watch-live-mark-read-limit", &raw)?;
        }
        "--paper-watch-live-mark-scan-limit" => {
            let raw = non_empty_arg(
                values.next(),
                "--paper-watch-live-mark-scan-limit requires a positive integer",
            )?;
            args.paper_watch_live_mark_scan_limit =
                parse_positive_usize("--paper-watch-live-mark-scan-limit", &raw)?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}

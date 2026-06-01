use super::super::*;

pub(super) fn apply_observer_arg<I>(args: &mut Args, arg: &str, values: &mut I) -> AppResult<bool>
where
    I: Iterator<Item = String>,
{
    match arg {
        "--paper-watch-observer-read-limit" => {
            let raw = non_empty_arg(
                values.next(),
                "--paper-watch-observer-read-limit requires a positive integer",
            )?;
            args.paper_watch_observer_read_limit =
                parse_positive_usize("--paper-watch-observer-read-limit", &raw)?;
        }
        "--paper-watch-observer-scan-limit" => {
            let raw = non_empty_arg(
                values.next(),
                "--paper-watch-observer-scan-limit requires a positive integer",
            )?;
            args.paper_watch_observer_scan_limit =
                parse_positive_usize("--paper-watch-observer-scan-limit", &raw)?;
        }
        "--paper-watch-observer-poll-secs" => {
            let raw = non_empty_arg(
                values.next(),
                "--paper-watch-observer-poll-secs requires a positive integer",
            )?;
            args.paper_watch_observer_poll_secs =
                parse_positive_u64("--paper-watch-observer-poll-secs", &raw)?;
        }
        "--paper-watch-observer-max-iterations" => {
            let raw = non_empty_arg(
                values.next(),
                "--paper-watch-observer-max-iterations requires a non-negative integer",
            )?;
            args.paper_watch_observer_max_iterations =
                parse_non_negative_usize("--paper-watch-observer-max-iterations", &raw)?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}

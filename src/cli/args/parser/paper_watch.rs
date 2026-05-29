use super::*;

pub(super) fn apply_paper_watch_arg<I>(
    args: &mut Args,
    arg: &str,
    values: &mut I,
) -> AppResult<bool>
where
    I: Iterator<Item = String>,
{
    match arg {
        "--paper-watch-candidate-file" => {
            args.paper_watch_candidate_file = Some(absolute_path_arg(
                values.next(),
                "--paper-watch-candidate-file requires an absolute path",
            )?);
        }
        "--paper-watch-candidate-s3-bucket" => {
            args.paper_watch_candidate_s3_bucket = Some(non_empty_arg(
                values.next(),
                "--paper-watch-candidate-s3-bucket requires a value",
            )?);
        }
        "--paper-watch-candidate-s3-key" => {
            args.paper_watch_candidate_s3_key = Some(non_empty_arg(
                values.next(),
                "--paper-watch-candidate-s3-key requires a value",
            )?);
        }
        "--paper-watch-candidate-s3-prefix" => {
            args.paper_watch_candidate_s3_prefix = non_empty_arg(
                values.next(),
                "--paper-watch-candidate-s3-prefix requires a value",
            )?;
        }
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
        "--market-live-tick-file" => {
            args.market_live_tick_file = Some(absolute_path_arg(
                values.next(),
                "--market-live-tick-file requires an absolute path",
            )?);
        }
        "--market-live-nats-url" => {
            args.market_live_nats_url = Some(non_empty_arg(
                values.next(),
                "--market-live-nats-url requires a value",
            )?);
        }
        "--market-live-nats-stream" => {
            args.market_live_nats_stream =
                non_empty_arg(values.next(), "--market-live-nats-stream requires a value")?;
        }
        "--market-live-nats-subject" => {
            args.market_live_nats_subject =
                non_empty_arg(values.next(), "--market-live-nats-subject requires a value")?;
        }
        "--market-live-nats-consumer" => {
            args.market_live_nats_consumer = non_empty_arg(
                values.next(),
                "--market-live-nats-consumer requires a value",
            )?;
        }
        "--market-live-nats-deliver-policy" => {
            args.market_live_nats_deliver_policy = non_empty_arg(
                values.next(),
                "--market-live-nats-deliver-policy requires a value",
            )?;
        }
        "--market-live-nats-batch-size" => {
            let raw = non_empty_arg(
                values.next(),
                "--market-live-nats-batch-size requires a positive integer",
            )?;
            args.market_live_nats_batch_size =
                parse_positive_usize("--market-live-nats-batch-size", &raw)?;
        }
        "--market-live-nats-max-messages" => {
            let raw = non_empty_arg(
                values.next(),
                "--market-live-nats-max-messages requires a positive integer",
            )?;
            args.market_live_nats_max_messages =
                parse_positive_usize("--market-live-nats-max-messages", &raw)?;
        }
        "--market-live-nats-ack-wait-secs" => {
            let raw = non_empty_arg(
                values.next(),
                "--market-live-nats-ack-wait-secs requires a positive integer",
            )?;
            args.market_live_nats_ack_wait_secs =
                parse_positive_u64("--market-live-nats-ack-wait-secs", &raw)?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}

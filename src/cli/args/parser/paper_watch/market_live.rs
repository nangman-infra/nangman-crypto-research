use super::super::*;

pub(super) fn apply_market_live_arg<I>(
    args: &mut Args,
    arg: &str,
    values: &mut I,
) -> AppResult<bool>
where
    I: Iterator<Item = String>,
{
    match arg {
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

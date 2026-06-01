use super::*;

pub(super) fn apply_market_live_env(args: &mut Args) -> AppResult<()> {
    args.market_live_tick_file = env_string("RESEARCH_MARKET_LIVE_TICK_FILE").map(PathBuf::from);
    args.market_live_nats_url = env_string("RESEARCH_MARKET_LIVE_NATS_URL");
    args.market_live_nats_stream = env_string("RESEARCH_MARKET_LIVE_NATS_STREAM")
        .unwrap_or_else(|| DEFAULT_MARKET_LIVE_NATS_STREAM.to_owned());
    args.market_live_nats_subject = env_string("RESEARCH_MARKET_LIVE_NATS_SUBJECT")
        .unwrap_or_else(|| DEFAULT_MARKET_LIVE_NATS_SUBJECT.to_owned());
    args.market_live_nats_consumer = env_string("RESEARCH_MARKET_LIVE_NATS_CONSUMER")
        .unwrap_or_else(|| DEFAULT_MARKET_LIVE_NATS_CONSUMER.to_owned());
    args.market_live_nats_deliver_policy = env_string("RESEARCH_MARKET_LIVE_NATS_DELIVER_POLICY")
        .unwrap_or_else(|| DEFAULT_MARKET_LIVE_NATS_DELIVER_POLICY.to_owned());
    args.market_live_nats_batch_size = env_usize(
        "RESEARCH_MARKET_LIVE_NATS_BATCH_SIZE",
        DEFAULT_MARKET_LIVE_NATS_BATCH_SIZE,
    )?;
    args.market_live_nats_max_messages = env_usize(
        "RESEARCH_MARKET_LIVE_NATS_MAX_MESSAGES",
        DEFAULT_MARKET_LIVE_NATS_MAX_MESSAGES,
    )?;
    args.market_live_nats_ack_wait_secs = env_u64(
        "RESEARCH_MARKET_LIVE_NATS_ACK_WAIT_SECS",
        DEFAULT_MARKET_LIVE_NATS_ACK_WAIT_SECS,
    )?;
    Ok(())
}

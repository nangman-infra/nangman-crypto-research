use super::super::super::*;

pub(in crate::cli) fn paper_watch_observer_nats_config(
    args: &Args,
) -> AppResult<MarketLiveNatsConfig> {
    let Some(url) = args.market_live_nats_url.as_deref() else {
        return Err(AppError::config(
            "--run-paper-watch-observer requires --market-live-nats-url",
        ));
    };
    let consumer = if args.market_live_nats_consumer == DEFAULT_MARKET_LIVE_NATS_CONSUMER {
        "research-paper-watch-observer".to_owned()
    } else {
        args.market_live_nats_consumer.clone()
    };
    let deliver_policy =
        if args.market_live_nats_deliver_policy == DEFAULT_MARKET_LIVE_NATS_DELIVER_POLICY {
            "new".to_owned()
        } else {
            args.market_live_nats_deliver_policy.clone()
        };
    Ok(MarketLiveNatsConfig {
        url: url.to_owned(),
        stream: args.market_live_nats_stream.clone(),
        subject: args.market_live_nats_subject.clone(),
        consumer,
        deliver_policy,
        batch_size: args.market_live_nats_batch_size,
        max_messages: args.market_live_nats_max_messages,
        ack_wait_secs: args.market_live_nats_ack_wait_secs,
        delete_consumer_after_read: false,
    })
}

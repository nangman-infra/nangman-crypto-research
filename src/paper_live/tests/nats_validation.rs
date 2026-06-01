use super::*;

#[test]
fn nats_config_and_tick_validation_reject_bad_inputs() {
    let mut config = MarketLiveNatsConfig {
        url: "http://nats.example:4222".to_owned(),
        stream: "MARKET_LIVE".to_owned(),
        subject: "market_live_tick.created.>".to_owned(),
        consumer: "research-paper-watch-live".to_owned(),
        deliver_policy: "last_per_subject".to_owned(),
        batch_size: 100,
        max_messages: 500,
        ack_wait_secs: 30,
        delete_consumer_after_read: false,
    };
    assert!(validate_nats_config(&config).is_err());
    config.url = "nats://127.0.0.1:4222".to_owned();
    config.batch_size = 0;
    assert!(validate_nats_config(&config).is_err());

    assert!(deliver_policy("all").is_ok());
    assert!(deliver_policy("new").is_ok());
    assert!(deliver_policy("last").is_ok());
    assert!(deliver_policy("last_per_subject").is_ok());
    assert!(deliver_policy("unsupported").is_err());

    let mut bad_tick = tick("tick_bad", "SUI", 1_000, 1.0);
    bad_tick.event_id.clear();
    assert!(validate_tick(&bad_tick).is_err());
    bad_tick.event_id = "tick_bad".to_owned();
    bad_tick.mark_price = Some(f64::NAN);
    assert!(validate_tick(&bad_tick).is_err());
}

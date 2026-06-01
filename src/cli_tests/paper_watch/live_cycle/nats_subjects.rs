use super::*;

#[test]
fn paper_watch_live_cycle_defaults_nats_subjects_to_candidate_symbols() {
    let candidates = serde_json::from_value::<Vec<crate::model::PaperWatchCandidate>>(json!([
        paper_watch_candidate_json("watch_ton", "TON"),
        paper_watch_candidate_json("watch_zec", "ZEC"),
        paper_watch_candidate_json("watch_ton_duplicate", "ton")
    ]))
    .expect("paper watch candidates parse");
    let args = default_args();

    let configs =
        market_live_nats_configs_for_candidates(&args, &candidates, "nats://127.0.0.1:4222", 123);

    let subjects = configs
        .iter()
        .map(|config| config.subject.as_str())
        .collect::<Vec<_>>();
    let consumers = configs
        .iter()
        .map(|config| config.consumer.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        subjects,
        vec![
            "market_live_tick.created.*.ton",
            "market_live_tick.created.*.zec"
        ]
    );
    assert_eq!(
        consumers,
        vec![
            "research-paper-watch-live-123-ton",
            "research-paper-watch-live-123-zec"
        ]
    );
    assert!(
        configs
            .iter()
            .all(|config| config.url == "nats://127.0.0.1:4222")
    );
    assert!(
        configs
            .iter()
            .all(|config| config.delete_consumer_after_read)
    );
}

#[test]
fn paper_watch_live_cycle_keeps_explicit_nats_subject() {
    let candidates = serde_json::from_value::<Vec<crate::model::PaperWatchCandidate>>(json!([
        paper_watch_candidate_json("watch_ton", "TON"),
        paper_watch_candidate_json("watch_zec", "ZEC")
    ]))
    .expect("paper watch candidates parse");
    let args = Args {
        market_live_nats_subject: "market_live_tick.created.binance.ton".to_owned(),
        market_live_nats_consumer: "custom-consumer".to_owned(),
        ..default_args()
    };

    let configs =
        market_live_nats_configs_for_candidates(&args, &candidates, "nats://127.0.0.1:4222", 123);

    assert_eq!(configs.len(), 1);
    assert_eq!(configs[0].subject, "market_live_tick.created.binance.ton");
    assert_eq!(configs[0].consumer, "custom-consumer");
    assert!(!configs[0].delete_consumer_after_read);
}

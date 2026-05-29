use super::config::{deliver_policy, validate_nats_config};
use super::io::read_json_array_or_jsonl_bytes;
use super::nats::validate_tick;
use super::*;
use crate::model::{
    MARKET_LIVE_TICK_SCHEMA_VERSION, MarketLiveTick, PaperExpectedCostProfile,
    PaperExpectedRiskProfile, PaperWatchCandidate, PaperWatchReplaySampleSummary, PaperWatchSafety,
    ResearchBias, SurvivalBand,
};

#[test]
fn live_marks_match_only_paper_watch_symbols() {
    let candidates = vec![candidate("watch_1", "SUI"), candidate("watch_2", "TON")];
    let ticks = vec![
        tick("tick_sui_1", "SUI", 1_000, 1.0),
        tick("tick_eth_1", "ETH", 1_100, 10.0),
        tick("tick_sui_2", "SUI", 1_200, 1.02),
    ];

    let marks = build_paper_watch_live_marks(&candidates, &ticks);

    assert_eq!(marks.len(), 2);
    assert_eq!(marks[0].paper_watch_candidate_id, "watch_1");
    assert_eq!(marks[0].net_return_bps, 0.0);
    assert!((marks[1].net_return_bps - 200.0).abs() < 0.0001);
    assert!(marks[1].safety.paper_only);
    assert!(!marks[1].safety.live_enabled);
    assert!(!marks[1].safety.order_execution_enabled);
    assert!(!marks[1].safety.execution_approval_emitted);
}

#[test]
fn live_marks_keep_separate_entry_prices_per_venue_and_quote_asset() {
    let candidates = vec![candidate("watch_1", "DOGE")];
    let mut upbit_entry = tick("tick_doge_upbit_1", "DOGE", 1_000, 149.0);
    upbit_entry.venue = "upbit".to_owned();
    upbit_entry.symbol_native = "KRW-DOGE".to_owned();
    upbit_entry.quote_asset = "KRW".to_owned();
    let mut binance_entry = tick("tick_doge_binance_1", "DOGE", 1_010, 0.101);
    binance_entry.venue = "binance".to_owned();
    binance_entry.quote_asset = "USDT".to_owned();
    let mut binance_next = tick("tick_doge_binance_2", "DOGE", 1_020, 0.10201);
    binance_next.venue = "binance".to_owned();
    binance_next.quote_asset = "USDT".to_owned();

    let marks =
        build_paper_watch_live_marks(&candidates, &[upbit_entry, binance_entry, binance_next]);

    assert_eq!(marks.len(), 3);
    assert_eq!(marks[0].venue, "upbit");
    assert_eq!(marks[0].entry_mark_price, 149.0);
    assert_eq!(marks[0].net_return_bps, 0.0);
    assert_eq!(marks[1].venue, "binance");
    assert_eq!(marks[1].entry_mark_price, 0.101);
    assert_eq!(marks[1].net_return_bps, 0.0);
    assert!((marks[2].net_return_bps - 100.0).abs() < 0.0001);
    assert!(marks[2].reason_codes.contains(&"venue=binance".to_owned()));
    assert!(
        marks[2]
            .reason_codes
            .contains(&"quote_asset=USDT".to_owned())
    );
}

#[test]
fn live_entry_book_restores_existing_market_entry() {
    let candidates = vec![candidate("watch_1", "DOGE")];
    let mut book = PaperWatchLiveEntryBook::default();
    book.restore_entry("watch_1", "binance", "USDT", 0.100);
    let mut tick = tick("tick_doge_binance_2", "DOGE", 1_020, 0.102);
    tick.venue = "binance".to_owned();
    tick.quote_asset = "USDT".to_owned();

    let marks = build_paper_watch_live_marks_with_entry_book(&candidates, &[tick], &mut book);

    assert_eq!(marks.len(), 1);
    assert_eq!(marks[0].entry_mark_price, 0.100);
    assert!((marks[0].net_return_bps - 200.0).abs() < 0.0001);
}

#[test]
fn absolute_window_marks_force_flat_due_without_order_approval() {
    let mut candidate = candidate("watch_1", "SUI");
    candidate.created_at_ms = 0;
    candidate.target_max_holding_hours = 1;
    candidate.absolute_max_holding_hours = 2;
    let ticks = vec![tick("tick_sui_1", "SUI", 3 * 60 * 60 * 1000, 1.0)];

    let marks = build_paper_watch_live_marks(&[candidate], &ticks);

    assert_eq!(marks[0].lifecycle_state, "force_flat_due");
    assert!(marks[0].reason_codes.contains(&"force_flat_due".to_owned()));
    assert!(!marks[0].safety.order_execution_enabled);
}

#[test]
fn target_window_marks_open_before_force_flat_due() {
    let mut candidate = candidate("watch_1", "SUI");
    candidate.created_at_ms = 0;
    candidate.target_max_holding_hours = 1;
    candidate.absolute_max_holding_hours = 3;
    let ticks = vec![tick("tick_sui_1", "SUI", 2 * 60 * 60 * 1000, 1.0)];

    let marks = build_paper_watch_live_marks(&[candidate], &ticks);

    assert_eq!(marks[0].lifecycle_state, "target_holding_window_open");
    assert!(
        marks[0]
            .reason_codes
            .contains(&"target_holding_window_open".to_owned())
    );
    assert!(!marks[0].safety.execution_approval_emitted);
}

#[test]
fn unsafe_candidates_and_invalid_ticks_are_ignored() {
    let mut unsafe_candidate = candidate("watch_unsafe", "SUI");
    unsafe_candidate.safety.live_enabled = true;
    let mut invalid_tick = tick("tick_bad", "SUI", 1_000, 1.0);
    invalid_tick.schema_version = "old".to_owned();
    let mut missing_price = tick("tick_missing_price", "SUI", 1_100, 1.0);
    missing_price.mark_price = None;

    let marks = build_paper_watch_live_marks(
        &[unsafe_candidate, candidate("watch_safe", "TON")],
        &[
            invalid_tick,
            missing_price,
            tick("tick_ton", "TON", 1_200, 2.0),
        ],
    );

    assert_eq!(marks.len(), 1);
    assert_eq!(marks[0].paper_watch_candidate_id, "watch_safe");
}

#[test]
fn json_array_jsonl_and_single_object_inputs_parse() {
    let first = tick("tick_1", "SUI", 1_000, 1.0);
    let second = tick("tick_2", "TON", 1_100, 2.0);
    let array_bytes = serde_json::to_vec(&vec![first.clone(), second.clone()]).unwrap();
    let jsonl_bytes = format!(
        "{}\n{}\n",
        serde_json::to_string(&first).unwrap(),
        serde_json::to_string(&second).unwrap()
    );
    let object_bytes = serde_json::to_vec(&first).unwrap();

    assert_eq!(
        read_json_array_or_jsonl_bytes::<MarketLiveTick>("array", &array_bytes)
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        read_json_array_or_jsonl_bytes::<MarketLiveTick>("jsonl", jsonl_bytes.as_bytes())
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        read_json_array_or_jsonl_bytes::<MarketLiveTick>("object", &object_bytes)
            .unwrap()
            .len(),
        1
    );
    assert!(read_json_array_or_jsonl_bytes::<MarketLiveTick>("empty", b" \n").is_err());
}

#[test]
fn parses_market_ingest_live_tick_field_names() {
    let raw = br#"{
      "schema_version":"market_live_tick_v1",
      "event_id":"evt_upbit_depth_snapshot_1779886041381_207258",
      "producer_run_id":"market-ingest-upbit-1779884335",
      "venue":"upbit",
      "source_role":"execution",
      "market_type":"spot",
      "event_type":"depth_snapshot",
      "symbol_native":"KRW-WLD",
      "symbol_canonical":"WLD",
      "base_asset":"WLD",
      "quote_asset":"KRW",
      "exchange_timestamp_ms":1779885936566,
      "ingest_timestamp_ms":1779886041381,
      "latency_ms":104815,
      "sequence_id":"upbit:orderbook:ts-1779885936566",
      "sequence_tag":"upbit:orderbook:ts-1779885936566",
      "price_source":"orderbook_top_mid",
      "last_price":null,
      "best_bid_price":532.0,
      "best_ask_price":534.0,
      "mark_price":533.0,
      "trade_volume":null,
      "payload_sha256":"dc4dbd13392d35cde59e3ddc525c54d20e0c3c3abd1c67ff9a5646400e3e795d"
    }"#;

    let tick: MarketLiveTick = serde_json::from_slice(raw).unwrap();

    assert_eq!(tick.symbol_canonical, "WLD");
    assert_eq!(tick.quantity, None);
    assert_eq!(
        tick.raw_payload_sha256,
        "dc4dbd13392d35cde59e3ddc525c54d20e0c3c3abd1c67ff9a5646400e3e795d"
    );
    validate_tick(&tick).unwrap();
}

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

fn candidate(id: &str, symbol: &str) -> PaperWatchCandidate {
    PaperWatchCandidate {
        paper_watch_candidate_id: id.to_owned(),
        candidate_id: format!("cand_{id}"),
        candidate_lifecycle_key: format!("cand_{id}:v1"),
        symbol_canonical: symbol.to_owned(),
        source_research_run_id: "research_run_001".to_owned(),
        source_research_packet_id: "packet_001".to_owned(),
        source_research_bias: ResearchBias::RetestBias,
        historical_survival_band: SurvivalBand::Stable,
        admission_reason_codes: vec!["retest_positive_watch_admitted".to_owned()],
        blocked_promotion_reason_codes: vec!["needs_forward_observation".to_owned()],
        replay_sample_summary: PaperWatchReplaySampleSummary {
            research_aggregate_key: "agg_001".to_owned(),
            replay_run_count: 10,
            completed_count: 5,
            positive_net_count: 3,
            non_positive_net_count: 2,
            missing_market_replay_data_count: 0,
            insufficient_evidence_count: 0,
            effective_completed_sample_weight: 5.0,
            weighted_mean_net_after_cost_bps: Some(10.0),
            weighted_profit_factor_ppm: Some(1_100_000),
        },
        expected_cost_profile: PaperExpectedCostProfile {
            fee_model_version: "fee".to_owned(),
            slippage_model_version: "slippage".to_owned(),
            estimated_cost_bps: Some(8.0),
            cost_stressed_mean_net_after_cost_bps: Some(2.0),
        },
        expected_risk_profile: PaperExpectedRiskProfile {
            survival_band: SurvivalBand::Stable,
            max_drawdown_band: "low".to_owned(),
            positive_net_count: 3,
            non_positive_net_count: 2,
        },
        target_max_holding_hours: 24,
        absolute_max_holding_hours: 72,
        force_flat_policy: "paper_watch_only_no_order_execution".to_owned(),
        paper_start_recommendation: "start_forward_paper_watch".to_owned(),
        safety: PaperWatchSafety {
            paper_only: true,
            live_enabled: false,
            order_execution_enabled: false,
            execution_approval_emitted: false,
        },
        created_at_ms: 1_000,
        schema_version: "paper_watch_candidate_v1".to_owned(),
    }
}

fn tick(id: &str, symbol: &str, timestamp_ms: i64, mark_price: f64) -> MarketLiveTick {
    MarketLiveTick {
        schema_version: MARKET_LIVE_TICK_SCHEMA_VERSION.to_owned(),
        event_id: id.to_owned(),
        producer_run_id: "market_run_001".to_owned(),
        venue: "binance".to_owned(),
        source_role: "reference".to_owned(),
        market_type: "spot".to_owned(),
        event_type: "trade".to_owned(),
        symbol_native: format!("{symbol}USDT"),
        symbol_canonical: symbol.to_owned(),
        base_asset: symbol.to_owned(),
        quote_asset: "USDT".to_owned(),
        exchange_timestamp_ms: timestamp_ms,
        ingest_timestamp_ms: timestamp_ms + 10,
        latency_ms: 10,
        sequence_id: id.to_owned(),
        sequence_tag: "trade_id".to_owned(),
        price_source: "last_price".to_owned(),
        last_price: Some(mark_price),
        best_bid_price: None,
        best_ask_price: None,
        mark_price: Some(mark_price),
        quantity: Some(1.0),
        raw_payload_sha256: "sha256:test".to_owned(),
    }
}

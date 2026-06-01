use super::*;

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

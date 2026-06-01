use super::*;

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

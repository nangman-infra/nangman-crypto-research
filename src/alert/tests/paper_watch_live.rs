use super::*;

#[test]
fn safe_paper_watch_live_mark_batches_are_suppressed() {
    let marks = vec![
        test_live_mark("PENGU", "binance", 12.5),
        test_live_mark("TON", "upbit", -3.0),
        test_live_mark("ZEC", "binance", 0.5),
    ];

    assert!(paper_watch_live_mark_alert_event(&marks, &test_config(AlertPriority::P2)).is_none());
}

#[test]
fn unsafe_paper_watch_live_mark_forces_p0_alert() {
    let mut marks = vec![
        test_live_mark("PENGU", "binance", 12.5),
        test_live_mark("TON", "upbit", -3.0),
        test_live_mark("ZEC", "binance", 0.5),
    ];
    marks[1].safety.live_enabled = true;

    let event = paper_watch_live_mark_alert_event(&marks, &test_config(AlertPriority::P2))
        .expect("unsafe live mark event is created");
    let text = event.text("dev");

    assert_eq!(event.priority, AlertPriority::P0);
    assert!(text.contains("paper-watch live safety boundary changed: 3 marks"));
    assert!(text.contains("관찰 코인: PENGU, TON, ZEC"));
    assert!(text.contains("거래소별 mark: binance 2개, upbit 1개"));
    assert!(text.contains("모의 수익률 범위: -3.00 ~ 12.50 bps"));
    assert!(text.contains("paper-only 안전 경계가 깨진 항목"));
}

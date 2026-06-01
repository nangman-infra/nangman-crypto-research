use super::super::PaperWatchObserverState;
use super::fixtures::{candidate, mark};
use std::collections::BTreeMap;

#[test]
fn observer_state_dedupes_marks_and_summarizes_candidates() {
    let candidate = candidate("watch_1", "XRP");
    let mark = mark("mark_1", &candidate, "binance", 100.0);
    let mut state = PaperWatchObserverState::default();

    state.restore_marks(&[mark.clone(), mark]);
    let snapshot = state.snapshot("observer_1", 1, 2_000, &[candidate], &[]);

    assert_eq!(snapshot.total_live_mark_count, 1);
    assert_eq!(snapshot.candidate_summaries.len(), 1);
    assert_eq!(
        snapshot.candidate_summaries[0].observer_verdict,
        "WATCHING_POSITIVE"
    );
    assert!(!snapshot.safety.order_execution_enabled);
}

#[test]
fn observer_marks_target_window_as_shadow_review_candidate() {
    let mut candidate = candidate("watch_1", "XRP");
    candidate.created_at_ms = 0;
    candidate.target_max_holding_hours = 1;
    let mut mark = mark("mark_1", &candidate, "binance", 50.0);
    mark.lifecycle_state = "target_holding_window_open".to_owned();
    let mut state = PaperWatchObserverState::default();

    state.restore_marks(&[mark]);
    let snapshot = state.snapshot("observer_1", 1, 2 * 60 * 60 * 1000, &[candidate], &[]);

    assert_eq!(
        snapshot.candidate_summaries[0].observer_verdict,
        "SHADOW_REVIEW_CANDIDATE"
    );
}

#[test]
fn observer_snapshot_marks_no_tick_and_risk_review_states() {
    let waiting = candidate("watch_waiting", "PAXG");
    let risky = candidate("watch_risky", "PENGU");
    let mut risky_mark = mark("mark_risky", &risky, "upbit", -250.0);
    risky_mark.lifecycle_state = "watching".to_owned();
    let mut state = PaperWatchObserverState::default();

    state.restore_marks(&[risky_mark]);
    let snapshot = state.snapshot(
        "observer_1",
        1,
        2_000,
        &[waiting.clone(), risky.clone()],
        &[],
    );

    let by_symbol = snapshot
        .candidate_summaries
        .iter()
        .map(|summary| {
            (
                summary.symbol_canonical.as_str(),
                summary.observer_verdict.as_str(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(by_symbol["PAXG"], "WAIT_FOR_LIVE_TICK");
    assert_eq!(by_symbol["PENGU"], "RISK_REVIEW");
}

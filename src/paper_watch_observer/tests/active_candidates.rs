use super::super::active_candidates;
use super::fixtures::candidate;

#[test]
fn active_candidates_exclude_expired_and_unsafe_candidates() {
    let safe = candidate("watch_safe", "DOGE");
    let mut expired = candidate("watch_expired", "XRP");
    expired.created_at_ms = 0;
    expired.absolute_max_holding_hours = 1;
    let mut live_enabled = candidate("watch_live", "TON");
    live_enabled.safety.live_enabled = true;
    let mut order_enabled = candidate("watch_order", "ZEC");
    order_enabled.safety.order_execution_enabled = true;

    let active = active_candidates(
        &[safe.clone(), expired, live_enabled, order_enabled],
        2 * 60 * 60 * 1000,
    );

    assert_eq!(active.len(), 1);
    assert_eq!(
        active[0].paper_watch_candidate_id,
        safe.paper_watch_candidate_id
    );
}

pub(super) fn observer_verdict(
    mark_count: usize,
    latest_return_bps: Option<f64>,
    min_return_bps: Option<f64>,
    max_return_bps: Option<f64>,
    lifecycle_state: &str,
) -> String {
    if mark_count == 0 {
        return "WAIT_FOR_LIVE_TICK".to_owned();
    }
    if min_return_bps.is_some_and(|value| value <= -200.0) {
        return "RISK_REVIEW".to_owned();
    }
    if matches!(
        lifecycle_state,
        "target_holding_window_open" | "force_flat_due"
    ) && latest_return_bps.is_some_and(|value| value > 0.0)
        && min_return_bps.is_some_and(|value| value > -100.0)
    {
        return "SHADOW_REVIEW_CANDIDATE".to_owned();
    }
    if max_return_bps.is_some_and(|value| value > 0.0) {
        return "WATCHING_POSITIVE".to_owned();
    }
    "WATCHING".to_owned()
}

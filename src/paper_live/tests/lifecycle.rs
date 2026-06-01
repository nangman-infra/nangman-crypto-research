use super::*;

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

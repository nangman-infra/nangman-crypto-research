use super::*;

#[test]
fn focused_shadow_cycle_decision_creates_p2_alert() {
    let decision = test_shadow_decision(
        ShadowCycleSchedulerAction::RunFocusedShadowSampleAccumulationResearch,
        false,
    );

    let event = shadow_cycle_decision_alert_event(&decision, &test_config(AlertPriority::P2))
        .expect("focused shadow event is created");

    assert_eq!(event.priority, AlertPriority::P2);
    assert!(
        event
            .text("dev")
            .contains("shadow sample accumulation dispatch")
    );
}

#[test]
fn wait_shadow_cycle_decision_requires_opt_in() {
    let decision = test_shadow_decision(
        ShadowCycleSchedulerAction::WaitUntilTargetWindowMaterializes,
        false,
    );
    assert!(
        shadow_cycle_decision_alert_event(&decision, &test_config(AlertPriority::P3)).is_none()
    );

    let mut config = test_config(AlertPriority::P3);
    config.include_shadow_wait = true;
    let event = shadow_cycle_decision_alert_event(&decision, &config)
        .expect("wait event is created when enabled");

    assert_eq!(event.priority, AlertPriority::P3);
    assert!(event.text("dev").contains("shadow holding window"));
}

#[test]
fn unsafe_shadow_cycle_decision_forces_p0_alert() {
    let decision = test_shadow_decision(ShadowCycleSchedulerAction::Noop, true);

    let event = shadow_cycle_decision_alert_event(&decision, &test_config(AlertPriority::P0))
        .expect("unsafe decision event is created");

    assert_eq!(event.priority, AlertPriority::P0);
    assert!(event.text("dev").contains("order_execution_enabled: true"));
}

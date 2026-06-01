use super::super::*;
use super::fixtures::{order_execution_enabled_json, wait_without_not_before_json};

#[test]
fn rejects_wait_decision_without_not_before_time() {
    let decision: ShadowCycleDecision =
        serde_json::from_str(wait_without_not_before_json()).expect("wait decision parses");
    let err = validate_shadow_cycle_decision(&decision).expect_err("missing not-before rejects");
    assert!(
        err.to_string().contains("run_not_before_ms"),
        "unexpected error: {err:?}"
    );
}

#[test]
fn rejects_decision_that_enables_order_execution() {
    let decision: ShadowCycleDecision =
        serde_json::from_str(order_execution_enabled_json()).expect("unsafe decision parses");
    let err = validate_shadow_cycle_decision(&decision).expect_err("unsafe decision rejects");
    assert!(
        err.to_string().contains("paper/live/order execution"),
        "unexpected error: {err:?}"
    );
}

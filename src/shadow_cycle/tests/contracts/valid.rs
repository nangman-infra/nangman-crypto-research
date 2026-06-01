use super::super::*;
use super::fixtures::{focused_accumulation_decision_json, wait_decision_json};

#[test]
fn validates_wait_decision_contract() {
    let decision: ShadowCycleDecision =
        serde_json::from_str(wait_decision_json()).expect("wait decision parses");
    validate_shadow_cycle_decision(&decision).expect("wait decision validates");
}

#[test]
fn validates_focused_accumulation_decision_contract() {
    let decision: ShadowCycleDecision = serde_json::from_str(focused_accumulation_decision_json())
        .expect("focused decision parses");
    validate_shadow_cycle_decision(&decision).expect("focused decision validates");
}

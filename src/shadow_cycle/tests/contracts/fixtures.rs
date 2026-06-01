#[path = "fixtures/invalid.rs"]
mod invalid;
#[path = "fixtures/valid.rs"]
mod valid;

pub(super) use invalid::{order_execution_enabled_json, wait_without_not_before_json};
pub(super) use valid::{focused_accumulation_decision_json, wait_decision_json};

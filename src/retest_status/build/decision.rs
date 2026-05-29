use super::super::status_parts::{blocked_actions, next_decision_verdict, safe_next_actions};
use super::horizon::HorizonSummary;
use super::stage::StageState;

pub(super) struct DecisionState {
    pub(super) verdict: &'static str,
    pub(super) safe_next_actions: Vec<String>,
    pub(super) blocked_actions: Vec<String>,
}

pub(super) fn build_decision_state(stage: &StageState, horizon: &HorizonSummary) -> DecisionState {
    let verdict = next_decision_verdict(
        stage.promotion_passed,
        horizon.promotion_ready_for_review_count,
        horizon.market_l1_coverage_extension_count,
        horizon.ready_for_replay_count,
        horizon.waiting_for_market_l1_count,
        horizon.sample_accumulation_count,
    );
    let safe_next_actions = safe_next_actions(
        stage.promotion_passed,
        horizon.promotion_ready_for_review_count,
        horizon.market_l1_coverage_extension_count,
        horizon.ready_for_replay_count,
        horizon.waiting_for_market_l1_count,
        horizon.sample_accumulation_count,
    );
    let blocked_actions = blocked_actions(stage.promotion_passed, stage.shadow_created);

    DecisionState {
        verdict,
        safe_next_actions,
        blocked_actions,
    }
}

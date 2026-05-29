use serde_json::{Value, json};

use super::super::status_parts::bool_pointer;
use super::context::StatusBuildContext;

pub(super) struct StageState {
    pub(super) value: Value,
    pub(super) promotion_passed: bool,
    pub(super) shadow_created: bool,
    pub(super) paper_created: bool,
}

pub(super) fn build_stage_state(context: &StatusBuildContext) -> StageState {
    let plan_research_replay_completed = !context.candidate_ids.is_empty()
        && context
            .candidate_ids
            .iter()
            .all(|candidate_id| context.replayed_candidate_ids.contains(candidate_id));
    let promotion_passed = bool_pointer(&context.driver, "/stage_state/promotion_passed")
        .or_else(|| bool_pointer(&context.driver, "/stage_state/promoted"))
        .unwrap_or(false);
    let shadow_created =
        bool_pointer(&context.driver, "/stage_state/shadow_created").unwrap_or(false);
    let paper_created =
        bool_pointer(&context.driver, "/stage_state/paper_created").unwrap_or(false);
    let value = json!({
        "candidate_generated": bool_pointer(&context.driver, "/stage_state/candidate_generated").unwrap_or(!context.rows.is_empty()),
        "research_replay_completed": bool_pointer(&context.driver, "/stage_state/research_replay_completed").unwrap_or(plan_research_replay_completed),
        "promotion_passed": promotion_passed,
        "shadow_created": shadow_created,
        "paper_created": paper_created,
        "live_enabled": false
    });

    StageState {
        value,
        promotion_passed,
        shadow_created,
        paper_created,
    }
}

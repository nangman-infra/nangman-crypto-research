use serde_json::{Value, json};

use super::super::status_parts::{
    action_counts, by_horizon, by_symbol, candidate_horizon_matrix, count_action, count_actions,
    first_symbol, string_field, unique_sorted_strings,
};
use super::context::StatusBuildContext;

pub(super) struct HorizonSummary {
    pub(super) value: Value,
    pub(super) action_counts: Vec<Value>,
    pub(super) ready_for_replay_count: usize,
    pub(super) waiting_for_market_l1_count: usize,
    pub(super) market_l1_coverage_extension_count: usize,
    pub(super) sample_accumulation_count: usize,
    pub(super) promotion_ready_for_review_count: usize,
    pub(super) symbols: Vec<String>,
    pub(super) by_symbol: Vec<Value>,
    pub(super) by_horizon: Vec<Value>,
    pub(super) candidate_horizon_matrix: Vec<Value>,
}

pub(super) fn build_horizon_summary(context: &StatusBuildContext) -> HorizonSummary {
    let action_counts = action_counts(&context.rows);
    let ready_for_replay_count = count_actions(
        &context.rows,
        &[
            "run_research_replay_for_horizon",
            "materialize_completed_native_replay_sample",
            "accumulate_completed_native_replay_samples",
        ],
    );
    let waiting_for_market_l1_count = count_action(&context.rows, "wait_for_market_l1_horizon");
    let market_l1_coverage_extension_count =
        count_action(&context.rows, "extend_market_l1_horizon_coverage");
    let sample_accumulation_count =
        count_action(&context.rows, "accumulate_completed_native_replay_samples");
    let promotion_ready_for_review_count =
        count_action(&context.rows, "promotion_gate_ready_for_review");
    let symbols = unique_sorted_strings(
        context
            .rows
            .iter()
            .filter_map(|row| string_field(row, "primary_symbol").or_else(|| first_symbol(row))),
    );
    let value = json!({
        "candidate_count": context.candidate_ids.len(),
        "horizon_count": context.rows.len(),
        "symbols": symbols,
        "next_action_counts": action_counts,
        "ready_for_replay_count": ready_for_replay_count,
        "waiting_for_market_l1_count": waiting_for_market_l1_count,
        "market_l1_coverage_extension_count": market_l1_coverage_extension_count,
        "sample_accumulation_count": sample_accumulation_count,
        "promotion_ready_for_review_count": promotion_ready_for_review_count
    });

    HorizonSummary {
        value,
        action_counts,
        ready_for_replay_count,
        waiting_for_market_l1_count,
        market_l1_coverage_extension_count,
        sample_accumulation_count,
        promotion_ready_for_review_count,
        symbols,
        by_symbol: by_symbol(&context.rows),
        by_horizon: by_horizon(&context.rows),
        candidate_horizon_matrix: candidate_horizon_matrix(&context.rows),
    }
}

use super::super::build_partition_aggregates;
use super::fixtures::{evidence_bundle, replay_run};
use crate::gate::default_research_gate_policy;

#[test]
fn liquidity_filter_requirement_applies_before_replay_sample_counting() {
    let run_without_bundle = replay_run("replay-a", "cand-a", 1_000);
    let run_with_bundle = replay_run("replay-b", "cand-b", 2_000);
    let bundle = evidence_bundle("cand-b", true);

    let aggregates = build_partition_aggregates(
        &[bundle],
        &[run_without_bundle, run_with_bundle],
        &default_research_gate_policy(),
        3_000,
    );

    assert_eq!(aggregates.len(), 1);
    let aggregate = &aggregates[0];
    assert_eq!(aggregate.completed_count, 2);
    assert_eq!(aggregate.liquidity_filter_materialized_count, 2);
    assert_eq!(aggregate.liquidity_filter_passed_count, 2);
    assert_eq!(aggregate.liquidity_filter_failed_count, 0);
}

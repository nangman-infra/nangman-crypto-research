mod accumulator;
mod evaluation;
mod survival;
#[cfg(test)]
mod tests;

use crate::model::{
    IntelCandidateEvidenceBundle, ReplayRun, ResearchGatePolicy, ResearchPartitionAggregate,
};
use std::collections::BTreeMap;

use accumulator::AggregateAccumulator;

pub fn build_partition_aggregates(
    bundles: &[IntelCandidateEvidenceBundle],
    replay_runs: &[ReplayRun],
    policy: &ResearchGatePolicy,
    gate_as_of_ms: i64,
) -> Vec<ResearchPartitionAggregate> {
    let bundle_by_candidate_id = bundles
        .iter()
        .map(|bundle| (bundle.candidate_id.as_str(), bundle))
        .collect::<BTreeMap<_, _>>();
    let mut accumulators = BTreeMap::<String, AggregateAccumulator>::new();

    for run in replay_runs {
        let accumulator = accumulators
            .entry(run.research_aggregate_key.clone())
            .or_insert_with(|| AggregateAccumulator::new(run));
        accumulator.apply_bundle_requirements(
            bundle_by_candidate_id
                .get(run.source_candidate_id.as_str())
                .copied(),
        );
    }

    for run in replay_runs {
        let accumulator = accumulators
            .get_mut(&run.research_aggregate_key)
            .expect("accumulator was initialized in requirements pass");
        accumulator.add_run(
            run,
            bundle_by_candidate_id
                .get(run.source_candidate_id.as_str())
                .copied(),
            policy,
            gate_as_of_ms,
        );
    }

    accumulators
        .into_values()
        .map(|accumulator| accumulator.finish(policy))
        .collect()
}

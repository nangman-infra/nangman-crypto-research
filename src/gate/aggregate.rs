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
    let mut runs_by_aggregate_key = BTreeMap::<String, Vec<&ReplayRun>>::new();

    for run in replay_runs {
        runs_by_aggregate_key
            .entry(run.research_aggregate_key.clone())
            .or_default()
            .push(run);
    }

    runs_by_aggregate_key
        .into_values()
        .filter_map(|runs| {
            let first_run = runs.first().copied()?;
            let mut accumulator = AggregateAccumulator::new(first_run);

            for run in runs.iter().copied() {
                accumulator.apply_bundle_requirements(
                    bundle_by_candidate_id
                        .get(run.source_candidate_id.as_str())
                        .copied(),
                );
            }

            for run in runs {
                accumulator.add_run(
                    run,
                    bundle_by_candidate_id
                        .get(run.source_candidate_id.as_str())
                        .copied(),
                    policy,
                    gate_as_of_ms,
                );
            }

            Some(accumulator.finish(policy))
        })
        .collect()
}

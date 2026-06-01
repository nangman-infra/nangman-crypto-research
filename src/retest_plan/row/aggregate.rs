use crate::model::{IntelCandidateEvidenceBundle, ResearchPartitionAggregate, ResearchRunReport};

pub(super) struct AggregateMetrics {
    pub(super) completed_count: usize,
    pub(super) effective_completed_sample_weight: f64,
    pub(super) replay_run_count: usize,
    pub(super) inferred_unseen_window_count: usize,
    pub(super) train_validation_split_materialized: bool,
    pub(super) liquidity_filter_materialized_count: usize,
    pub(super) missing_market_replay_data_count: usize,
}

pub(super) fn matching_aggregates<'a>(
    bundle: &IntelCandidateEvidenceBundle,
    horizon: &str,
    report: &'a ResearchRunReport,
) -> Vec<&'a ResearchPartitionAggregate> {
    report
        .partition_aggregates
        .iter()
        .filter(|aggregate| {
            aggregate
                .source_candidate_ids
                .iter()
                .any(|candidate_id| candidate_id == &bundle.candidate_id)
                && horizon_from_aggregate_key(&aggregate.research_aggregate_key) == horizon
        })
        .collect::<Vec<_>>()
}

pub(super) fn aggregate_metrics(aggregates: &[&ResearchPartitionAggregate]) -> AggregateMetrics {
    AggregateMetrics {
        completed_count: aggregates
            .iter()
            .map(|aggregate| aggregate.completed_count)
            .max()
            .unwrap_or(0),
        effective_completed_sample_weight: aggregates
            .iter()
            .map(|aggregate| aggregate.effective_completed_sample_weight)
            .fold(0.0_f64, f64::max),
        replay_run_count: aggregates
            .iter()
            .map(|aggregate| aggregate.replay_run_count)
            .sum(),
        inferred_unseen_window_count: aggregates
            .iter()
            .map(|aggregate| aggregate.inferred_unseen_window_count)
            .max()
            .unwrap_or(0),
        train_validation_split_materialized: aggregates
            .iter()
            .any(|aggregate| aggregate.train_validation_split_summary.materialized),
        liquidity_filter_materialized_count: aggregates
            .iter()
            .map(|aggregate| aggregate.liquidity_filter_materialized_count)
            .max()
            .unwrap_or(0),
        missing_market_replay_data_count: aggregates
            .iter()
            .map(|aggregate| aggregate.missing_market_replay_data_count)
            .sum(),
    }
}

fn horizon_from_aggregate_key(key: &str) -> &str {
    key.split(':').nth(3).unwrap_or("unknown")
}

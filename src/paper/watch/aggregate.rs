use super::eligibility::paper_watch_eligible_aggregate;
use crate::model::ResearchPartitionAggregate;

pub(super) fn best_paper_watch_aggregate<'a>(
    aggregates: &[&'a ResearchPartitionAggregate],
) -> Option<&'a ResearchPartitionAggregate> {
    aggregates
        .iter()
        .copied()
        .filter(|aggregate| paper_watch_eligible_aggregate(aggregate))
        .max_by(|left, right| {
            left.weighted_mean_net_after_cost_bps
                .unwrap_or_default()
                .partial_cmp(&right.weighted_mean_net_after_cost_bps.unwrap_or_default())
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| left.completed_count.cmp(&right.completed_count))
                .then_with(|| left.positive_net_count.cmp(&right.positive_net_count))
        })
}

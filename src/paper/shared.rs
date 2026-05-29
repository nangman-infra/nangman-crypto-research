use crate::model::{
    IntelCandidateEvidenceBundle, PAPER_TRADE_CANDIDATE_SCHEMA_VERSION, ResearchPartitionAggregate,
    ShadowValidationRun, ShadowValidationStatus,
};
use std::collections::BTreeMap;

pub(super) fn aggregate_by_candidate_key(
    aggregates: &[ResearchPartitionAggregate],
) -> BTreeMap<&str, &ResearchPartitionAggregate> {
    let mut values = BTreeMap::new();
    for aggregate in aggregates {
        for candidate_lifecycle_key in &aggregate.source_candidate_lifecycle_keys {
            values.insert(candidate_lifecycle_key.as_str(), aggregate);
        }
    }
    values
}

pub(super) fn aggregates_by_candidate_key(
    aggregates: &[ResearchPartitionAggregate],
) -> BTreeMap<&str, Vec<&ResearchPartitionAggregate>> {
    let mut values = BTreeMap::<&str, Vec<&ResearchPartitionAggregate>>::new();
    for aggregate in aggregates {
        for candidate_lifecycle_key in &aggregate.source_candidate_lifecycle_keys {
            values
                .entry(candidate_lifecycle_key.as_str())
                .or_default()
                .push(aggregate);
        }
    }
    values
}

pub fn is_completed_passed_shadow(run: &ShadowValidationRun) -> bool {
    run.status == ShadowValidationStatus::Completed
        && run.passed
        && run.paper_trade_candidate_contract_version == PAPER_TRADE_CANDIDATE_SCHEMA_VERSION
}

pub(super) fn passed_shadow_by_candidate_key(
    runs: &[ShadowValidationRun],
) -> BTreeMap<&str, &ShadowValidationRun> {
    runs.iter()
        .filter(|run| is_completed_passed_shadow(run))
        .map(|run| (run.candidate_lifecycle_key.as_str(), run))
        .collect()
}

pub(super) fn has_major_failure_event(bundle: &IntelCandidateEvidenceBundle) -> bool {
    bundle.event_types.iter().any(|event_type| {
        matches!(
            event_type.as_str(),
            "exchange_delisting" | "exchange_halt" | "security_incident" | "chain_halt"
        )
    })
}

pub(super) fn max_drawdown_band(aggregate: &ResearchPartitionAggregate) -> String {
    if aggregate.completed_count == 0 {
        return "unknown".to_owned();
    }
    let non_positive_ratio =
        aggregate.non_positive_net_count as f64 / aggregate.completed_count as f64;
    if non_positive_ratio == 0.0 {
        "low".to_owned()
    } else if non_positive_ratio <= 0.2 {
        "controlled".to_owned()
    } else {
        "elevated".to_owned()
    }
}

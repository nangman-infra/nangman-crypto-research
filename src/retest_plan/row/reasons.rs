use crate::model::{ResearchPartitionAggregate, ResearchRunReport};
use std::collections::BTreeSet;

pub(super) fn aggregate_reason_codes(aggregates: &[&ResearchPartitionAggregate]) -> Vec<String> {
    let mut reason_codes = BTreeSet::<String>::new();
    for aggregate in aggregates {
        reason_codes.extend(aggregate.gate_reason_codes.iter().cloned());
        if aggregate.missing_market_replay_data_count > 0 {
            reason_codes.insert("missing_native_replay_market_data".to_owned());
        }
    }
    reason_codes.into_iter().collect()
}

pub(super) fn candidate_reason_codes(
    report: &ResearchRunReport,
    candidate_id: &str,
) -> Vec<String> {
    let mut reason_codes = BTreeSet::<String>::new();
    for finding in &report.summary_findings {
        if finding.candidate_id == candidate_id {
            reason_codes.extend(finding.reason_codes.iter().cloned());
        }
    }
    reason_codes.into_iter().collect()
}

pub(super) fn gate_biases(aggregates: &[&ResearchPartitionAggregate]) -> Vec<String> {
    let mut values = BTreeSet::<String>::new();
    for aggregate in aggregates {
        values.insert(
            serde_json::to_value(&aggregate.gate_bias)
                .ok()
                .and_then(|value| value.as_str().map(ToOwned::to_owned))
                .unwrap_or_else(|| "unknown".to_owned()),
        );
    }
    values.into_iter().collect()
}

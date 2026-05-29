use crate::model::{ResearchPartitionAggregate, SurvivalBand};

pub(super) fn net_result_band(aggregate: &ResearchPartitionAggregate) -> String {
    match aggregate.weighted_mean_net_after_cost_bps {
        Some(value) if value >= 20.0 => "strong_positive".to_owned(),
        Some(value) if value > 0.0 => "positive".to_owned(),
        Some(_) => "non_positive".to_owned(),
        None => "unknown".to_owned(),
    }
}

pub(super) fn survival_result(aggregate: &ResearchPartitionAggregate) -> String {
    if aggregate
        .weighted_mean_net_after_cost_bps
        .is_none_or(|value| value <= 0.0)
    {
        return "failed_fast".to_owned();
    }
    if aggregate.non_positive_net_count > 0 {
        return "mixed".to_owned();
    }
    if aggregate.survival_band == SurvivalBand::Exceptional {
        "survived_strong".to_owned()
    } else {
        "survived".to_owned()
    }
}

pub(super) fn promote_recommendation(survival_result: &str) -> String {
    match survival_result {
        "survived_strong" => "approve_execution_review".to_owned(),
        "survived" => "retest".to_owned(),
        _ => "reject".to_owned(),
    }
}

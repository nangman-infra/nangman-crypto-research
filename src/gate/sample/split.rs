use super::CompletedSample;
use crate::gate::metrics::mean;
use crate::model::TrainValidationSplitSummary;

pub(in crate::gate) fn train_validation_split_summary(
    required: bool,
    completed_samples: &[CompletedSample],
) -> TrainValidationSplitSummary {
    if !required {
        return not_required_summary();
    }

    let mut samples = completed_samples.to_vec();
    samples.sort_by_key(|sample| sample.window_start_ms);
    let split_index = samples.len() / 2;
    let (train, validation) = samples.split_at(split_index);
    let train_nets = sample_nets(train);
    let validation_nets = sample_nets(validation);
    let train_mean_net_after_cost_bps = mean(&train_nets);
    let validation_mean_net_after_cost_bps = mean(&validation_nets);
    let materialized = !train.is_empty() && !validation.is_empty();
    let passed = materialized
        && train_mean_net_after_cost_bps.is_some_and(|value| value > 0.0)
        && validation_mean_net_after_cost_bps.is_some_and(|value| value > 0.0);

    TrainValidationSplitSummary {
        required,
        materialized,
        train_completed_count: train.len(),
        validation_completed_count: validation.len(),
        train_mean_net_after_cost_bps,
        validation_mean_net_after_cost_bps,
        train_positive_net_count: positive_net_count(train),
        validation_positive_net_count: positive_net_count(validation),
        passed,
    }
}

fn not_required_summary() -> TrainValidationSplitSummary {
    TrainValidationSplitSummary {
        required: false,
        materialized: false,
        train_completed_count: 0,
        validation_completed_count: 0,
        train_mean_net_after_cost_bps: None,
        validation_mean_net_after_cost_bps: None,
        train_positive_net_count: 0,
        validation_positive_net_count: 0,
        passed: true,
    }
}

fn sample_nets(samples: &[CompletedSample]) -> Vec<f64> {
    samples
        .iter()
        .map(|sample| sample.net_after_cost_bps)
        .collect()
}

fn positive_net_count(samples: &[CompletedSample]) -> usize {
    samples
        .iter()
        .filter(|sample| sample.net_after_cost_bps > 0.0)
        .count()
}

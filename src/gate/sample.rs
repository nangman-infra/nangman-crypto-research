use crate::model::{RegimeReplaySummary, ResearchGatePolicy, TrainValidationSplitSummary};
use std::collections::BTreeMap;

use super::metrics::mean;

const MS_PER_DAY: i64 = 24 * 60 * 60 * 1000;

#[derive(Debug, Clone)]
pub(super) struct CompletedSample {
    pub(super) window_start_ms: i64,
    pub(super) net_after_cost_bps: f64,
    pub(super) estimated_cost_bps: f64,
    pub(super) market_regime_labels: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DecayBand {
    Fresh,
    Decayed,
    Stale,
    Expired,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct ReplaySampleWeight {
    pub(super) band: DecayBand,
    pub(super) weight: f64,
}

pub(super) fn train_validation_split_summary(
    required: bool,
    completed_samples: &[CompletedSample],
) -> TrainValidationSplitSummary {
    if !required {
        return TrainValidationSplitSummary {
            required,
            materialized: false,
            train_completed_count: 0,
            validation_completed_count: 0,
            train_mean_net_after_cost_bps: None,
            validation_mean_net_after_cost_bps: None,
            train_positive_net_count: 0,
            validation_positive_net_count: 0,
            passed: true,
        };
    }

    let mut samples = completed_samples.to_vec();
    samples.sort_by_key(|sample| sample.window_start_ms);
    let split_index = samples.len() / 2;
    let (train, validation) = samples.split_at(split_index);
    let train_nets = train
        .iter()
        .map(|sample| sample.net_after_cost_bps)
        .collect::<Vec<_>>();
    let validation_nets = validation
        .iter()
        .map(|sample| sample.net_after_cost_bps)
        .collect::<Vec<_>>();
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
        train_positive_net_count: train
            .iter()
            .filter(|sample| sample.net_after_cost_bps > 0.0)
            .count(),
        validation_positive_net_count: validation
            .iter()
            .filter(|sample| sample.net_after_cost_bps > 0.0)
            .count(),
        passed,
    }
}

pub(super) fn regime_summaries(completed_samples: &[CompletedSample]) -> Vec<RegimeReplaySummary> {
    let mut regime_nets = BTreeMap::<String, Vec<f64>>::new();
    for sample in completed_samples {
        for label in &sample.market_regime_labels {
            regime_nets
                .entry(label.clone())
                .or_default()
                .push(sample.net_after_cost_bps);
        }
    }

    regime_nets
        .into_iter()
        .map(|(regime_label, nets)| RegimeReplaySummary {
            regime_label,
            completed_count: nets.len(),
            mean_net_after_cost_bps: mean(&nets),
            positive_net_count: nets.iter().filter(|value| **value > 0.0).count(),
        })
        .collect()
}

pub(super) fn cost_stressed_mean_net_after_cost_bps(
    completed_samples: &[CompletedSample],
    cost_stress_multiplier: f64,
) -> Option<f64> {
    if completed_samples.is_empty() {
        return None;
    }
    let extra_cost_multiplier = (cost_stress_multiplier - 1.0).max(0.0);
    let stressed = completed_samples
        .iter()
        .map(|sample| {
            sample.net_after_cost_bps - (sample.estimated_cost_bps * extra_cost_multiplier)
        })
        .collect::<Vec<_>>();
    mean(&stressed)
}

pub(super) fn replay_sample_weight(
    gate_as_of_ms: i64,
    window_end_ms: i64,
    policy: &ResearchGatePolicy,
) -> ReplaySampleWeight {
    let age_ms = gate_as_of_ms.saturating_sub(window_end_ms);
    let age_days = age_ms / MS_PER_DAY;

    if age_days > policy.expired_sample_max_age_days as i64 {
        return ReplaySampleWeight {
            band: DecayBand::Expired,
            weight: 0.0,
        };
    }
    if age_days > policy.decayed_sample_max_age_days as i64 {
        return ReplaySampleWeight {
            band: DecayBand::Stale,
            weight: policy.stale_sample_weight,
        };
    }
    if age_days > policy.full_weight_sample_max_age_days as i64 {
        return ReplaySampleWeight {
            band: DecayBand::Decayed,
            weight: policy.decayed_sample_weight,
        };
    }
    ReplaySampleWeight {
        band: DecayBand::Fresh,
        weight: 1.0,
    }
}

use super::CompletedSample;
use crate::gate::metrics::mean;

pub(in crate::gate) fn cost_stressed_mean_net_after_cost_bps(
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

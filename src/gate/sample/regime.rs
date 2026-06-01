use super::CompletedSample;
use crate::gate::metrics::mean;
use crate::model::RegimeReplaySummary;
use std::collections::BTreeMap;

pub(in crate::gate) fn regime_summaries(
    completed_samples: &[CompletedSample],
) -> Vec<RegimeReplaySummary> {
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

mod cost;
mod decay;
mod regime;
mod split;
#[cfg(test)]
mod tests;

pub(super) use cost::cost_stressed_mean_net_after_cost_bps;
pub(super) use decay::replay_sample_weight;
pub(super) use regime::regime_summaries;
pub(super) use split::train_validation_split_summary;

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

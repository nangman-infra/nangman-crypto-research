use super::{DecayBand, ReplaySampleWeight};
use crate::model::ResearchGatePolicy;

const MS_PER_DAY: i64 = 24 * 60 * 60 * 1000;

pub(in crate::gate) fn replay_sample_weight(
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

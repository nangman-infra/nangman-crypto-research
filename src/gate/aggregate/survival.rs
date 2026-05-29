use crate::model::{ResearchBias, SurvivalBand};

pub(in crate::gate::aggregate) fn survival_band(
    bias: &ResearchBias,
    completed_count: usize,
    mean_net_after_cost_bps: Option<f64>,
    profit_factor_ppm: Option<u64>,
) -> SurvivalBand {
    match bias {
        ResearchBias::PruneBias => SurvivalBand::Fragile,
        ResearchBias::PromoteToShadowBias => {
            if completed_count >= 100
                && mean_net_after_cost_bps.is_some_and(|value| value >= 20.0)
                && profit_factor_ppm.is_some_and(|value| value >= 2_000_000)
            {
                SurvivalBand::Exceptional
            } else {
                SurvivalBand::Stable
            }
        }
        ResearchBias::RetestBias => {
            if mean_net_after_cost_bps.is_some_and(|value| value > 0.0) {
                SurvivalBand::Conditional
            } else {
                SurvivalBand::Fragile
            }
        }
        ResearchBias::PromoteToPaperBias => SurvivalBand::Conditional,
    }
}

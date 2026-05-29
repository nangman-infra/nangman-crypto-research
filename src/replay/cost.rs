use crate::model::IntelCandidateEvidenceBundle;

pub(super) fn estimated_cost_bps(bundle: &IntelCandidateEvidenceBundle) -> f64 {
    let mut cost = 0.0;
    if bundle.validation_requirements.include_fee {
        cost += 10.0;
    }
    if bundle.validation_requirements.include_slippage {
        cost += 5.0;
    }
    if bundle.validation_requirements.include_latency_assumption {
        cost += 2.0;
    }
    cost
}

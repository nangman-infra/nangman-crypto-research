mod adapter;
mod checks;
mod eligibility;
mod lineage;
mod quality;
mod replay;
mod symbol_resolution;
mod timing;
mod universe;

use super::types::AdmissionDecision;
use crate::model::IntelCandidateEvidenceBundle;

#[cfg(test)]
mod tests;

pub fn validate_bundle_admission(bundle: &IntelCandidateEvidenceBundle) -> AdmissionDecision {
    let mut reasons = Vec::new();

    eligibility::validate(bundle, &mut reasons);
    timing::validate(bundle, &mut reasons);
    universe::validate(bundle, &mut reasons);
    replay::validate(bundle, &mut reasons);
    quality::validate(bundle, &mut reasons);
    symbol_resolution::validate(bundle, &mut reasons);
    adapter::validate(bundle, &mut reasons);
    lineage::validate(bundle, &mut reasons);

    AdmissionDecision {
        admitted: reasons.is_empty(),
        reason_codes: reasons,
    }
}

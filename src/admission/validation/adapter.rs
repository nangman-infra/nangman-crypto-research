use super::checks::require;
use crate::model::IntelCandidateEvidenceBundle;

pub(super) fn validate(bundle: &IntelCandidateEvidenceBundle, reasons: &mut Vec<String>) {
    require(
        bundle
            .validation_requirements
            .required_adapters
            .iter()
            .any(|adapter| adapter == "native_replay"),
        "native_replay_not_required",
        reasons,
    );
}

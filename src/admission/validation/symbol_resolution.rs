use super::checks::require;
use crate::model::IntelCandidateEvidenceBundle;

pub(super) fn validate(bundle: &IntelCandidateEvidenceBundle, reasons: &mut Vec<String>) {
    require(
        super::super::symbol_mapping::has_allowed_symbol_mapping(bundle),
        "not_admitted_symbol_resolution",
        reasons,
    );
}

use super::checks::require;
use crate::model::{CANDIDATE_BUNDLE_SCHEMA_VERSION, IntelCandidateEvidenceBundle};

pub(super) fn validate(bundle: &IntelCandidateEvidenceBundle, reasons: &mut Vec<String>) {
    require(
        bundle.schema_version == CANDIDATE_BUNDLE_SCHEMA_VERSION,
        "invalid_candidate_bundle_schema",
        reasons,
    );
    require(
        bundle.research_eligible,
        "candidate_not_research_eligible",
        reasons,
    );
    require(
        bundle.candidate_class.is_research_eligible(),
        "candidate_class_not_research_eligible",
        reasons,
    );
}

use super::checks::require;
use crate::model::IntelCandidateEvidenceBundle;

pub(super) fn validate(bundle: &IntelCandidateEvidenceBundle, reasons: &mut Vec<String>) {
    require(
        bundle.forbidden_lookahead_boundary_ms == bundle.decision_available_at_ms,
        "lookahead_boundary_mismatch",
        reasons,
    );
    require(
        bundle.decision_available_at_ms >= latest_source_time_ms(bundle),
        "decision_available_at_before_source_time",
        reasons,
    );
}

fn latest_source_time_ms(bundle: &IntelCandidateEvidenceBundle) -> i64 {
    bundle
        .published_at_ms
        .unwrap_or(bundle.decision_available_at_ms)
        .max(bundle.fetched_at_ms)
        .max(bundle.structured_at_ms)
        .max(bundle.candidate_created_at_ms)
}

use super::*;

pub(in crate::cli) fn deterministic_report_created_at_ms(
    bundles: &[IntelCandidateEvidenceBundle],
) -> i64 {
    bundles
        .iter()
        .map(|bundle| {
            bundle
                .created_at_ms
                .max(bundle.candidate_created_at_ms)
                .max(bundle.decision_available_at_ms)
                .max(bundle.forbidden_lookahead_boundary_ms)
        })
        .max()
        .unwrap_or_else(now_ms)
}

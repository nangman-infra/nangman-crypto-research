use super::checks::require;
use crate::model::IntelCandidateEvidenceBundle;

pub(super) fn validate(bundle: &IntelCandidateEvidenceBundle, reasons: &mut Vec<String>) {
    require(
        !bundle.data_quality_summary.status.trim().is_empty(),
        "missing_data_quality_summary_status",
        reasons,
    );
    require(
        bundle
            .data_quality_summary
            .market_data_quality_summary_key
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty()),
        "missing_data_quality_summary",
        reasons,
    );
    require(
        bundle.source_independence.independent_source_count > 0,
        "not_admitted_source_independence",
        reasons,
    );
}

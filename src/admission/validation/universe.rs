use super::checks::require;
use crate::model::IntelCandidateEvidenceBundle;

pub(super) fn validate(bundle: &IntelCandidateEvidenceBundle, reasons: &mut Vec<String>) {
    require(
        !bundle.symbol_universe_snapshot_id.trim().is_empty(),
        "missing_symbol_universe_snapshot_id",
        reasons,
    );
    require(
        bundle.universe_as_of_ms <= bundle.decision_available_at_ms,
        "universe_as_of_after_decision_available",
        reasons,
    );
    require(
        bundle.approved_universe_symbol,
        "not_admitted_universe",
        reasons,
    );
    require(
        !bundle.normalized_symbols.is_empty(),
        "missing_symbol_set",
        reasons,
    );
}

use super::checks::require;
use crate::holding::horizon_within_absolute_limit;
use crate::model::IntelCandidateEvidenceBundle;

pub(super) fn validate(bundle: &IntelCandidateEvidenceBundle, reasons: &mut Vec<String>) {
    require(
        !bundle.allowed_horizons.is_empty()
            && bundle
                .allowed_horizons
                .iter()
                .any(|value| super::super::horizon::horizon_ms(value).is_some()),
        "missing_replay_time_range",
        reasons,
    );
    require(
        bundle
            .allowed_horizons
            .iter()
            .filter_map(|value| super::super::horizon::horizon_ms(value))
            .all(horizon_within_absolute_limit),
        "holding_horizon_contract_violation",
        reasons,
    );
}

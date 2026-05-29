use crate::model::{IntelCandidateEvidenceBundle, ShadowValidationRun, SurvivalBand};
use std::collections::BTreeSet;

pub(super) fn first_symbol(
    bundle: &IntelCandidateEvidenceBundle,
    shadow: &ShadowValidationRun,
) -> String {
    bundle
        .normalized_symbols
        .first()
        .cloned()
        .unwrap_or_else(|| shadow.symbol_canonical.clone())
}

pub(super) fn critical_event_reason(bundle: &IntelCandidateEvidenceBundle) -> Option<&'static str> {
    let critical = bundle
        .event_types
        .iter()
        .map(|event_type| event_type.as_str())
        .collect::<BTreeSet<_>>();
    [
        "exchange_delisting",
        "deposit_withdrawal_halt",
        "chain_halt",
        "exploit",
        "regulatory_ban",
        "exchange_operational_event",
        "liquidity_vanish",
        "market_data_integrity_failure",
    ]
    .into_iter()
    .find(|reason| critical.contains(reason))
}

pub(super) fn infer_market_regime(shadow_validation_runs: &[ShadowValidationRun]) -> String {
    let strongest = shadow_validation_runs
        .iter()
        .map(|run| &run.expected_survival_band)
        .max_by_key(|band| match band {
            SurvivalBand::Fragile => 0,
            SurvivalBand::Conditional => 1,
            SurvivalBand::Stable => 2,
            SurvivalBand::Exceptional => 3,
        });
    match strongest {
        Some(SurvivalBand::Exceptional) => "survival_exceptional".to_owned(),
        Some(SurvivalBand::Stable) => "survival_stable".to_owned(),
        Some(SurvivalBand::Conditional) => "survival_conditional".to_owned(),
        Some(SurvivalBand::Fragile) => "survival_fragile".to_owned(),
        None => "no_active_shadow_candidate".to_owned(),
    }
}

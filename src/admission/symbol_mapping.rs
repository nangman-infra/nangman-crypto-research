use crate::model::{ConfidenceBand, IntelCandidateEvidenceBundle};

pub(super) fn has_allowed_symbol_mapping(bundle: &IntelCandidateEvidenceBundle) -> bool {
    bundle.symbol_resolution_trace.iter().any(|trace| {
        trace.canonical_symbol.as_deref().is_some_and(|symbol| {
            let symbol = symbol.to_ascii_uppercase();
            bundle
                .normalized_symbols
                .iter()
                .any(|candidate| candidate.to_ascii_uppercase() == symbol)
        }) && matches!(
            trace.mapping_confidence,
            ConfidenceBand::Moderate
                | ConfidenceBand::Medium
                | ConfidenceBand::Strong
                | ConfidenceBand::High
        )
    })
}

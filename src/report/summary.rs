use crate::model::IntelCandidateEvidenceBundle;
use std::collections::BTreeSet;

pub(super) fn source_candidate_ids(bundles: &[IntelCandidateEvidenceBundle]) -> Vec<String> {
    bundles
        .iter()
        .map(|bundle| bundle.candidate_id.clone())
        .collect()
}

pub(super) fn top_symbols(bundles: &[IntelCandidateEvidenceBundle]) -> Vec<String> {
    bundles
        .iter()
        .flat_map(|bundle| bundle.normalized_symbols.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub(super) fn top_families(bundles: &[IntelCandidateEvidenceBundle]) -> Vec<String> {
    bundles
        .iter()
        .map(|bundle| bundle.hypothesis_type.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

use crate::model::ResearchInputManifest;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SourceCandidateRef {
    pub(super) uri: String,
    pub(super) candidate_id: Option<String>,
}

pub(super) fn source_candidate_refs(
    source_manifest: &ResearchInputManifest,
) -> Vec<SourceCandidateRef> {
    source_manifest
        .candidate_bundle_refs
        .iter()
        .map(|artifact_ref| SourceCandidateRef {
            uri: artifact_ref.uri.clone(),
            candidate_id: candidate_id_from_uri(&artifact_ref.uri),
        })
        .collect()
}

pub(super) fn selected_candidate_refs(
    source_refs: &[SourceCandidateRef],
    focus_candidate_ids: &[String],
) -> Vec<SourceCandidateRef> {
    let focus_candidate_ids = focus_candidate_ids
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let mut seen_uris = BTreeSet::new();
    let mut selected = Vec::new();
    for source_ref in source_refs {
        let Some(candidate_id) = source_ref.candidate_id.as_deref() else {
            continue;
        };
        if !focus_candidate_ids.contains(candidate_id) || !seen_uris.insert(source_ref.uri.as_str())
        {
            continue;
        }
        selected.push(source_ref.clone());
    }
    selected
}

fn candidate_id_from_uri(uri: &str) -> Option<String> {
    let (_, rest) = uri.split_once("candidate_id=")?;
    let candidate_id = rest.split('/').next()?.trim();
    (!candidate_id.is_empty()).then(|| candidate_id.to_owned())
}

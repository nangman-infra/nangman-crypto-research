use super::checks::require;
use crate::model::IntelCandidateEvidenceBundle;

pub(super) fn validate(bundle: &IntelCandidateEvidenceBundle, reasons: &mut Vec<String>) {
    require(
        !bundle.source_structured_packet_ids.is_empty()
            || !bundle.parent_artifact_ids.is_empty()
            || !bundle.evidence_refs.is_empty()
            || !bundle.source_story_cluster_ids.is_empty(),
        "missing_source_artifact_lineage",
        reasons,
    );
}

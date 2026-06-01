use crate::model::{IntelCandidateEvidenceBundle, ResearchRunReport, ShadowValidationRun};
use std::collections::BTreeMap;

pub(super) struct PortfolioInputIndex<'a> {
    bundles_by_key: BTreeMap<String, &'a IntelCandidateEvidenceBundle>,
    shadows_by_key: BTreeMap<String, &'a ShadowValidationRun>,
}

impl<'a> PortfolioInputIndex<'a> {
    pub(super) fn new(
        report: &'a ResearchRunReport,
        bundles: &'a [IntelCandidateEvidenceBundle],
    ) -> Self {
        Self {
            bundles_by_key: bundles
                .iter()
                .map(|bundle| (bundle.candidate_lifecycle_key.clone(), bundle))
                .collect(),
            shadows_by_key: report
                .shadow_validation_runs
                .iter()
                .map(|run| (run.candidate_lifecycle_key.clone(), run))
                .collect(),
        }
    }

    pub(super) fn bundle(&self, lifecycle_key: &str) -> Option<&'a IntelCandidateEvidenceBundle> {
        self.bundles_by_key.get(lifecycle_key).copied()
    }

    pub(super) fn shadow(&self, lifecycle_key: &str) -> Option<&'a ShadowValidationRun> {
        self.shadows_by_key.get(lifecycle_key).copied()
    }
}

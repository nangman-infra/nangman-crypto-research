use super::index::PortfolioInputIndex;
use crate::model::{IntelCandidateEvidenceBundle, ShadowValidationRun, SummaryFinding};

pub(super) struct PortfolioCandidate<'a> {
    pub(super) lifecycle_key: &'a str,
    pub(super) symbol: String,
    pub(super) family: String,
    pub(super) bundle: &'a IntelCandidateEvidenceBundle,
    pub(super) shadow: &'a ShadowValidationRun,
}

impl<'a> PortfolioCandidate<'a> {
    pub(super) fn from_finding(
        finding: &'a SummaryFinding,
        index: &'a PortfolioInputIndex<'a>,
    ) -> Option<Self> {
        let bundle = index.bundle(&finding.candidate_lifecycle_key)?;
        let shadow = index.shadow(&finding.candidate_lifecycle_key)?;
        Some(Self {
            lifecycle_key: &finding.candidate_lifecycle_key,
            symbol: super::super::symbols::first_symbol(bundle, shadow),
            family: bundle.hypothesis_type.clone(),
            bundle,
            shadow,
        })
    }
}

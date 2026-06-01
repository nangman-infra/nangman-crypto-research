use super::summary::FocusedRetestManifestSummary;
use crate::model::ResearchInputManifest;

#[derive(Debug)]
pub struct FocusedRetestManifestBuild {
    pub manifest: ResearchInputManifest,
    pub summary: FocusedRetestManifestSummary,
}

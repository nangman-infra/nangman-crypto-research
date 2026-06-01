use super::*;

#[derive(Debug)]
pub(in crate::cli) struct ShadowAccumulationDispatch {
    pub(in crate::cli) manifest_uri: String,
    pub(in crate::cli) created: bool,
    pub(in crate::cli) focused_horizon_count: usize,
    pub(in crate::cli) focused_candidate_bundle_refs: usize,
    pub(in crate::cli) deficit_lifecycle_keys: Vec<String>,
}

#[derive(Debug)]
pub(in crate::cli) struct ShadowAccumulationManifestDispatch {
    pub(in crate::cli) key: String,
    pub(in crate::cli) manifest: ResearchInputManifest,
    pub(in crate::cli) focused_horizon_count: usize,
    pub(in crate::cli) focused_candidate_bundle_refs: usize,
    pub(in crate::cli) deficit_lifecycle_keys: Vec<String>,
}

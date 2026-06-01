use super::*;

mod location;

use location::{ArtifactLocation, artifact_location, validate_artifact_ref_location};

pub(in crate::cli) fn all_manifest_refs(
    manifest: &ResearchInputManifest,
) -> Vec<&ResearchArtifactRef> {
    manifest
        .candidate_bundle_refs
        .iter()
        .chain(manifest.market_feature_delta_refs.iter())
        .chain(manifest.market_regime_context_refs.iter())
        .chain(manifest.shadow_validation_run_refs.iter())
        .chain(manifest.hypothesis_harness_result_refs.iter())
        .chain(manifest.oss_adapter_run_refs.iter())
        .chain(manifest.historical_replay_run_refs.iter())
        .chain(manifest.historical_replay_run_index_refs.iter())
        .collect()
}

pub(in crate::cli) fn validate_artifact_ref(artifact_ref: &ResearchArtifactRef) -> AppResult<()> {
    validate_artifact_ref_location(artifact_ref)
}

pub(in crate::cli) async fn read_candidate_bundles_from_ref(
    artifact_ref: &ResearchArtifactRef,
) -> AppResult<Vec<IntelCandidateEvidenceBundle>> {
    match artifact_location(artifact_ref)? {
        ArtifactLocation::Local(path) => read_candidate_bundles(&path),
        ArtifactLocation::S3 { bucket, key } => read_candidate_bundles_from_s3(&bucket, &key).await,
    }
}

pub(in crate::cli) async fn read_market_feature_deltas_from_ref(
    artifact_ref: &ResearchArtifactRef,
) -> AppResult<Vec<MarketFeatureDelta>> {
    match artifact_location(artifact_ref)? {
        ArtifactLocation::Local(path) => read_market_feature_deltas(&path),
        ArtifactLocation::S3 { bucket, key } => {
            read_market_feature_deltas_from_s3(
                &bucket,
                std::slice::from_ref(&key),
                &BTreeSet::new(),
            )
            .await
        }
    }
}

pub(in crate::cli) async fn read_market_regime_contexts_from_ref(
    artifact_ref: &ResearchArtifactRef,
) -> AppResult<Vec<MarketRegimeContext>> {
    match artifact_location(artifact_ref)? {
        ArtifactLocation::Local(path) => read_market_regime_contexts(&path),
        ArtifactLocation::S3 { bucket, key } => {
            read_market_regime_contexts_from_s3(&bucket, std::slice::from_ref(&key)).await
        }
    }
}

pub(in crate::cli) async fn read_replay_runs_from_ref(
    artifact_ref: &ResearchArtifactRef,
) -> AppResult<Vec<ReplayRun>> {
    match artifact_location(artifact_ref)? {
        ArtifactLocation::Local(path) => read_replay_runs(&path),
        ArtifactLocation::S3 { bucket, key } => {
            read_replay_runs_from_s3(&bucket, std::slice::from_ref(&key)).await
        }
    }
}

pub(in crate::cli) async fn read_replay_run_index_records_from_ref(
    artifact_ref: &ResearchArtifactRef,
) -> AppResult<Vec<ReplayRunIndexRecord>> {
    match artifact_location(artifact_ref)? {
        ArtifactLocation::Local(path) => read_replay_run_index_records(&path),
        ArtifactLocation::S3 { bucket, key } => {
            read_replay_run_index_records_from_s3(&bucket, std::slice::from_ref(&key)).await
        }
    }
}

pub(in crate::cli) async fn read_oss_adapter_runs_from_ref(
    artifact_ref: &ResearchArtifactRef,
) -> AppResult<Vec<OssAdapterRun>> {
    match artifact_location(artifact_ref)? {
        ArtifactLocation::Local(path) => read_oss_adapter_runs(&path),
        ArtifactLocation::S3 { bucket, key } => {
            read_oss_adapter_runs_from_s3(&bucket, std::slice::from_ref(&key)).await
        }
    }
}

pub(in crate::cli) async fn read_shadow_validation_runs_from_ref(
    artifact_ref: &ResearchArtifactRef,
) -> AppResult<Vec<ShadowValidationRun>> {
    match artifact_location(artifact_ref)? {
        ArtifactLocation::Local(path) => read_shadow_validation_runs(&path),
        ArtifactLocation::S3 { bucket, key } => {
            read_shadow_validation_runs_from_s3(&bucket, std::slice::from_ref(&key)).await
        }
    }
}

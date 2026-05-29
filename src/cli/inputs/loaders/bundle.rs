use super::super::*;

pub(in crate::cli) async fn read_input_bundles(
    args: &Args,
    manifest: Option<&ResearchInputManifest>,
) -> AppResult<Vec<IntelCandidateEvidenceBundle>> {
    let mut bundles = Vec::new();
    if let Some(path) = args.input_bundle_file.as_deref() {
        append_unique_bundles(&mut bundles, read_candidate_bundles(path)?);
    }
    if let (Some(bucket), Some(key)) = (
        args.input_bundle_s3_bucket.as_deref(),
        args.input_bundle_s3_key.as_deref(),
    ) {
        append_unique_bundles(
            &mut bundles,
            read_candidate_bundles_from_s3(bucket, key).await?,
        );
    }
    if let Some(manifest) = manifest {
        for artifact_ref in &manifest.candidate_bundle_refs {
            append_unique_bundles(
                &mut bundles,
                read_candidate_bundles_from_ref(artifact_ref).await?,
            );
        }
    }
    Ok(bundles)
}

pub(in crate::cli) fn build_replay_runs(
    bundles: &[crate::model::IntelCandidateEvidenceBundle],
    market_deltas: &[MarketFeatureDelta],
    regime_contexts: &[MarketRegimeContext],
) -> Vec<ReplayRun> {
    let mut replay_runs = Vec::new();
    for bundle in bundles {
        let admission = validate_bundle_admission(bundle);
        if !admission.admitted {
            replay_runs.push(build_invalid_replay_run(bundle, &admission));
            continue;
        }
        replay_runs.extend(run_native_replay(bundle, market_deltas, regime_contexts));
    }
    replay_runs
}

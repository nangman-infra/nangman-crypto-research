use super::*;

pub(super) struct RetestRefreshCycleInputs {
    pub(super) output_partition_at_ms: i64,
    pub(super) manifest: ResearchInputManifest,
    pub(super) bundles: Vec<IntelCandidateEvidenceBundle>,
    pub(super) report: crate::model::ResearchRunReport,
    pub(super) latest_l1_as_of_ms: Option<i64>,
}

pub(super) async fn load_retest_refresh_cycle_inputs(
    args: &Args,
) -> AppResult<RetestRefreshCycleInputs> {
    let output_partition_at_ms = args.now_ms.unwrap_or_else(now_ms);
    let manifest = load_input_manifest(args).await?.ok_or_else(|| {
        AppError::config(
            "--run-retest-refresh-cycle requires --input-manifest-file or S3 manifest input",
        )
    })?;
    validate_input_manifest(Some(&manifest))?;
    let bundles = read_input_bundles(args, Some(&manifest)).await?;
    validate_manifest_budget(Some(&manifest), &manifest.runtime_budget_policy)?;
    enforce_budget(
        "candidate_bundle_count",
        bundles.len(),
        manifest.runtime_budget_policy.max_candidate_bundle_count,
    )?;
    let report = load_research_report(args).await?;
    let latest_l1_as_of_ms = retest_plan_latest_l1_as_of_ms(args).await?;
    Ok(RetestRefreshCycleInputs {
        output_partition_at_ms,
        manifest,
        bundles,
        report,
        latest_l1_as_of_ms,
    })
}

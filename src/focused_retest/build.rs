mod candidate_refs;
mod counts;
mod manifest;
mod rows;
mod selection;
mod summary;
mod validation;

use self::manifest::build_manifest;
use self::selection::select_focused_candidates;
use self::summary::build_summary;
use self::validation::validate_options;
use super::types::{FocusedRetestBuildOptions, FocusedRetestManifestBuild};
use crate::error::AppResult;
use crate::model::ResearchInputManifest;
use crate::retest_cycle::validate_retest_horizon_status;
use serde_json::Value;

pub fn build_focused_retest_manifest(
    status: &Value,
    source_manifest: &ResearchInputManifest,
    options: &FocusedRetestBuildOptions,
) -> AppResult<FocusedRetestManifestBuild> {
    validate_retest_horizon_status(status)?;
    validate_options(options)?;

    let selection = select_focused_candidates(status, source_manifest, options)?;
    let carry_historical_index_refs = options
        .historical_replay_index_ref_mode
        .should_carry(&options.next_actions);
    let manifest = build_manifest(
        source_manifest,
        options,
        &selection.selected_refs,
        carry_historical_index_refs,
    );
    let summary = build_summary(
        source_manifest,
        options,
        &manifest,
        selection,
        carry_historical_index_refs,
    );

    Ok(FocusedRetestManifestBuild { manifest, summary })
}

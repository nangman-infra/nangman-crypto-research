use super::super::types::{FocusedRetestBuildOptions, FocusedRetestRow};
use super::candidate_refs::{SourceCandidateRef, selected_candidate_refs, source_candidate_refs};
use super::counts::unique_sorted;
use super::rows::focus_rows;
use crate::error::{AppError, AppResult};
use crate::model::ResearchInputManifest;
use serde_json::Value;

pub(super) struct FocusedRetestSelection {
    pub(super) rows: Vec<FocusedRetestRow>,
    pub(super) focus_candidate_ids: Vec<String>,
    pub(super) selected_refs: Vec<SourceCandidateRef>,
    pub(super) selected_candidate_ids: Vec<String>,
    pub(super) missing_candidate_ref_ids: Vec<String>,
}

pub(super) fn select_focused_candidates(
    status: &Value,
    source_manifest: &ResearchInputManifest,
    options: &FocusedRetestBuildOptions,
) -> AppResult<FocusedRetestSelection> {
    let rows = focus_rows(
        status,
        &options.next_actions,
        &options.candidate_lifecycle_key_filter,
    )?;
    let focus_candidate_ids = unique_sorted(rows.iter().map(|row| row.candidate_id.as_str()));
    let source_refs = source_candidate_refs(source_manifest);
    let selected_refs = selected_candidate_refs(&source_refs, &focus_candidate_ids);
    let selected_candidate_ids = unique_sorted(
        selected_refs
            .iter()
            .filter_map(|candidate_ref| candidate_ref.candidate_id.as_deref()),
    );
    let missing_candidate_ref_ids = focus_candidate_ids
        .iter()
        .filter(|candidate_id| !selected_candidate_ids.contains(candidate_id))
        .cloned()
        .collect::<Vec<_>>();

    if selected_refs.is_empty() {
        return Err(AppError::validation(format!(
            "focused retest selected zero candidate bundle refs; focus_candidate_count={}, missing_candidate_ref_ids={}",
            focus_candidate_ids.len(),
            missing_candidate_ref_ids.join(",")
        )));
    }

    Ok(FocusedRetestSelection {
        rows,
        focus_candidate_ids,
        selected_refs,
        selected_candidate_ids,
        missing_candidate_ref_ids,
    })
}

use crate::model::PaperWatchCandidate;

pub(super) fn lifecycle_state(candidate: &PaperWatchCandidate, holding_elapsed_ms: i64) -> String {
    let absolute_ms = i64::from(candidate.absolute_max_holding_hours) * 60 * 60 * 1000;
    let target_ms = i64::from(candidate.target_max_holding_hours) * 60 * 60 * 1000;
    if holding_elapsed_ms >= absolute_ms {
        "force_flat_due".to_owned()
    } else if holding_elapsed_ms >= target_ms {
        "target_holding_window_open".to_owned()
    } else {
        "watching".to_owned()
    }
}

use crate::hash::stable_id;

pub(super) fn shadow_cycle_decision_id(
    source_verdict: &str,
    latest_l1_as_of_ms: Option<i64>,
    generated_at_ms: i64,
    run_identity_parts: &[String],
) -> String {
    let latest_l1_part = latest_l1_as_of_ms
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_owned());
    let generated_at_part = generated_at_ms.to_string();
    let run_identity_part = run_identity_parts.join("|");
    stable_id(
        "shadow_cycle_decision",
        &[
            source_verdict,
            &latest_l1_part,
            &generated_at_part,
            &run_identity_part,
        ],
    )
}

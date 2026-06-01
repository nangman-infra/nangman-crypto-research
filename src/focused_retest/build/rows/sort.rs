use super::FocusedRetestRow;

pub(super) fn sort_rows(rows: &mut [FocusedRetestRow]) {
    rows.sort_by(|left, right| {
        (
            left.symbol.as_str(),
            left.candidate_id.as_str(),
            horizon_order(&left.horizon),
        )
            .cmp(&(
                right.symbol.as_str(),
                right.candidate_id.as_str(),
                horizon_order(&right.horizon),
            ))
    });
}

pub(in crate::focused_retest::build) fn horizon_order(horizon: &str) -> u8 {
    match horizon {
        "1h" => 1,
        "4h" => 2,
        "24h" | "1d" => 3,
        "72h" => 4,
        "7d" => 5,
        _ => 99,
    }
}

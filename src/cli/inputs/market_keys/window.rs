use super::*;

pub(in crate::cli) fn market_l1_replay_window_starts(
    bundles: &[IntelCandidateEvidenceBundle],
    discovery_cutoff_ms: i64,
) -> Vec<i64> {
    let mut starts = BTreeSet::new();
    for bundle in bundles {
        if !validate_bundle_admission(bundle).admitted {
            continue;
        }
        let Some(max_horizon_ms) = bundle
            .allowed_horizons
            .iter()
            .filter_map(|horizon| horizon_ms(horizon))
            .max()
        else {
            continue;
        };
        let replay_start_ms = bundle.forbidden_lookahead_boundary_ms;
        let replay_end_ms = (replay_start_ms + max_horizon_ms).min(discovery_cutoff_ms);
        if replay_end_ms < replay_start_ms {
            continue;
        }
        let mut window_start_ms = align_market_l1_window_start(replay_start_ms);
        let last_window_start_ms = align_market_l1_window_start(replay_end_ms);
        while window_start_ms <= last_window_start_ms {
            starts.insert(window_start_ms);
            window_start_ms += MARKET_L1_REPLAY_WINDOW_MS;
        }
    }
    starts.into_iter().collect()
}

pub(in crate::cli) fn align_market_l1_window_start(timestamp_ms: i64) -> i64 {
    timestamp_ms.div_euclid(MARKET_L1_REPLAY_WINDOW_MS) * MARKET_L1_REPLAY_WINDOW_MS
}

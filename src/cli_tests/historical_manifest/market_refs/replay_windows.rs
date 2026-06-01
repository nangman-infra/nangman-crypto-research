use super::super::*;

#[test]
fn skips_replay_window_discovery_for_invalid_or_missing_horizon_bundle() {
    let mut invalid_bundle = bundle_json_with_gate_inputs(0, 1_300);
    invalid_bundle["research_eligible"] = json!(false);
    let mut missing_horizon_bundle = bundle_json_with_gate_inputs(1, 1_300);
    missing_horizon_bundle["allowed_horizons"] = json!(["unsupported"]);
    let bundles = vec![
        serde_json::from_value(invalid_bundle).expect("invalid bundle json matches model"),
        serde_json::from_value(missing_horizon_bundle)
            .expect("missing horizon bundle json matches model"),
    ];

    assert_eq!(
        market_l1_replay_window_starts(&bundles, 2_100_000),
        Vec::<i64>::new()
    );
}

#[test]
fn derives_market_l1_replay_window_starts_from_candidate_horizons() {
    let mut bundle = bundle_json_with_gate_inputs(0, 1_300);
    bundle["allowed_horizons"] = json!(["1h", "4h", "24h"]);
    let bundles =
        vec![serde_json::from_value(bundle).expect("candidate bundle test json matches model")];

    assert_eq!(
        market_l1_replay_window_starts(&bundles, 2_100_000),
        vec![0, 900_000, 1_800_000]
    );
}

use super::*;
use crate::gate::default_research_gate_policy;

#[test]
fn train_validation_split_sorts_samples_by_window_start() {
    let samples = vec![
        sample(3_000, 9.0, 1.0, &["bull"]),
        sample(1_000, 3.0, 1.0, &["bull"]),
        sample(2_000, -1.0, 1.0, &["bear"]),
        sample(4_000, 7.0, 1.0, &["bear"]),
    ];

    let summary = train_validation_split_summary(true, &samples);

    assert!(summary.materialized);
    assert_eq!(summary.train_completed_count, 2);
    assert_eq!(summary.validation_completed_count, 2);
    assert_eq!(summary.train_mean_net_after_cost_bps, Some(1.0));
    assert_eq!(summary.validation_mean_net_after_cost_bps, Some(8.0));
    assert_eq!(summary.train_positive_net_count, 1);
    assert_eq!(summary.validation_positive_net_count, 2);
    assert!(summary.passed);
}

#[test]
fn regime_summaries_group_samples_by_label() {
    let samples = vec![
        sample(1_000, 4.0, 1.0, &["bull", "high_vol"]),
        sample(2_000, -2.0, 1.0, &["bull"]),
    ];

    let summaries = regime_summaries(&samples);

    assert_eq!(summaries.len(), 2);
    assert_eq!(summaries[0].regime_label, "bull");
    assert_eq!(summaries[0].completed_count, 2);
    assert_eq!(summaries[0].mean_net_after_cost_bps, Some(1.0));
    assert_eq!(summaries[0].positive_net_count, 1);
    assert_eq!(summaries[1].regime_label, "high_vol");
    assert_eq!(summaries[1].completed_count, 1);
}

#[test]
fn cost_stress_multiplier_charges_only_extra_cost() {
    let samples = vec![sample(1_000, 10.0, 2.0, &[]), sample(2_000, 6.0, 4.0, &[])];

    assert_eq!(
        cost_stressed_mean_net_after_cost_bps(&samples, 2.0),
        Some(5.0)
    );
    assert_eq!(
        cost_stressed_mean_net_after_cost_bps(&samples, 0.5),
        Some(8.0)
    );
}

#[test]
fn replay_sample_weight_respects_decay_boundaries() {
    let policy = default_research_gate_policy();
    let now = 100 * 24 * 60 * 60 * 1000;

    assert_sample_weight(
        replay_sample_weight(now, now, &policy),
        DecayBand::Fresh,
        1.0,
    );
    assert_sample_weight(
        replay_sample_weight(now, now - 31 * 24 * 60 * 60 * 1000, &policy),
        DecayBand::Decayed,
        policy.decayed_sample_weight,
    );
    assert_sample_weight(
        replay_sample_weight(now, now - 61 * 24 * 60 * 60 * 1000, &policy),
        DecayBand::Stale,
        policy.stale_sample_weight,
    );
    assert_sample_weight(
        replay_sample_weight(now, now - 91 * 24 * 60 * 60 * 1000, &policy),
        DecayBand::Expired,
        0.0,
    );
}

fn sample(
    window_start_ms: i64,
    net_after_cost_bps: f64,
    estimated_cost_bps: f64,
    labels: &[&str],
) -> CompletedSample {
    CompletedSample {
        window_start_ms,
        net_after_cost_bps,
        estimated_cost_bps,
        market_regime_labels: labels.iter().map(|label| (*label).to_owned()).collect(),
    }
}

fn assert_sample_weight(actual: ReplaySampleWeight, band: DecayBand, weight: f64) {
    assert_eq!(actual.band, band);
    assert_eq!(actual.weight, weight);
}

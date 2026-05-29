use super::*;
use crate::model::{
    PaperExpectedCostProfile, PaperExpectedRiskProfile, PaperWatchLiveMark,
    PaperWatchReplaySampleSummary, ResearchBias, SurvivalBand,
};
use std::collections::BTreeMap;

#[test]
fn observer_state_dedupes_marks_and_summarizes_candidates() {
    let candidate = candidate("watch_1", "XRP");
    let mark = mark("mark_1", &candidate, "binance", 100.0);
    let mut state = PaperWatchObserverState::default();

    state.restore_marks(&[mark.clone(), mark]);
    let snapshot = state.snapshot("observer_1", 1, 2_000, &[candidate], &[]);

    assert_eq!(snapshot.total_live_mark_count, 1);
    assert_eq!(snapshot.candidate_summaries.len(), 1);
    assert_eq!(
        snapshot.candidate_summaries[0].observer_verdict,
        "WATCHING_POSITIVE"
    );
    assert!(!snapshot.safety.order_execution_enabled);
}

#[test]
fn observer_marks_target_window_as_shadow_review_candidate() {
    let mut candidate = candidate("watch_1", "XRP");
    candidate.created_at_ms = 0;
    candidate.target_max_holding_hours = 1;
    let mut mark = mark("mark_1", &candidate, "binance", 50.0);
    mark.lifecycle_state = "target_holding_window_open".to_owned();
    let mut state = PaperWatchObserverState::default();

    state.restore_marks(&[mark]);
    let snapshot = state.snapshot("observer_1", 1, 2 * 60 * 60 * 1000, &[candidate], &[]);

    assert_eq!(
        snapshot.candidate_summaries[0].observer_verdict,
        "SHADOW_REVIEW_CANDIDATE"
    );
}

#[test]
fn active_candidates_exclude_expired_and_unsafe_candidates() {
    let safe = candidate("watch_safe", "DOGE");
    let mut expired = candidate("watch_expired", "XRP");
    expired.created_at_ms = 0;
    expired.absolute_max_holding_hours = 1;
    let mut live_enabled = candidate("watch_live", "TON");
    live_enabled.safety.live_enabled = true;
    let mut order_enabled = candidate("watch_order", "ZEC");
    order_enabled.safety.order_execution_enabled = true;

    let active = active_candidates(
        &[safe.clone(), expired, live_enabled, order_enabled],
        2 * 60 * 60 * 1000,
    );

    assert_eq!(active.len(), 1);
    assert_eq!(
        active[0].paper_watch_candidate_id,
        safe.paper_watch_candidate_id
    );
}

#[test]
fn observer_snapshot_marks_no_tick_and_risk_review_states() {
    let waiting = candidate("watch_waiting", "PAXG");
    let risky = candidate("watch_risky", "PENGU");
    let mut risky_mark = mark("mark_risky", &risky, "upbit", -250.0);
    risky_mark.lifecycle_state = "watching".to_owned();
    let mut state = PaperWatchObserverState::default();

    state.restore_marks(&[risky_mark]);
    let snapshot = state.snapshot(
        "observer_1",
        1,
        2_000,
        &[waiting.clone(), risky.clone()],
        &[],
    );

    let by_symbol = snapshot
        .candidate_summaries
        .iter()
        .map(|summary| {
            (
                summary.symbol_canonical.as_str(),
                summary.observer_verdict.as_str(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(by_symbol["PAXG"], "WAIT_FOR_LIVE_TICK");
    assert_eq!(by_symbol["PENGU"], "RISK_REVIEW");
}

fn candidate(id: &str, symbol: &str) -> crate::model::PaperWatchCandidate {
    crate::model::PaperWatchCandidate {
        paper_watch_candidate_id: id.to_owned(),
        candidate_id: format!("cand_{id}"),
        candidate_lifecycle_key: format!("cand_{id}:v1"),
        symbol_canonical: symbol.to_owned(),
        source_research_run_id: "research_run_001".to_owned(),
        source_research_packet_id: "packet_001".to_owned(),
        source_research_bias: ResearchBias::RetestBias,
        historical_survival_band: SurvivalBand::Stable,
        admission_reason_codes: vec!["retest_positive_watch_admitted".to_owned()],
        blocked_promotion_reason_codes: vec!["needs_forward_observation".to_owned()],
        replay_sample_summary: PaperWatchReplaySampleSummary {
            research_aggregate_key: "agg_001".to_owned(),
            replay_run_count: 10,
            completed_count: 5,
            positive_net_count: 3,
            non_positive_net_count: 2,
            missing_market_replay_data_count: 0,
            insufficient_evidence_count: 0,
            effective_completed_sample_weight: 5.0,
            weighted_mean_net_after_cost_bps: Some(10.0),
            weighted_profit_factor_ppm: Some(1_100_000),
        },
        expected_cost_profile: PaperExpectedCostProfile {
            fee_model_version: "fee".to_owned(),
            slippage_model_version: "slippage".to_owned(),
            estimated_cost_bps: Some(8.0),
            cost_stressed_mean_net_after_cost_bps: Some(2.0),
        },
        expected_risk_profile: PaperExpectedRiskProfile {
            survival_band: SurvivalBand::Stable,
            max_drawdown_band: "low".to_owned(),
            positive_net_count: 3,
            non_positive_net_count: 2,
        },
        target_max_holding_hours: 24,
        absolute_max_holding_hours: 72,
        force_flat_policy: "paper_watch_only_no_order_execution".to_owned(),
        paper_start_recommendation: "start_forward_paper_watch".to_owned(),
        safety: crate::model::PaperWatchSafety {
            paper_only: true,
            live_enabled: false,
            order_execution_enabled: false,
            execution_approval_emitted: false,
        },
        created_at_ms: 1_000,
        schema_version: "paper_watch_candidate_v1".to_owned(),
    }
}

fn mark(
    id: &str,
    candidate: &crate::model::PaperWatchCandidate,
    venue: &str,
    net_return_bps: f64,
) -> PaperWatchLiveMark {
    PaperWatchLiveMark {
        paper_watch_live_mark_id: id.to_owned(),
        paper_watch_candidate_id: candidate.paper_watch_candidate_id.clone(),
        candidate_id: candidate.candidate_id.clone(),
        candidate_lifecycle_key: candidate.candidate_lifecycle_key.clone(),
        symbol_canonical: candidate.symbol_canonical.clone(),
        source_research_run_id: candidate.source_research_run_id.clone(),
        source_market_live_event_id: format!("event_{id}"),
        venue: venue.to_owned(),
        mark_source: "market_live_tick".to_owned(),
        marked_at_ms: 2_000,
        exchange_timestamp_ms: 2_000,
        ingest_timestamp_ms: 2_010,
        holding_elapsed_ms: 1_000,
        entry_mark_price: 1.0,
        current_mark_price: 1.0 + net_return_bps / 10_000.0,
        net_return_bps,
        target_max_holding_hours: candidate.target_max_holding_hours,
        absolute_max_holding_hours: candidate.absolute_max_holding_hours,
        lifecycle_state: "watching".to_owned(),
        reason_codes: vec![
            "paper_watch_live_mark".to_owned(),
            format!("venue={venue}"),
            "quote_asset=USDT".to_owned(),
        ],
        safety: crate::model::PaperWatchSafety {
            paper_only: true,
            live_enabled: false,
            order_execution_enabled: false,
            execution_approval_emitted: false,
        },
        schema_version: "paper_watch_live_mark_v1".to_owned(),
    }
}

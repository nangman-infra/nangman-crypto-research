use super::super::*;
use crate::model::{
    PaperExpectedCostProfile, PaperExpectedRiskProfile, PaperWatchCandidate, PaperWatchLiveMark,
    PaperWatchReplaySampleSummary, PaperWatchSafety, SurvivalBand,
};

pub(crate) fn test_paper_watch_candidate(symbol: &str) -> PaperWatchCandidate {
    PaperWatchCandidate {
        paper_watch_candidate_id: format!("watch_{symbol}"),
        candidate_id: format!("cand_{symbol}"),
        candidate_lifecycle_key: format!("life_{symbol}"),
        symbol_canonical: symbol.to_owned(),
        source_research_run_id: "report_test".to_owned(),
        source_research_packet_id: "packet_test".to_owned(),
        source_research_bias: ResearchBias::RetestBias,
        historical_survival_band: SurvivalBand::Stable,
        admission_reason_codes: vec!["retest_positive_watch_admitted".to_owned()],
        blocked_promotion_reason_codes: vec![
            "native_replay_positive_but_promotion_blocked".to_owned(),
        ],
        replay_sample_summary: PaperWatchReplaySampleSummary {
            research_aggregate_key: "agg_test".to_owned(),
            replay_run_count: 10,
            completed_count: 5,
            positive_net_count: 3,
            non_positive_net_count: 2,
            missing_market_replay_data_count: 0,
            insufficient_evidence_count: 0,
            effective_completed_sample_weight: 5.0,
            weighted_mean_net_after_cost_bps: Some(12.5),
            weighted_profit_factor_ppm: Some(1_200_000),
        },
        expected_cost_profile: PaperExpectedCostProfile {
            fee_model_version: "fee".to_owned(),
            slippage_model_version: "slippage".to_owned(),
            estimated_cost_bps: Some(8.0),
            cost_stressed_mean_net_after_cost_bps: Some(4.5),
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
        safety: PaperWatchSafety {
            paper_only: true,
            live_enabled: false,
            order_execution_enabled: false,
            execution_approval_emitted: false,
        },
        created_at_ms: 1_000,
        schema_version: "paper_watch_candidate_v1".to_owned(),
    }
}

pub(crate) fn test_live_mark(symbol: &str, venue: &str, net_return_bps: f64) -> PaperWatchLiveMark {
    PaperWatchLiveMark {
        paper_watch_live_mark_id: format!("mark_{symbol}_{venue}"),
        paper_watch_candidate_id: format!("watch_{symbol}"),
        candidate_id: format!("cand_{symbol}"),
        candidate_lifecycle_key: format!("life_{symbol}"),
        symbol_canonical: symbol.to_owned(),
        source_research_run_id: "report_test".to_owned(),
        source_market_live_event_id: format!("tick_{symbol}_{venue}"),
        venue: venue.to_owned(),
        mark_source: "market_live_tick".to_owned(),
        marked_at_ms: 2_000,
        exchange_timestamp_ms: 1_900,
        ingest_timestamp_ms: 2_000,
        holding_elapsed_ms: 1_000,
        entry_mark_price: 100.0,
        current_mark_price: 100.0,
        net_return_bps,
        target_max_holding_hours: 24,
        absolute_max_holding_hours: 72,
        lifecycle_state: "watching".to_owned(),
        reason_codes: vec!["paper_watch_live_mark".to_owned()],
        safety: PaperWatchSafety {
            paper_only: true,
            live_enabled: false,
            order_execution_enabled: false,
            execution_approval_emitted: false,
        },
        schema_version: "paper_watch_live_mark_v1".to_owned(),
    }
}

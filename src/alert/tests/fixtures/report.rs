use super::super::*;
use crate::model::{
    DEFAULT_RESEARCH_GATE_POLICY_VERSION, HypothesisOutput, PortfolioAllocationSnapshot,
    ResearchGatePolicy, ResearchRunReport, ResearchRunStatus, SummaryFinding,
};

pub(crate) fn finding(candidate_id: &str, bias: ResearchBias, reasons: &[&str]) -> SummaryFinding {
    SummaryFinding {
        candidate_id: candidate_id.to_owned(),
        candidate_lifecycle_key: format!("life_{candidate_id}"),
        bias,
        reason_codes: reasons.iter().map(|reason| (*reason).to_owned()).collect(),
    }
}

pub(crate) fn test_report(summary_findings: Vec<SummaryFinding>) -> ResearchRunReport {
    ResearchRunReport {
        research_run_report_id: "report_test".to_owned(),
        research_packet_id: "packet_test".to_owned(),
        source_candidate_ids: Vec::new(),
        run_scope: "test_scope".to_owned(),
        partition_count: 0,
        top_symbols: Vec::new(),
        top_families: Vec::new(),
        surviving_candidate_keys: Vec::new(),
        pruned_candidate_keys: Vec::new(),
        retest_candidate_keys: Vec::new(),
        shadow_validation_runs: Vec::new(),
        paper_watch_candidates: Vec::new(),
        paper_trade_candidates: Vec::new(),
        oss_adapter_run_ids: Vec::new(),
        oss_adapter_reject_count: 0,
        portfolio_allocation_snapshot: None,
        portfolio_risk_reject_events: Vec::new(),
        portfolio_reduce_only_signals: Vec::new(),
        hypothesis_outputs: HypothesisOutput::None,
        research_gate_policy: test_policy(),
        partition_aggregates: Vec::new(),
        summary_findings,
        research_run_status: ResearchRunStatus::Completed,
        created_at_ms: 0,
        replay_run_ids: Vec::new(),
        invalid_input_candidate_keys: Vec::new(),
        schema_version: "research_run_report_v1".to_owned(),
    }
}

pub(crate) fn test_portfolio_snapshot() -> PortfolioAllocationSnapshot {
    PortfolioAllocationSnapshot {
        portfolio_allocation_snapshot_id: "portfolio_test".to_owned(),
        schema_version: "portfolio_allocation_snapshot_v1".to_owned(),
        allocation_policy_version: "test_policy".to_owned(),
        computed_at_ms: 0,
        market_regime: "test".to_owned(),
        active_candidate_count: 1,
        max_total_notional_pct: 0.1,
        max_symbol_notional_pct: 0.1,
        max_candidate_notional_pct: 0.1,
        max_regime_notional_pct: 0.1,
        candidate_allocations: Vec::new(),
        rejected_candidates: Vec::new(),
        reason_codes: vec!["portfolio_notional_non_zero".to_owned()],
    }
}

fn test_policy() -> ResearchGatePolicy {
    ResearchGatePolicy {
        policy_version: DEFAULT_RESEARCH_GATE_POLICY_VERSION.to_owned(),
        min_completed_samples_for_shadow: 30,
        min_win_rate_ppm_for_shadow: 500_000,
        min_profit_factor_ppm_for_shadow: 1_300_000,
        min_mean_net_after_cost_bps_for_shadow: 5.0,
        max_missing_or_insufficient_ratio_ppm_for_shadow: 200_000,
        min_market_regime_label_count_for_shadow: 1,
        cost_stress_multiplier_for_shadow: 2.0,
        full_weight_sample_max_age_days: 30,
        decayed_sample_max_age_days: 60,
        expired_sample_max_age_days: 90,
        decayed_sample_weight: 0.7,
        stale_sample_weight: 0.4,
        allow_promote_to_paper_bias: false,
    }
}

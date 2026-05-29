use super::*;
use crate::model::{
    DEFAULT_RESEARCH_GATE_POLICY_VERSION, HypothesisOutput, PaperExpectedCostProfile,
    PaperExpectedRiskProfile, PaperWatchReplaySampleSummary, PaperWatchSafety,
    PortfolioAllocationSnapshot, ResearchGatePolicy, ResearchRunStatus, ShadowCycleDecisionSafety,
    ShadowCycleSampleState, SummaryFinding, SurvivalBand,
};

#[test]
fn parses_priority_and_filters_by_minimum_priority() {
    assert_eq!(AlertPriority::parse("p0"), Some(AlertPriority::P0));
    assert_eq!(AlertPriority::parse(" P3 "), Some(AlertPriority::P3));
    assert_eq!(AlertPriority::parse("later"), None);

    let config = test_config(AlertPriority::P2);
    assert!(config.allows(AlertPriority::P0));
    assert!(config.allows(AlertPriority::P2));
    assert!(!config.allows(AlertPriority::P3));
}

#[test]
fn retest_summary_message_is_human_readable() {
    let report = test_report(vec![SummaryFinding {
        candidate_id: "cand_001".to_owned(),
        candidate_lifecycle_key: "life_001".to_owned(),
        bias: ResearchBias::RetestBias,
        reason_codes: vec![
            "promotion_sample_count_below_minimum".to_owned(),
            "unseen_window_validation_not_proven".to_owned(),
        ],
    }]);
    let mut config = test_config(AlertPriority::P3);
    config.include_retest_summary = true;
    let event = research_report_alert_event(&report, &[], &config).expect("event is created");
    let text = event.text("dev");

    assert!(text.contains("[P3][research-app] RETEST"));
    assert!(text.contains("결론:"));
    assert!(text.contains("현재 상태:"));
    assert!(text.contains("주요 원인:"));
    assert!(text.contains("다음 행동:"));
    assert!(text.contains("안전 상태:"));
    assert!(text.contains("승급에 필요한 샘플 수가 부족함: 1개"));
}

#[test]
fn promote_to_shadow_alert_has_p2_priority() {
    let report = test_report(vec![SummaryFinding {
        candidate_id: "cand_001".to_owned(),
        candidate_lifecycle_key: "life_001".to_owned(),
        bias: ResearchBias::PromoteToShadowBias,
        reason_codes: vec!["deterministic_shadow_gate_passed".to_owned()],
    }]);
    let config = test_config(AlertPriority::P2);
    let event = research_report_alert_event(&report, &[], &config).expect("event is created");

    assert_eq!(event.priority, AlertPriority::P2);
    assert!(event.text("dev").contains("PROMOTE_TO_SHADOW"));
}

#[test]
fn paper_watch_alert_has_p2_priority_without_execution_language() {
    let mut report = test_report(vec![SummaryFinding {
        candidate_id: "cand_001".to_owned(),
        candidate_lifecycle_key: "life_001".to_owned(),
        bias: ResearchBias::RetestBias,
        reason_codes: vec!["native_replay_positive_but_promotion_blocked".to_owned()],
    }]);
    let candidates = vec![test_paper_watch_candidate("TON")];
    report.paper_watch_candidates = candidates
        .iter()
        .map(|candidate| candidate.paper_watch_candidate_id.clone())
        .collect();
    let event = research_report_alert_event(&report, &candidates, &test_config(AlertPriority::P2))
        .expect("paper watch event is created");
    let text = event.text("dev");

    assert_eq!(event.priority, AlertPriority::P2);
    assert!(text.contains("모의 관찰 후보 1개 발생"));
    assert!(text.contains("관찰 코인: TON"));
    assert!(text.contains("완료 replay 샘플: 5/10"));
    assert!(text.contains("실제 주문: 꺼짐"));
    assert!(!text.contains("LIVE_READY: true"));
}

#[test]
fn retest_summary_is_quiet_by_default() {
    let report = test_report(vec![SummaryFinding {
        candidate_id: "cand_001".to_owned(),
        candidate_lifecycle_key: "life_001".to_owned(),
        bias: ResearchBias::RetestBias,
        reason_codes: vec!["promotion_sample_count_below_minimum".to_owned()],
    }]);

    assert!(research_report_alert_event(&report, &[], &test_config(AlertPriority::P3)).is_none());
}

#[test]
fn retest_summary_is_filtered_when_min_priority_is_too_high() {
    let report = test_report(vec![SummaryFinding {
        candidate_id: "cand_001".to_owned(),
        candidate_lifecycle_key: "life_001".to_owned(),
        bias: ResearchBias::RetestBias,
        reason_codes: vec!["promotion_sample_count_below_minimum".to_owned()],
    }]);
    let mut config = test_config(AlertPriority::P2);
    config.include_retest_summary = true;

    assert!(research_report_alert_event(&report, &[], &config).is_none());
}

#[test]
fn promote_to_paper_alert_has_p1_priority() {
    let report = test_report(vec![SummaryFinding {
        candidate_id: "cand_001".to_owned(),
        candidate_lifecycle_key: "life_001".to_owned(),
        bias: ResearchBias::PromoteToPaperBias,
        reason_codes: vec!["shadow_validation_passed".to_owned()],
    }]);
    let event = research_report_alert_event(&report, &[], &test_config(AlertPriority::P1))
        .expect("paper event is created");

    assert_eq!(event.priority, AlertPriority::P1);
    assert!(
        event
            .text("prod")
            .contains("실제 주문과 live trading은 계속 꺼둡니다.")
    );
}

#[test]
fn non_zero_portfolio_notional_forces_p0_alert() {
    let mut report = test_report(vec![SummaryFinding {
        candidate_id: "cand_001".to_owned(),
        candidate_lifecycle_key: "life_001".to_owned(),
        bias: ResearchBias::RetestBias,
        reason_codes: vec!["portfolio_notional_non_zero".to_owned()],
    }]);
    report.portfolio_allocation_snapshot = Some(test_portfolio_snapshot());

    let event = research_report_alert_event(&report, &[], &test_config(AlertPriority::P0))
        .expect("portfolio event is created");

    assert_eq!(event.priority, AlertPriority::P0);
    assert!(event.text("dev").contains("실제 투자 비중: 0.1000"));
}

#[test]
fn reason_summary_limits_to_top_five_reasons() {
    let report = test_report(vec![
        finding(
            "cand_001",
            ResearchBias::PromoteToShadowBias,
            &[
                "native_replay_horizon_not_materialized",
                "needs_unseen_window_validation",
            ],
        ),
        finding(
            "cand_002",
            ResearchBias::PromoteToShadowBias,
            &[
                "native_replay_horizon_not_materialized",
                "promotion_sample_count_below_minimum",
            ],
        ),
        finding(
            "cand_003",
            ResearchBias::PromoteToShadowBias,
            &[
                "native_replay_horizon_not_materialized",
                "promotion_sample_count_below_minimum",
            ],
        ),
        finding(
            "cand_004",
            ResearchBias::PromoteToShadowBias,
            &["liquidity_filter_not_materialized"],
        ),
        finding(
            "cand_005",
            ResearchBias::PromoteToShadowBias,
            &["liquidity_filter_no_positive_volume_observed"],
        ),
        finding(
            "cand_006",
            ResearchBias::PromoteToShadowBias,
            &["native_replay_positive_but_survival_not_proven"],
        ),
    ]);

    let reasons = top_reason_lines(&report, AlertPriority::P2);

    assert_eq!(reasons.len(), 5);
    assert_eq!(
        reasons[0],
        "목표 보유시간만큼의 시장 데이터가 아직 부족함: 3개"
    );
    assert!(reasons.contains(&"승급에 필요한 샘플 수가 부족함: 2개".to_owned()));
    assert!(!reasons.iter().any(|line| line.contains("positive_volume")));
}

#[test]
fn safe_paper_watch_live_mark_batches_are_suppressed() {
    let marks = vec![
        test_live_mark("PENGU", "binance", 12.5),
        test_live_mark("TON", "upbit", -3.0),
        test_live_mark("ZEC", "binance", 0.5),
    ];

    assert!(paper_watch_live_mark_alert_event(&marks, &test_config(AlertPriority::P2)).is_none());
}

#[test]
fn unsafe_paper_watch_live_mark_forces_p0_alert() {
    let mut marks = vec![
        test_live_mark("PENGU", "binance", 12.5),
        test_live_mark("TON", "upbit", -3.0),
        test_live_mark("ZEC", "binance", 0.5),
    ];
    marks[1].safety.live_enabled = true;

    let event = paper_watch_live_mark_alert_event(&marks, &test_config(AlertPriority::P2))
        .expect("unsafe live mark event is created");
    let text = event.text("dev");

    assert_eq!(event.priority, AlertPriority::P0);
    assert!(text.contains("paper-watch live safety boundary changed: 3 marks"));
    assert!(text.contains("관찰 코인: PENGU, TON, ZEC"));
    assert!(text.contains("거래소별 mark: binance 2개, upbit 1개"));
    assert!(text.contains("모의 수익률 범위: -3.00 ~ 12.50 bps"));
    assert!(text.contains("paper-only 안전 경계가 깨진 항목"));
}

#[test]
fn focused_shadow_cycle_decision_creates_p2_alert() {
    let decision = test_shadow_decision(
        ShadowCycleSchedulerAction::RunFocusedShadowSampleAccumulationResearch,
        false,
    );

    let event = shadow_cycle_decision_alert_event(&decision, &test_config(AlertPriority::P2))
        .expect("focused shadow event is created");

    assert_eq!(event.priority, AlertPriority::P2);
    assert!(
        event
            .text("dev")
            .contains("shadow sample accumulation dispatch")
    );
}

#[test]
fn wait_shadow_cycle_decision_requires_opt_in() {
    let decision = test_shadow_decision(
        ShadowCycleSchedulerAction::WaitUntilTargetWindowMaterializes,
        false,
    );
    assert!(
        shadow_cycle_decision_alert_event(&decision, &test_config(AlertPriority::P3)).is_none()
    );

    let mut config = test_config(AlertPriority::P3);
    config.include_shadow_wait = true;
    let event = shadow_cycle_decision_alert_event(&decision, &config)
        .expect("wait event is created when enabled");

    assert_eq!(event.priority, AlertPriority::P3);
    assert!(event.text("dev").contains("shadow holding window"));
}

#[test]
fn unsafe_shadow_cycle_decision_forces_p0_alert() {
    let decision = test_shadow_decision(ShadowCycleSchedulerAction::Noop, true);

    let event = shadow_cycle_decision_alert_event(&decision, &test_config(AlertPriority::P0))
        .expect("unsafe decision event is created");

    assert_eq!(event.priority, AlertPriority::P0);
    assert!(event.text("dev").contains("order_execution_enabled: true"));
}

#[test]
fn pipeline_alert_event_key_is_hour_partitioned() {
    let key = pipeline_alert_event_key(
        "pipeline-alert-event/schema=pipeline_alert_event_v1/",
        1779937200123,
        "research-app",
        "P2",
        "pipeline_alert_abc123",
    )
    .expect("timestamp is valid");

    assert_eq!(
        key,
        "pipeline-alert-event/schema=pipeline_alert_event_v1/dt=2026-05-28/hour=03/app=research-app/priority=P2/pipeline_alert_abc123.json"
    );
}

#[test]
fn pipeline_alert_event_payload_preserves_operator_sections() {
    let event = AlertEvent {
        priority: AlertPriority::P2,
        title: "모의 관찰 후보 2개 발생".to_owned(),
        conclusion: "paper-watch 후보를 관찰 단계로 올렸습니다.".to_owned(),
        current_state: vec!["관찰 코인: DOGE, XRP".to_owned()],
        reasons: vec!["과거 검증은 긍정적이지만 승급 조건이 아직 부족함: 2개".to_owned()],
        next_actions: vec!["실제 주문 없이 live mark를 계속 누적합니다.".to_owned()],
        safety: vec!["실제 주문: 꺼짐".to_owned()],
    };

    let payload = PipelineAlertEvent::from_alert_event(
        &event,
        "pipeline_alert_test",
        "pipeline_alert_dedupe_test",
        "dev",
        1779937200123,
    );
    let json = serde_json::to_value(&payload).expect("payload serializes");

    assert_eq!(json["schema_version"], "pipeline_alert_event_v1");
    assert_eq!(json["app"], APP_NAME);
    assert_eq!(json["priority"], "P2");
    assert_eq!(json["title"], "모의 관찰 후보 2개 발생");
    assert_eq!(json["current_state"][0], "관찰 코인: DOGE, XRP");
    assert_eq!(json["safety"][0], "실제 주문: 꺼짐");
    assert_eq!(json["created_at_ms"], 1779937200123_i64);
}

#[test]
fn build_pipeline_alert_delivery_writes_expected_key_and_body() {
    let config = test_config(AlertPriority::P2);
    let event = AlertEvent {
        priority: AlertPriority::P2,
        title: "PROMOTE_TO_SHADOW 후보 발생".to_owned(),
        conclusion: "후보가 shadow 관측 단계로 올라갔습니다.".to_owned(),
        current_state: vec!["shadow 관찰 생성: 1개".to_owned()],
        reasons: vec!["deterministic_shadow_gate_passed: 1개".to_owned()],
        next_actions: vec!["주문 실행은 계속 꺼둡니다.".to_owned()],
        safety: vec!["실제 주문: 꺼짐".to_owned()],
    };

    let delivery =
        build_pipeline_alert_delivery(&config, &event, 1779937200123).expect("delivery is built");
    let payload: serde_json::Value =
        serde_json::from_slice(&delivery.body).expect("body is valid json");

    assert!(delivery.key.starts_with(
            "pipeline-alert-event/schema=pipeline_alert_event_v1/dt=2026-05-28/hour=03/app=research-app/priority=P2/"
        ));
    assert_eq!(payload["schema_version"], "pipeline_alert_event_v1");
    assert_eq!(payload["environment"], "dev");
    assert_eq!(payload["title"], "PROMOTE_TO_SHADOW 후보 발생");
    assert_eq!(payload["next_actions"][0], "주문 실행은 계속 꺼둡니다.");
    assert!(
        payload["event_id"]
            .as_str()
            .is_some_and(|value| value.starts_with("pipeline_alert_"))
    );
    assert!(
        payload["dedupe_key"]
            .as_str()
            .is_some_and(|value| value.starts_with("pipeline_alert_dedupe_"))
    );
}

#[test]
fn s3_key_token_replaces_unsafe_characters() {
    assert_eq!(s3_key_token("research app/P2"), "research_app_P2");
    assert_eq!(
        s3_key_token("pipeline_alert_abc-123"),
        "pipeline_alert_abc-123"
    );
}

#[test]
fn pipeline_alert_event_key_rejects_invalid_timestamp() {
    let error = pipeline_alert_event_key(
        DEFAULT_PIPELINE_ALERT_S3_PREFIX,
        i64::MAX,
        "research-app",
        "P2",
        "event",
    )
    .expect_err("invalid timestamp is rejected");

    assert_eq!(error, "created_at_ms is outside supported timestamp range");
}

fn test_config(min_priority: AlertPriority) -> AlertConfig {
    AlertConfig {
        event_bucket: "nangman-crypto-dev-research-962214".to_owned(),
        event_prefix: DEFAULT_PIPELINE_ALERT_S3_PREFIX.to_owned(),
        environment: "dev".to_owned(),
        min_priority,
        include_retest_summary: false,
        include_shadow_wait: false,
    }
}

fn finding(candidate_id: &str, bias: ResearchBias, reasons: &[&str]) -> SummaryFinding {
    SummaryFinding {
        candidate_id: candidate_id.to_owned(),
        candidate_lifecycle_key: format!("life_{candidate_id}"),
        bias,
        reason_codes: reasons.iter().map(|reason| (*reason).to_owned()).collect(),
    }
}

fn test_paper_watch_candidate(symbol: &str) -> PaperWatchCandidate {
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

fn test_live_mark(symbol: &str, venue: &str, net_return_bps: f64) -> PaperWatchLiveMark {
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

fn test_report(summary_findings: Vec<SummaryFinding>) -> ResearchRunReport {
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

fn test_portfolio_snapshot() -> PortfolioAllocationSnapshot {
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

fn test_shadow_decision(
    scheduler_action: ShadowCycleSchedulerAction,
    unsafe_boundary: bool,
) -> ShadowCycleDecision {
    ShadowCycleDecision {
        schema_version: "research_shadow_cycle_decision_v1".to_owned(),
        generated_at: "2026-05-26T00:00:00Z".to_owned(),
        decision_id: "decision_test".to_owned(),
        source_cycle_summary_file: None,
        run_dir: None,
        scheduler_action,
        source_verdict: "WAIT_FOR_TARGET_HOLDING_WINDOW".to_owned(),
        run_not_before_ms: Some(1),
        run_not_before_at: Some("2026-05-26T01:00:00Z".to_owned()),
        run_not_before_source: Some("target_window".to_owned()),
        focused_research_manifest_file: None,
        focused_research_summary_file: None,
        latest_l1_as_of_ms: Some(1),
        shadow_sample_state: ShadowCycleSampleState {
            shadow_validation_count: 1,
            target_window_materialized_count: 0,
            candidate_lifecycle_count: 1,
            partially_materialized_candidate_count: 0,
            pending_target_window_candidate_count: 1,
            total_sample_deficit: 3,
            symbols: vec!["DOGE".to_owned()],
        },
        safe_next_actions: vec!["wait for target window".to_owned()],
        blocked_actions: vec!["paper is blocked".to_owned()],
        safety: ShadowCycleDecisionSafety {
            s3_write: false,
            ecs_task_started: false,
            dispatcher_mode_changed: false,
            local_decision_only: true,
            shadow_status_mutated: false,
            paper_live_enabled: false,
            live_enabled: unsafe_boundary,
            order_execution_enabled: unsafe_boundary,
        },
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

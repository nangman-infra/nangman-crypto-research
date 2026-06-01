use super::*;

#[tokio::test]
async fn paper_watch_live_cycle_marks_live_ticks_without_order_approval() {
    let root = test_root("paper-watch-live-cycle");
    let candidates_file = root.join("paper-watch-candidates.json");
    let ticks_file = root.join("market-live-ticks.json");
    let output = root.join("out");

    write_json(
        &candidates_file,
        &json!([{
            "paper_watch_candidate_id": "watch_001",
            "candidate_id": "cand_001",
            "candidate_lifecycle_key": "cand_001:v1",
            "symbol_canonical": "SUI",
            "source_research_run_id": "research_run_001",
            "source_research_packet_id": "packet_001",
            "source_research_bias": "RETEST_BIAS",
            "historical_survival_band": "stable",
            "admission_reason_codes": ["retest_positive_watch_admitted"],
            "blocked_promotion_reason_codes": ["needs_forward_observation"],
            "replay_sample_summary": {
                "research_aggregate_key": "agg_001",
                "replay_run_count": 10,
                "completed_count": 5,
                "positive_net_count": 3,
                "non_positive_net_count": 2,
                "missing_market_replay_data_count": 0,
                "insufficient_evidence_count": 0,
                "effective_completed_sample_weight": 5.0,
                "weighted_mean_net_after_cost_bps": 12.5,
                "weighted_profit_factor_ppm": 1200000
            },
            "expected_cost_profile": {
                "fee_model_version": "fee",
                "slippage_model_version": "slippage",
                "estimated_cost_bps": 8.0,
                "cost_stressed_mean_net_after_cost_bps": 4.5
            },
            "expected_risk_profile": {
                "survival_band": "stable",
                "max_drawdown_band": "low",
                "positive_net_count": 3,
                "non_positive_net_count": 2
            },
            "target_max_holding_hours": 24,
            "absolute_max_holding_hours": 72,
            "force_flat_policy": "paper_watch_only_no_order_execution",
            "paper_start_recommendation": "start_forward_paper_watch",
            "safety": {
                "paper_only": true,
                "live_enabled": false,
                "order_execution_enabled": false,
                "execution_approval_emitted": false
            },
            "created_at_ms": 1_000,
            "schema_version": "paper_watch_candidate_v1"
        }]),
    );
    write_json(
        &ticks_file,
        &json!([
            market_live_tick_json("tick_001", "SUI", 2_000, 1.0),
            market_live_tick_json("tick_002", "ETH", 2_100, 10.0),
            market_live_tick_json("tick_003", "SUI", 2_200, 1.03)
        ]),
    );

    let summary = run(Args {
        run_paper_watch_live_cycle: true,
        paper_watch_candidate_file: Some(candidates_file),
        market_live_tick_file: Some(ticks_file),
        output_dir: Some(output),
        now_ms: Some(120_000_000),
        ..default_args()
    })
    .await
    .expect("paper watch live cycle succeeds");

    assert_eq!(summary.paper_watch_live_marks_created, 2);
    let mark_file = output_file_containing(&summary, "/paper-watch-live-mark/");
    let mark_text = fs::read_to_string(mark_file).expect("paper watch mark output exists");
    assert!(!mark_text.contains("EXECUTION_APPROVED"));
    assert!(!mark_text.contains("LIVE_READY"));
    let marks = mark_text
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).expect("mark json parses"))
        .collect::<Vec<_>>();
    assert_eq!(marks[0]["safety"]["paper_only"], json!(true));
    assert_eq!(marks[0]["safety"]["live_enabled"], json!(false));
    assert_eq!(marks[0]["safety"]["order_execution_enabled"], json!(false));
    assert_eq!(marks[0]["net_return_bps"], json!(0.0));
    assert_eq!(marks[1]["source_market_live_event_id"], json!("tick_003"));
}

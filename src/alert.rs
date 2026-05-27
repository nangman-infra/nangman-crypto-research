use crate::model::{
    ResearchBias, ResearchRunReport, ShadowCycleDecision, ShadowCycleSchedulerAction,
};
use serde_json::json;
use std::collections::BTreeMap;
use std::env;
use std::time::Duration;

const APP_NAME: &str = "research-app";
const DEFAULT_ENVIRONMENT: &str = "dev";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertPriority {
    P0,
    P1,
    P2,
    P3,
}

impl AlertPriority {
    fn as_str(self) -> &'static str {
        match self {
            Self::P0 => "P0",
            Self::P1 => "P1",
            Self::P2 => "P2",
            Self::P3 => "P3",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_uppercase().as_str() {
            "P0" => Some(Self::P0),
            "P1" => Some(Self::P1),
            "P2" => Some(Self::P2),
            "P3" => Some(Self::P3),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
struct AlertConfig {
    webhook_url: String,
    environment: String,
    min_priority: AlertPriority,
    include_retest_summary: bool,
    include_shadow_wait: bool,
}

impl AlertConfig {
    fn from_env() -> Option<Self> {
        let webhook_url = env::var("NANGMAN_ALERT_WEBHOOK_URL")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .or_else(|| {
                env::var("MATTERMOST_WEBHOOK_URL")
                    .ok()
                    .filter(|value| !value.trim().is_empty())
            })?;
        let environment =
            env::var("NANGMAN_ALERT_ENV").unwrap_or_else(|_| DEFAULT_ENVIRONMENT.to_owned());
        let min_priority = env::var("NANGMAN_ALERT_MIN_PRIORITY")
            .ok()
            .and_then(|value| AlertPriority::parse(&value))
            .unwrap_or(AlertPriority::P2);
        let include_retest_summary = env_bool("NANGMAN_ALERT_INCLUDE_RETEST_SUMMARY");
        let include_shadow_wait = env_bool("NANGMAN_ALERT_INCLUDE_SHADOW_WAIT");

        Some(Self {
            webhook_url,
            environment,
            min_priority,
            include_retest_summary,
            include_shadow_wait,
        })
    }

    fn allows(&self, priority: AlertPriority) -> bool {
        priority <= self.min_priority
    }
}

#[derive(Debug, Clone)]
pub struct AlertEvent {
    priority: AlertPriority,
    title: String,
    conclusion: String,
    current_state: Vec<String>,
    reasons: Vec<String>,
    next_actions: Vec<String>,
    safety: Vec<String>,
}

impl AlertEvent {
    fn text(&self, environment: &str) -> String {
        let mut sections = vec![
            format!("[{}][{}] {}", self.priority.as_str(), APP_NAME, self.title),
            String::new(),
            "결론:".to_owned(),
            self.conclusion.clone(),
            String::new(),
            "현재 상태:".to_owned(),
        ];
        sections.extend(bullet_lines(&self.current_state));
        sections.push(format!("- env: {environment}"));
        append_section(&mut sections, "주요 원인:", &self.reasons);
        append_section(&mut sections, "다음 행동:", &self.next_actions);
        append_section(&mut sections, "안전 상태:", &self.safety);
        sections.join("\n")
    }
}

pub async fn emit_research_report_alert_from_env(report: &ResearchRunReport) {
    let Some(config) = AlertConfig::from_env() else {
        return;
    };
    let Some(event) = research_report_alert_event(report, &config) else {
        return;
    };
    if let Err(error) = send_event(&config, &event).await {
        eprintln!("mattermost alert delivery failed: {error}");
    }
}

pub async fn emit_shadow_cycle_decision_alert_from_env(decision: &ShadowCycleDecision) {
    let Some(config) = AlertConfig::from_env() else {
        return;
    };
    let Some(event) = shadow_cycle_decision_alert_event(decision, &config) else {
        return;
    };
    if let Err(error) = send_event(&config, &event).await {
        eprintln!("mattermost alert delivery failed: {error}");
    }
}

fn research_report_alert_event(
    report: &ResearchRunReport,
    config: &AlertConfig,
) -> Option<AlertEvent> {
    let counts = bias_counts(report);
    let promote_paper_count = *counts
        .get(ResearchBias::PromoteToPaperBias.report_key())
        .unwrap_or(&0);
    let promote_shadow_count = *counts
        .get(ResearchBias::PromoteToShadowBias.report_key())
        .unwrap_or(&0);
    let retest_count = *counts
        .get(ResearchBias::RetestBias.report_key())
        .unwrap_or(&0);
    let prune_count = *counts
        .get(ResearchBias::PruneBias.report_key())
        .unwrap_or(&0);
    let paper_count = report.paper_trade_candidates.len();
    let paper_watch_count = report.paper_watch_candidates.len();
    let shadow_count = report.shadow_validation_runs.len();
    let max_total_notional_pct = report
        .portfolio_allocation_snapshot
        .as_ref()
        .map(|snapshot| snapshot.max_total_notional_pct)
        .unwrap_or_default();

    let (priority, title, conclusion) = if max_total_notional_pct > 0.0 {
        (
            AlertPriority::P0,
            "portfolio notional is non-zero".to_owned(),
            "portfolio allocation notional이 0보다 큽니다. live/order 경계가 의도대로 잠겨 있는지 확인해야 합니다."
                .to_owned(),
        )
    } else if paper_count > 0 || promote_paper_count > 0 {
        (
            AlertPriority::P1,
            "PROMOTE_TO_PAPER 후보 발생".to_owned(),
            "shadow를 통과한 paper 후보가 생성됐습니다. 아직 EXECUTION_APPROVED/LIVE_READY는 아닙니다."
                .to_owned(),
        )
    } else if paper_watch_count > 0 {
        (
            AlertPriority::P2,
            "PAPER_WATCH 후보 발생".to_owned(),
            "positive RETEST 후보가 돈을 쓰지 않는 forward paper-watch 관측 단계로 올라갔습니다."
                .to_owned(),
        )
    } else if shadow_count > 0 || promote_shadow_count > 0 {
        (
            AlertPriority::P2,
            "PROMOTE_TO_SHADOW 후보 발생".to_owned(),
            "후보가 shadow 관측 단계로 올라갔습니다. 주문 실행은 없고 forward observation만 시작됩니다."
                .to_owned(),
        )
    } else if config.include_retest_summary && retest_count > 0 {
        (
            AlertPriority::P3,
            "RETEST 블로커 요약".to_owned(),
            "후보가 아직 PROMOTE로 올라가지 못하고 RETEST 상태입니다.".to_owned(),
        )
    } else {
        return None;
    };

    if !config.allows(priority) {
        return None;
    }

    Some(AlertEvent {
        priority,
        title,
        conclusion,
        current_state: vec![
            format!("report_id: {}", report.research_run_report_id),
            format!("run_scope: {}", report.run_scope),
            format!("total candidates: {}", report.summary_findings.len()),
            format!("RETEST: {retest_count}"),
            format!("PRUNE: {prune_count}"),
            format!("PROMOTE_TO_SHADOW: {promote_shadow_count}"),
            format!("PROMOTE_TO_PAPER: {promote_paper_count}"),
            format!("shadow validation created: {shadow_count}"),
            format!("paper-watch candidates: {paper_watch_count}"),
            format!("paper candidates: {paper_count}"),
            format!("max_total_notional_pct: {max_total_notional_pct:.4}"),
        ],
        reasons: top_reason_lines(report, priority),
        next_actions: research_next_actions(priority),
        safety: vec![
            "order execution: disabled by research-app contract".to_owned(),
            "EXECUTION_APPROVED: never emitted by research-app".to_owned(),
            "LIVE_READY: never emitted by research-app".to_owned(),
        ],
    })
}

fn shadow_cycle_decision_alert_event(
    decision: &ShadowCycleDecision,
    config: &AlertConfig,
) -> Option<AlertEvent> {
    let priority = if decision.safety.live_enabled || decision.safety.order_execution_enabled {
        AlertPriority::P0
    } else if decision
        .scheduler_action
        .requires_focused_research_manifest()
    {
        AlertPriority::P2
    } else if config.include_shadow_wait && decision.scheduler_action.is_wait_action() {
        AlertPriority::P3
    } else {
        return None;
    };

    if !config.allows(priority) {
        return None;
    }

    let title = match decision.scheduler_action {
        ShadowCycleSchedulerAction::RunFocusedShadowSampleAccumulationResearch => {
            "shadow sample accumulation dispatch 준비".to_owned()
        }
        ShadowCycleSchedulerAction::WaitUntilTargetWindowMaterializes
        | ShadowCycleSchedulerAction::WaitUntilPendingShadowTargetWindowMaterializes => {
            "shadow holding window 대기".to_owned()
        }
        _ if priority == AlertPriority::P0 => "shadow cycle safety boundary changed".to_owned(),
        _ => "shadow cycle decision".to_owned(),
    };

    Some(AlertEvent {
        priority,
        title,
        conclusion: format!(
            "shadow cycle scheduler action은 {:?}입니다.",
            decision.scheduler_action
        ),
        current_state: vec![
            format!("decision_id: {}", decision.decision_id),
            format!("source_verdict: {}", decision.source_verdict),
            format!(
                "run_not_before: {}",
                decision
                    .run_not_before_at
                    .clone()
                    .unwrap_or_else(|| "none".to_owned())
            ),
            format!(
                "shadow_validation_count: {}",
                decision.shadow_sample_state.shadow_validation_count
            ),
            format!(
                "target_window_materialized_count: {}",
                decision
                    .shadow_sample_state
                    .target_window_materialized_count
            ),
            format!(
                "pending_target_window_candidate_count: {}",
                decision
                    .shadow_sample_state
                    .pending_target_window_candidate_count
            ),
            format!(
                "total_sample_deficit: {}",
                decision.shadow_sample_state.total_sample_deficit
            ),
        ],
        reasons: decision.blocked_actions.clone(),
        next_actions: decision.safe_next_actions.clone(),
        safety: vec![
            format!("s3_write: {}", decision.safety.s3_write),
            format!("ecs_task_started: {}", decision.safety.ecs_task_started),
            format!("paper_live_enabled: {}", decision.safety.paper_live_enabled),
            format!("live_enabled: {}", decision.safety.live_enabled),
            format!(
                "order_execution_enabled: {}",
                decision.safety.order_execution_enabled
            ),
        ],
    })
}

async fn send_event(config: &AlertConfig, event: &AlertEvent) -> Result<(), String> {
    if !config.webhook_url.starts_with("https://") && !config.webhook_url.starts_with("http://") {
        return Err("webhook URL must start with http:// or https://".to_owned());
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(|error| error.to_string())?;
    let response = client
        .post(&config.webhook_url)
        .json(&json!({ "text": event.text(&config.environment) }))
        .send()
        .await
        .map_err(|error| error.to_string())?;
    let status = response.status();
    if !status.is_success() {
        return Err(format!("webhook returned HTTP {status}"));
    }
    Ok(())
}

fn bias_counts(report: &ResearchRunReport) -> BTreeMap<&'static str, usize> {
    let mut counts = BTreeMap::new();
    for finding in &report.summary_findings {
        *counts.entry(finding.bias.report_key()).or_insert(0) += 1;
    }
    counts
}

fn top_reason_lines(report: &ResearchRunReport, priority: AlertPriority) -> Vec<String> {
    let mut reason_counts = BTreeMap::<String, usize>::new();
    for finding in &report.summary_findings {
        if priority == AlertPriority::P3 && finding.bias != ResearchBias::RetestBias {
            continue;
        }
        for reason in &finding.reason_codes {
            *reason_counts.entry(reason.clone()).or_default() += 1;
        }
    }
    let mut rows = reason_counts.into_iter().collect::<Vec<_>>();
    rows.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    rows.into_iter()
        .take(5)
        .map(|(reason, count)| format!("{reason}: {count}"))
        .collect()
}

fn research_next_actions(priority: AlertPriority) -> Vec<String> {
    match priority {
        AlertPriority::P0 => vec![
            "confirm live/order configuration immediately".to_owned(),
            "pause promotion workflow if this was not an intentional change".to_owned(),
        ],
        AlertPriority::P1 => vec![
            "review paper candidate contract before any execution boundary changes".to_owned(),
            "keep live/order execution disabled".to_owned(),
        ],
        AlertPriority::P2 => vec![
            "watch forward paper/shadow validation until completion evidence exists".to_owned(),
            "do not create live approval from pending observation state".to_owned(),
        ],
        AlertPriority::P3 => vec![
            "run focused retest or gap resolver for the listed blockers".to_owned(),
            "keep candidate in RETEST until sample, unseen, split, and liquidity evidence clear"
                .to_owned(),
        ],
    }
}

fn env_bool(name: &str) -> bool {
    env::var(name)
        .ok()
        .is_some_and(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
}

fn bullet_lines(values: &[String]) -> Vec<String> {
    values.iter().map(|value| format!("- {value}")).collect()
}

fn append_section(sections: &mut Vec<String>, title: &str, values: &[String]) {
    if values.is_empty() {
        return;
    }
    sections.push(String::new());
    sections.push(title.to_owned());
    sections.extend(bullet_lines(values));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        DEFAULT_RESEARCH_GATE_POLICY_VERSION, HypothesisOutput, PortfolioAllocationSnapshot,
        ResearchGatePolicy, ResearchRunStatus, ShadowCycleDecisionSafety, ShadowCycleSampleState,
        SummaryFinding,
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
        let event = research_report_alert_event(&report, &config).expect("event is created");
        let text = event.text("dev");

        assert!(text.contains("[P3][research-app] RETEST"));
        assert!(text.contains("결론:"));
        assert!(text.contains("현재 상태:"));
        assert!(text.contains("주요 원인:"));
        assert!(text.contains("다음 행동:"));
        assert!(text.contains("안전 상태:"));
        assert!(text.contains("promotion_sample_count_below_minimum: 1"));
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
        let event = research_report_alert_event(&report, &config).expect("event is created");

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
        report.paper_watch_candidates = vec!["paper_watch_candidate_001".to_owned()];
        let event = research_report_alert_event(&report, &test_config(AlertPriority::P2))
            .expect("paper watch event is created");
        let text = event.text("dev");

        assert_eq!(event.priority, AlertPriority::P2);
        assert!(text.contains("PAPER_WATCH"));
        assert!(text.contains("paper-watch candidates: 1"));
        assert!(text.contains("order execution: disabled by research-app contract"));
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

        assert!(research_report_alert_event(&report, &test_config(AlertPriority::P3)).is_none());
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

        assert!(research_report_alert_event(&report, &config).is_none());
    }

    #[test]
    fn promote_to_paper_alert_has_p1_priority() {
        let report = test_report(vec![SummaryFinding {
            candidate_id: "cand_001".to_owned(),
            candidate_lifecycle_key: "life_001".to_owned(),
            bias: ResearchBias::PromoteToPaperBias,
            reason_codes: vec!["shadow_validation_passed".to_owned()],
        }]);
        let event = research_report_alert_event(&report, &test_config(AlertPriority::P1))
            .expect("paper event is created");

        assert_eq!(event.priority, AlertPriority::P1);
        assert!(
            event
                .text("prod")
                .contains("keep live/order execution disabled")
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

        let event = research_report_alert_event(&report, &test_config(AlertPriority::P0))
            .expect("portfolio event is created");

        assert_eq!(event.priority, AlertPriority::P0);
        assert!(event.text("dev").contains("max_total_notional_pct: 0.1000"));
    }

    #[test]
    fn reason_summary_limits_to_top_five_reasons() {
        let report = test_report(vec![
            finding("cand_001", ResearchBias::PromoteToShadowBias, &["c", "a"]),
            finding("cand_002", ResearchBias::PromoteToShadowBias, &["c", "b"]),
            finding("cand_003", ResearchBias::PromoteToShadowBias, &["c", "b"]),
            finding("cand_004", ResearchBias::PromoteToShadowBias, &["d"]),
            finding("cand_005", ResearchBias::PromoteToShadowBias, &["e"]),
            finding("cand_006", ResearchBias::PromoteToShadowBias, &["f"]),
        ]);

        let reasons = top_reason_lines(&report, AlertPriority::P2);

        assert_eq!(reasons.len(), 5);
        assert_eq!(reasons[0], "c: 3");
        assert!(reasons.contains(&"b: 2".to_owned()));
        assert!(!reasons.contains(&"f: 1".to_owned()));
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

    #[tokio::test]
    async fn send_event_rejects_invalid_webhook_url_before_http_call() {
        let config = AlertConfig {
            webhook_url: "not-a-url".to_owned(),
            environment: "dev".to_owned(),
            min_priority: AlertPriority::P0,
            include_retest_summary: false,
            include_shadow_wait: false,
        };
        let event = AlertEvent {
            priority: AlertPriority::P0,
            title: "test".to_owned(),
            conclusion: "test".to_owned(),
            current_state: Vec::new(),
            reasons: Vec::new(),
            next_actions: Vec::new(),
            safety: Vec::new(),
        };

        let error = send_event(&config, &event)
            .await
            .expect_err("URL is rejected");
        assert_eq!(error, "webhook URL must start with http:// or https://");
    }

    fn test_config(min_priority: AlertPriority) -> AlertConfig {
        AlertConfig {
            webhook_url: "https://example.com/hook".to_owned(),
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
}

use crate::model::{
    PaperWatchCandidate, PaperWatchLiveMark, ResearchBias, ResearchRunReport, ShadowCycleDecision,
    ShadowCycleSchedulerAction,
};
use crate::{hash::stable_id, time::now_ms};
use aws_config::BehaviorVersion;
use aws_sdk_s3::{Client, primitives::ByteStream};
use aws_types::region::Region;
use chrono::{DateTime, Datelike, Timelike, Utc};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::env;

const APP_NAME: &str = "research-app";
const DEFAULT_ENVIRONMENT: &str = "dev";
const DEFAULT_PIPELINE_ALERT_S3_PREFIX: &str =
    "pipeline-alert-event/schema=pipeline_alert_event_v1";

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
    event_bucket: String,
    event_prefix: String,
    environment: String,
    min_priority: AlertPriority,
    include_retest_summary: bool,
    include_shadow_wait: bool,
}

impl AlertConfig {
    fn from_env() -> Option<Self> {
        let event_bucket = env::var("NANGMAN_PIPELINE_ALERT_S3_BUCKET")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .or_else(|| {
                env::var("RESEARCH_OUTPUT_S3_BUCKET")
                    .ok()
                    .filter(|value| !value.trim().is_empty())
            })?;
        let event_prefix = env::var("NANGMAN_PIPELINE_ALERT_S3_PREFIX")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_PIPELINE_ALERT_S3_PREFIX.to_owned());
        let environment =
            env::var("NANGMAN_ALERT_ENV").unwrap_or_else(|_| DEFAULT_ENVIRONMENT.to_owned());
        let min_priority = env::var("NANGMAN_ALERT_MIN_PRIORITY")
            .ok()
            .and_then(|value| AlertPriority::parse(&value))
            .unwrap_or(AlertPriority::P2);
        let include_retest_summary = env_bool("NANGMAN_ALERT_INCLUDE_RETEST_SUMMARY");
        let include_shadow_wait = env_bool("NANGMAN_ALERT_INCLUDE_SHADOW_WAIT");

        Some(Self {
            event_bucket,
            event_prefix,
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
    #[cfg(test)]
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

pub async fn emit_research_report_alert_from_env(
    report: &ResearchRunReport,
    paper_watch_candidates: &[PaperWatchCandidate],
) {
    let Some(config) = AlertConfig::from_env() else {
        return;
    };
    let Some(event) = research_report_alert_event(report, paper_watch_candidates, &config) else {
        return;
    };
    if let Err(error) = send_event(&config, &event).await {
        eprintln!("mattermost alert delivery failed: {error}");
    }
}

pub async fn emit_paper_watch_live_mark_alert_from_env(marks: &[PaperWatchLiveMark]) {
    let Some(config) = AlertConfig::from_env() else {
        return;
    };
    let Some(event) = paper_watch_live_mark_alert_event(marks, &config) else {
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
    paper_watch_candidates: &[PaperWatchCandidate],
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
    let paper_watch_count = report
        .paper_watch_candidates
        .len()
        .max(paper_watch_candidates.len());
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
            format!("모의 관찰 후보 {paper_watch_count}개 발생"),
            "실제 거래가 아니라, 수익 가능성이 보인 RETEST 후보를 돈 안 쓰는 실시간 모의 관찰 단계로 올렸습니다."
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

    let mut current_state = vec![
        format!("report_id: {}", report.research_run_report_id),
        format!("실행 범위: {}", report.run_scope),
        format!("전체 후보: {}개", report.summary_findings.len()),
        format!(
            "판정: RETEST {retest_count} / PRUNE {prune_count} / SHADOW 승급 {promote_shadow_count} / PAPER 승급 {promote_paper_count}"
        ),
        format!("shadow 관찰 생성: {shadow_count}개"),
        format!("모의 관찰 후보: {paper_watch_count}개"),
        format!("paper 후보: {paper_count}개"),
        format!("실제 투자 비중: {max_total_notional_pct:.4}"),
    ];
    current_state.extend(paper_watch_candidate_summary_lines(paper_watch_candidates));

    Some(AlertEvent {
        priority,
        title,
        conclusion,
        current_state,
        reasons: top_reason_lines(report, priority),
        next_actions: research_next_actions(priority),
        safety: vec![
            "실제 주문: 꺼짐".to_owned(),
            "실제 돈 사용: 없음".to_owned(),
            "EXECUTION_APPROVED/LIVE_READY: research-app에서 발행하지 않음".to_owned(),
        ],
    })
}

fn paper_watch_live_mark_alert_event(
    marks: &[PaperWatchLiveMark],
    config: &AlertConfig,
) -> Option<AlertEvent> {
    if marks.is_empty() {
        return None;
    }
    let unsafe_mark = marks.iter().any(|mark| {
        !mark.safety.paper_only
            || mark.safety.live_enabled
            || mark.safety.order_execution_enabled
            || mark.safety.execution_approval_emitted
    });
    if !unsafe_mark {
        return None;
    }
    let priority = AlertPriority::P0;
    if !config.allows(priority) {
        return None;
    }

    let symbols = unique_join(marks.iter().map(|mark| mark.symbol_canonical.as_str()));
    let venue_counts = count_join(marks.iter().map(|mark| mark.venue.as_str()));
    let state_counts = count_join(marks.iter().map(|mark| mark.lifecycle_state.as_str()));
    let net_returns = marks
        .iter()
        .map(|mark| mark.net_return_bps)
        .collect::<Vec<_>>();
    let min_net = net_returns.iter().copied().fold(f64::INFINITY, f64::min);
    let max_net = net_returns
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    let avg_net = net_returns.iter().sum::<f64>() / net_returns.len() as f64;

    Some(AlertEvent {
        priority,
        title: format!("paper-watch live safety boundary changed: {} marks", marks.len()),
        conclusion: "paper-watch live mark 안에서 paper-only 안전 경계가 깨진 항목이 감지됐습니다. 실제 주문 설정이 열린 것인지 즉시 확인해야 합니다."
            .to_owned(),
        current_state: vec![
            format!("관찰 코인: {symbols}"),
            format!("생성된 live mark: {}개", marks.len()),
            format!("거래소별 mark: {venue_counts}"),
            format!("후보 상태: {state_counts}"),
            format!("모의 수익률 범위: {min_net:.2} ~ {max_net:.2} bps"),
            format!("모의 평균 수익률: {avg_net:.2} bps"),
        ],
        reasons: Vec::new(),
        next_actions: vec![
            "live/order 설정이 의도적으로 열린 것인지 즉시 확인합니다.".to_owned(),
            "의도한 변경이 아니면 paper-watch observer와 downstream 승급 흐름을 멈춥니다.".to_owned(),
        ],
        safety: vec![
            "실제 주문: 꺼짐".to_owned(),
            "실제 돈 사용: 없음".to_owned(),
            "paper-watch live mark만 기록".to_owned(),
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
    let delivery = build_pipeline_alert_delivery(config, event, now_ms())?;
    s3_client()
        .await?
        .put_object()
        .bucket(&config.event_bucket)
        .key(delivery.key)
        .content_type("application/json")
        .body(ByteStream::from(delivery.body))
        .send()
        .await
        .map_err(|error| error.to_string())?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PipelineAlertDelivery {
    key: String,
    body: Vec<u8>,
}

fn build_pipeline_alert_delivery(
    config: &AlertConfig,
    event: &AlertEvent,
    created_at_ms: i64,
) -> Result<PipelineAlertDelivery, String> {
    let priority = event.priority.as_str();
    let event_id = stable_id(
        "pipeline_alert",
        &[APP_NAME, priority, &event.title, &created_at_ms.to_string()],
    );
    let dedupe_key = stable_id(
        "pipeline_alert_dedupe",
        &[
            APP_NAME,
            priority,
            &event.title,
            &event.conclusion,
            &event.current_state.join("\n"),
            &event.reasons.join("\n"),
        ],
    );
    let payload = PipelineAlertEvent::from_alert_event(
        event,
        &event_id,
        &dedupe_key,
        &config.environment,
        created_at_ms,
    );
    let key = pipeline_alert_event_key(
        &config.event_prefix,
        created_at_ms,
        APP_NAME,
        priority,
        &event_id,
    )?;
    let body = serde_json::to_vec_pretty(&payload).map_err(|error| error.to_string())?;
    Ok(PipelineAlertDelivery { key, body })
}

#[derive(Debug, Serialize)]
struct PipelineAlertEvent<'a> {
    schema_version: &'static str,
    event_id: &'a str,
    dedupe_key: &'a str,
    app: &'static str,
    environment: &'a str,
    priority: &'static str,
    title: &'a str,
    conclusion: &'a str,
    current_state: &'a [String],
    reasons: &'a [String],
    next_actions: &'a [String],
    safety: &'a [String],
    created_at_ms: i64,
}

impl<'a> PipelineAlertEvent<'a> {
    fn from_alert_event(
        event: &'a AlertEvent,
        event_id: &'a str,
        dedupe_key: &'a str,
        environment: &'a str,
        created_at_ms: i64,
    ) -> Self {
        Self {
            schema_version: "pipeline_alert_event_v1",
            event_id,
            dedupe_key,
            app: APP_NAME,
            environment,
            priority: event.priority.as_str(),
            title: &event.title,
            conclusion: &event.conclusion,
            current_state: &event.current_state,
            reasons: &event.reasons,
            next_actions: &event.next_actions,
            safety: &event.safety,
            created_at_ms,
        }
    }
}

async fn s3_client() -> Result<Client, String> {
    let mut loader = aws_config::defaults(BehaviorVersion::latest());
    if let Some(region) = env_string("AWS_REGION").or_else(|| env_string("AWS_DEFAULT_REGION")) {
        loader = loader.region(Region::new(region));
    }
    let config = loader.load().await;
    Ok(Client::new(&config))
}

fn pipeline_alert_event_key(
    prefix: &str,
    created_at_ms: i64,
    app: &str,
    priority: &str,
    event_id: &str,
) -> Result<String, String> {
    let created_at = DateTime::<Utc>::from_timestamp_millis(created_at_ms)
        .ok_or_else(|| "created_at_ms is outside supported timestamp range".to_owned())?;
    Ok(format!(
        "{}/dt={:04}-{:02}-{:02}/hour={:02}/app={}/priority={}/{}.json",
        prefix.trim().trim_matches('/'),
        created_at.year(),
        created_at.month(),
        created_at.day(),
        created_at.hour(),
        s3_key_token(app),
        s3_key_token(priority),
        s3_key_token(event_id),
    ))
}

fn s3_key_token(value: &str) -> String {
    value
        .chars()
        .map(|character| match character {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '=' => character,
            _ => '_',
        })
        .collect()
}

fn env_string(name: &str) -> Option<String> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
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
        .map(|(reason, count)| format!("{}: {}개", human_reason(&reason), count))
        .collect()
}

fn research_next_actions(priority: AlertPriority) -> Vec<String> {
    match priority {
        AlertPriority::P0 => vec![
            "live/order 설정이 의도적으로 열린 것인지 즉시 확인합니다.".to_owned(),
            "의도한 변경이 아니면 승급 흐름을 멈춥니다.".to_owned(),
        ],
        AlertPriority::P1 => vec![
            "paper 후보 계약을 먼저 검토하고, 실행 경계 변경은 별도로 승인합니다.".to_owned(),
            "실제 주문과 live trading은 계속 꺼둡니다.".to_owned(),
        ],
        AlertPriority::P2 => vec![
            "실제 주문 없이 paper-watch live mark를 계속 누적합니다.".to_owned(),
            "관찰이 끝나기 전에는 live 승인으로 올리지 않습니다.".to_owned(),
        ],
        AlertPriority::P3 => vec![
            "부족한 샘플과 시장 데이터 구간을 focused retest로 채웁니다.".to_owned(),
            "샘플, unseen, train/validation, liquidity 증거가 풀릴 때까지 RETEST로 둡니다."
                .to_owned(),
        ],
    }
}

fn paper_watch_candidate_summary_lines(candidates: &[PaperWatchCandidate]) -> Vec<String> {
    if candidates.is_empty() {
        return Vec::new();
    }
    let symbols = unique_join(
        candidates
            .iter()
            .map(|candidate| candidate.symbol_canonical.as_str()),
    );
    let completed = candidates
        .iter()
        .map(|candidate| candidate.replay_sample_summary.completed_count)
        .sum::<usize>();
    let replay_runs = candidates
        .iter()
        .map(|candidate| candidate.replay_sample_summary.replay_run_count)
        .sum::<usize>();
    let positive = candidates
        .iter()
        .map(|candidate| candidate.replay_sample_summary.positive_net_count)
        .sum::<usize>();
    let non_positive = candidates
        .iter()
        .map(|candidate| candidate.replay_sample_summary.non_positive_net_count)
        .sum::<usize>();
    let net_values = candidates
        .iter()
        .filter_map(|candidate| {
            candidate
                .replay_sample_summary
                .weighted_mean_net_after_cost_bps
        })
        .collect::<Vec<_>>();
    let mut lines = vec![
        format!("관찰 코인: {symbols}"),
        format!("완료 replay 샘플: {completed}/{replay_runs}"),
        format!("positive/non-positive 샘플: {positive}/{non_positive}"),
    ];
    if !net_values.is_empty() {
        let min_net = net_values.iter().copied().fold(f64::INFINITY, f64::min);
        let max_net = net_values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        lines.push(format!(
            "과거 검증 net_after_cost 범위: {min_net:.2} ~ {max_net:.2} bps"
        ));
    }
    lines
}

fn unique_join<'a>(values: impl Iterator<Item = &'a str>) -> String {
    let values = values
        .filter_map(|value| {
            let trimmed = value.trim();
            (!trimmed.is_empty()).then(|| trimmed.to_owned())
        })
        .collect::<BTreeSet<_>>();
    if values.is_empty() {
        "없음".to_owned()
    } else {
        values.into_iter().collect::<Vec<_>>().join(", ")
    }
}

fn count_join<'a>(values: impl Iterator<Item = &'a str>) -> String {
    let mut counts = BTreeMap::<String, usize>::new();
    for value in values {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            *counts.entry(trimmed.to_owned()).or_default() += 1;
        }
    }
    if counts.is_empty() {
        return "없음".to_owned();
    }
    counts
        .into_iter()
        .map(|(value, count)| format!("{value} {count}개"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn human_reason(reason: &str) -> &'static str {
    match reason {
        "native_replay_horizon_not_materialized" => "목표 보유시간만큼의 시장 데이터가 아직 부족함",
        "native_replay_positive_but_promotion_blocked" => {
            "과거 검증은 긍정적이지만 승급 조건이 아직 부족함"
        }
        "native_replay_positive_but_survival_not_proven" => {
            "시간이 지나도 신호가 유지되는지 아직 증명되지 않음"
        }
        "needs_unseen_window_validation" | "unseen_window_validation_not_proven" => {
            "아직 보지 않은 구간 검증이 부족함"
        }
        "no_completed_native_replay_samples" => "완료된 replay 샘플이 아직 없음",
        "promotion_sample_count_below_minimum" => "승급에 필요한 샘플 수가 부족함",
        "liquidity_filter_not_materialized" => "유동성 검증 데이터가 아직 준비되지 않음",
        "liquidity_filter_no_positive_volume_observed" => {
            "거래량 검증에서 충분한 유동성이 확인되지 않음"
        }
        "deterministic_shadow_gate_passed" => "shadow 관찰로 올릴 조건을 통과함",
        "shadow_validation_passed" => "shadow 검증을 통과함",
        "portfolio_notional_non_zero" => "실제 투자 비중이 0보다 큼",
        _ => "기타 검증 조건 미충족",
    }
}

fn env_bool(name: &str) -> bool {
    env::var(name)
        .ok()
        .is_some_and(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
}

#[cfg(test)]
fn bullet_lines(values: &[String]) -> Vec<String> {
    values.iter().map(|value| format!("- {value}")).collect()
}

#[cfg(test)]
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
        DEFAULT_RESEARCH_GATE_POLICY_VERSION, HypothesisOutput, PaperExpectedCostProfile,
        PaperExpectedRiskProfile, PaperWatchReplaySampleSummary, PaperWatchSafety,
        PortfolioAllocationSnapshot, ResearchGatePolicy, ResearchRunStatus,
        ShadowCycleDecisionSafety, ShadowCycleSampleState, SummaryFinding, SurvivalBand,
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
        let event =
            research_report_alert_event(&report, &candidates, &test_config(AlertPriority::P2))
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

        assert!(
            research_report_alert_event(&report, &[], &test_config(AlertPriority::P3)).is_none()
        );
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

        assert!(
            paper_watch_live_mark_alert_event(&marks, &test_config(AlertPriority::P2)).is_none()
        );
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

        let delivery = build_pipeline_alert_delivery(&config, &event, 1779937200123)
            .expect("delivery is built");
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
}

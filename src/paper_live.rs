use crate::error::{AppError, AppResult};
use crate::hash::stable_id;
use crate::model::{
    MARKET_LIVE_TICK_SCHEMA_VERSION, MarketLiveTick, PAPER_WATCH_LIVE_MARK_SCHEMA_VERSION,
    PaperWatchCandidate, PaperWatchLiveMark, PaperWatchSafety,
};
use async_nats::jetstream;
use async_nats::jetstream::consumer::PullConsumer;
use async_nats::jetstream::consumer::{AckPolicy, DeliverPolicy};
use futures_util::StreamExt;
use serde::de::DeserializeOwned;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::time::Duration;

pub const DEFAULT_MARKET_LIVE_NATS_STREAM: &str = "MARKET_LIVE";
pub const DEFAULT_MARKET_LIVE_NATS_SUBJECT: &str = "market_live_tick.created.>";
pub const DEFAULT_MARKET_LIVE_NATS_CONSUMER: &str = "research-paper-watch-live";
pub const DEFAULT_MARKET_LIVE_NATS_DELIVER_POLICY: &str = "last_per_subject";
pub const DEFAULT_MARKET_LIVE_NATS_BATCH_SIZE: usize = 100;
pub const DEFAULT_MARKET_LIVE_NATS_MAX_MESSAGES: usize = 500;
pub const DEFAULT_MARKET_LIVE_NATS_ACK_WAIT_SECS: u64 = 30;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarketLiveNatsConfig {
    pub url: String,
    pub stream: String,
    pub subject: String,
    pub consumer: String,
    pub deliver_policy: String,
    pub batch_size: usize,
    pub max_messages: usize,
    pub ack_wait_secs: u64,
}

pub fn read_paper_watch_candidates(path: &Path) -> AppResult<Vec<PaperWatchCandidate>> {
    read_json_array_or_jsonl(path)
}

pub fn read_market_live_ticks(path: &Path) -> AppResult<Vec<MarketLiveTick>> {
    read_json_array_or_jsonl(path)
}

pub async fn read_market_live_ticks_from_nats(
    config: &MarketLiveNatsConfig,
) -> AppResult<Vec<MarketLiveTick>> {
    validate_nats_config(config)?;
    let client = async_nats::connect(&config.url)
        .await
        .map_err(|error| AppError::nats(format!("connect {}: {error}", config.url)))?;
    let jetstream = jetstream::new(client);
    let stream = jetstream
        .get_stream(&config.stream)
        .await
        .map_err(|error| AppError::nats(format!("get stream {}: {error}", config.stream)))?;
    let consumer = stream
        .get_or_create_consumer(
            &config.consumer,
            jetstream::consumer::pull::Config {
                durable_name: Some(config.consumer.clone()),
                filter_subject: config.subject.clone(),
                ack_policy: AckPolicy::Explicit,
                ack_wait: Duration::from_secs(config.ack_wait_secs),
                max_ack_pending: config.batch_size as i64,
                deliver_policy: deliver_policy(&config.deliver_policy)?,
                ..Default::default()
            },
        )
        .await
        .map_err(|error| {
            AppError::nats(format!(
                "get/create consumer {} on stream {}: {error}",
                config.consumer, config.stream
            ))
        })?;

    read_ticks_from_consumer(consumer, config).await
}

pub fn build_paper_watch_live_marks(
    candidates: &[PaperWatchCandidate],
    ticks: &[MarketLiveTick],
) -> Vec<PaperWatchLiveMark> {
    let mut candidates_by_symbol: BTreeMap<String, Vec<&PaperWatchCandidate>> = BTreeMap::new();
    for candidate in candidates {
        if !candidate.safety.paper_only
            || candidate.safety.live_enabled
            || candidate.safety.order_execution_enabled
            || candidate.safety.execution_approval_emitted
        {
            continue;
        }
        candidates_by_symbol
            .entry(normalize_symbol(&candidate.symbol_canonical))
            .or_default()
            .push(candidate);
    }

    let mut ordered_ticks = ticks.to_vec();
    ordered_ticks.sort_by(|left, right| {
        (
            left.exchange_timestamp_ms,
            left.ingest_timestamp_ms,
            left.event_id.as_str(),
        )
            .cmp(&(
                right.exchange_timestamp_ms,
                right.ingest_timestamp_ms,
                right.event_id.as_str(),
            ))
    });

    let mut entry_by_watch_candidate = BTreeMap::<String, f64>::new();
    let mut marks = Vec::new();
    for tick in &ordered_ticks {
        if tick.schema_version != MARKET_LIVE_TICK_SCHEMA_VERSION {
            continue;
        }
        let Some(current_price) = valid_mark_price(tick.mark_price) else {
            continue;
        };
        let Some(matched_candidates) =
            candidates_by_symbol.get(&normalize_symbol(&tick.symbol_canonical))
        else {
            continue;
        };
        for candidate in matched_candidates {
            let entry_price = *entry_by_watch_candidate
                .entry(candidate.paper_watch_candidate_id.clone())
                .or_insert(current_price);
            marks.push(build_mark(candidate, tick, entry_price, current_price));
        }
    }
    marks
}

fn build_mark(
    candidate: &PaperWatchCandidate,
    tick: &MarketLiveTick,
    entry_price: f64,
    current_price: f64,
) -> PaperWatchLiveMark {
    let holding_elapsed_ms = tick
        .exchange_timestamp_ms
        .saturating_sub(candidate.created_at_ms)
        .max(0);
    let lifecycle_state = lifecycle_state(candidate, holding_elapsed_ms);
    let net_return_bps = ((current_price / entry_price) - 1.0) * 10_000.0;
    let mut reason_codes = vec![
        "paper_watch_live_mark".to_owned(),
        "paper_only_no_order_execution".to_owned(),
        format!("price_source={}", tick.price_source),
    ];
    if lifecycle_state != "watching" {
        reason_codes.push(lifecycle_state.clone());
    }

    PaperWatchLiveMark {
        paper_watch_live_mark_id: stable_id(
            "paper_watch_live_mark",
            &[&candidate.paper_watch_candidate_id, &tick.event_id],
        ),
        paper_watch_candidate_id: candidate.paper_watch_candidate_id.clone(),
        candidate_id: candidate.candidate_id.clone(),
        candidate_lifecycle_key: candidate.candidate_lifecycle_key.clone(),
        symbol_canonical: candidate.symbol_canonical.clone(),
        source_research_run_id: candidate.source_research_run_id.clone(),
        source_market_live_event_id: tick.event_id.clone(),
        venue: tick.venue.clone(),
        mark_source: "market_live_tick".to_owned(),
        marked_at_ms: tick.ingest_timestamp_ms.max(tick.exchange_timestamp_ms),
        exchange_timestamp_ms: tick.exchange_timestamp_ms,
        ingest_timestamp_ms: tick.ingest_timestamp_ms,
        holding_elapsed_ms,
        entry_mark_price: entry_price,
        current_mark_price: current_price,
        net_return_bps,
        target_max_holding_hours: candidate.target_max_holding_hours,
        absolute_max_holding_hours: candidate.absolute_max_holding_hours,
        lifecycle_state,
        reason_codes,
        safety: PaperWatchSafety {
            paper_only: true,
            live_enabled: false,
            order_execution_enabled: false,
            execution_approval_emitted: false,
        },
        schema_version: PAPER_WATCH_LIVE_MARK_SCHEMA_VERSION.to_owned(),
    }
}

async fn read_ticks_from_consumer(
    consumer: PullConsumer,
    config: &MarketLiveNatsConfig,
) -> AppResult<Vec<MarketLiveTick>> {
    let mut ticks = Vec::new();
    while ticks.len() < config.max_messages {
        let remaining = config.max_messages - ticks.len();
        let batch_size = config.batch_size.min(remaining).max(1);
        let mut messages = consumer
            .fetch()
            .max_messages(batch_size)
            .expires(Duration::from_secs(5))
            .messages()
            .await
            .map_err(|error| AppError::nats(format!("fetch market live messages: {error}")))?;
        let Some(message) = messages.next().await else {
            break;
        };
        let message = message
            .map_err(|error| AppError::nats(format!("read market live message: {error}")))?;
        let tick: MarketLiveTick = serde_json::from_slice(&message.payload)?;
        validate_tick(&tick)?;
        message
            .double_ack()
            .await
            .map_err(|error| AppError::nats(format!("market live double ack failed: {error}")))?;
        ticks.push(tick);
    }
    Ok(ticks)
}

fn validate_nats_config(config: &MarketLiveNatsConfig) -> AppResult<()> {
    if !config.url.starts_with("nats://") {
        return Err(AppError::config(
            "market live NATS url must start with nats://",
        ));
    }
    if config.stream.trim().is_empty() {
        return Err(AppError::config(
            "market live NATS stream must not be empty",
        ));
    }
    if config.subject.trim().is_empty() {
        return Err(AppError::config(
            "market live NATS subject must not be empty",
        ));
    }
    if config.consumer.trim().is_empty() {
        return Err(AppError::config(
            "market live NATS consumer must not be empty",
        ));
    }
    if config.batch_size == 0 || config.max_messages == 0 || config.ack_wait_secs == 0 {
        return Err(AppError::config(
            "market live NATS batch size, max messages, and ack wait must be positive",
        ));
    }
    deliver_policy(&config.deliver_policy)?;
    Ok(())
}

fn validate_tick(tick: &MarketLiveTick) -> AppResult<()> {
    if tick.schema_version != MARKET_LIVE_TICK_SCHEMA_VERSION {
        return Err(AppError::validation(format!(
            "market live tick schema_version must be {MARKET_LIVE_TICK_SCHEMA_VERSION}; got {}",
            tick.schema_version
        )));
    }
    if tick.event_id.trim().is_empty() {
        return Err(AppError::validation(
            "market live tick event_id is required",
        ));
    }
    if tick.symbol_canonical.trim().is_empty() {
        return Err(AppError::validation(
            "market live tick symbol_canonical is required",
        ));
    }
    if valid_mark_price(tick.mark_price).is_none() {
        return Err(AppError::validation(
            "market live tick mark_price must be positive and finite",
        ));
    }
    Ok(())
}

fn deliver_policy(value: &str) -> AppResult<DeliverPolicy> {
    match value {
        "all" => Ok(DeliverPolicy::All),
        "new" => Ok(DeliverPolicy::New),
        "last" => Ok(DeliverPolicy::Last),
        "last_per_subject" => Ok(DeliverPolicy::LastPerSubject),
        other => Err(AppError::config(format!(
            "unsupported market live deliver policy: {other}"
        ))),
    }
}

fn lifecycle_state(candidate: &PaperWatchCandidate, holding_elapsed_ms: i64) -> String {
    let absolute_ms = i64::from(candidate.absolute_max_holding_hours) * 60 * 60 * 1000;
    let target_ms = i64::from(candidate.target_max_holding_hours) * 60 * 60 * 1000;
    if holding_elapsed_ms >= absolute_ms {
        "force_flat_due".to_owned()
    } else if holding_elapsed_ms >= target_ms {
        "target_holding_window_open".to_owned()
    } else {
        "watching".to_owned()
    }
}

fn valid_mark_price(value: Option<f64>) -> Option<f64> {
    value.filter(|price| price.is_finite() && *price > 0.0)
}

fn normalize_symbol(value: &str) -> String {
    value.trim().to_ascii_uppercase()
}

fn read_json_array_or_jsonl<T>(path: &Path) -> AppResult<Vec<T>>
where
    T: DeserializeOwned,
{
    let bytes = fs::read(path)?;
    read_json_array_or_jsonl_bytes(&path.display().to_string(), &bytes)
}

fn read_json_array_or_jsonl_bytes<T>(label: &str, bytes: &[u8]) -> AppResult<Vec<T>>
where
    T: DeserializeOwned,
{
    let text =
        std::str::from_utf8(bytes).map_err(|error| AppError::Json(format!("{label}: {error}")))?;
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(AppError::validation(format!("{label} must not be empty")));
    }
    if trimmed.starts_with('[') {
        return Ok(serde_json::from_str(trimmed)?);
    }
    if trimmed.starts_with('{')
        && let Ok(value) = serde_json::from_str(trimmed)
    {
        return Ok(vec![value]);
    }

    let mut values = Vec::new();
    for (index, line) in trimmed.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        values.push(serde_json::from_str(line).map_err(|error| {
            AppError::Json(format!(
                "{label} line {} is not valid JSON: {error}",
                index + 1
            ))
        })?);
    }
    Ok(values)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        PaperExpectedCostProfile, PaperExpectedRiskProfile, PaperWatchReplaySampleSummary,
        ResearchBias, SurvivalBand,
    };

    #[test]
    fn live_marks_match_only_paper_watch_symbols() {
        let candidates = vec![candidate("watch_1", "SUI"), candidate("watch_2", "TON")];
        let ticks = vec![
            tick("tick_sui_1", "SUI", 1_000, 1.0),
            tick("tick_eth_1", "ETH", 1_100, 10.0),
            tick("tick_sui_2", "SUI", 1_200, 1.02),
        ];

        let marks = build_paper_watch_live_marks(&candidates, &ticks);

        assert_eq!(marks.len(), 2);
        assert_eq!(marks[0].paper_watch_candidate_id, "watch_1");
        assert_eq!(marks[0].net_return_bps, 0.0);
        assert!((marks[1].net_return_bps - 200.0).abs() < 0.0001);
        assert!(marks[1].safety.paper_only);
        assert!(!marks[1].safety.live_enabled);
        assert!(!marks[1].safety.order_execution_enabled);
        assert!(!marks[1].safety.execution_approval_emitted);
    }

    #[test]
    fn absolute_window_marks_force_flat_due_without_order_approval() {
        let mut candidate = candidate("watch_1", "SUI");
        candidate.created_at_ms = 0;
        candidate.target_max_holding_hours = 1;
        candidate.absolute_max_holding_hours = 2;
        let ticks = vec![tick("tick_sui_1", "SUI", 3 * 60 * 60 * 1000, 1.0)];

        let marks = build_paper_watch_live_marks(&[candidate], &ticks);

        assert_eq!(marks[0].lifecycle_state, "force_flat_due");
        assert!(marks[0].reason_codes.contains(&"force_flat_due".to_owned()));
        assert!(!marks[0].safety.order_execution_enabled);
    }

    fn candidate(id: &str, symbol: &str) -> PaperWatchCandidate {
        PaperWatchCandidate {
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

    fn tick(id: &str, symbol: &str, timestamp_ms: i64, mark_price: f64) -> MarketLiveTick {
        MarketLiveTick {
            schema_version: MARKET_LIVE_TICK_SCHEMA_VERSION.to_owned(),
            event_id: id.to_owned(),
            producer_run_id: "market_run_001".to_owned(),
            venue: "binance".to_owned(),
            source_role: "reference".to_owned(),
            market_type: "spot".to_owned(),
            event_type: "trade".to_owned(),
            symbol_native: format!("{symbol}USDT"),
            symbol_canonical: symbol.to_owned(),
            base_asset: symbol.to_owned(),
            quote_asset: "USDT".to_owned(),
            exchange_timestamp_ms: timestamp_ms,
            ingest_timestamp_ms: timestamp_ms + 10,
            latency_ms: 10,
            sequence_id: id.to_owned(),
            sequence_tag: "trade_id".to_owned(),
            price_source: "last_price".to_owned(),
            last_price: Some(mark_price),
            best_bid_price: None,
            best_ask_price: None,
            mark_price: Some(mark_price),
            quantity: Some(1.0),
            raw_payload_sha256: "sha256:test".to_owned(),
        }
    }
}

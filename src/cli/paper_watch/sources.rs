use super::super::*;

pub(super) async fn load_paper_watch_candidates(
    args: &Args,
) -> AppResult<Vec<crate::model::PaperWatchCandidate>> {
    match (
        args.paper_watch_candidate_file.as_deref(),
        args.paper_watch_candidate_s3_bucket.as_deref(),
        args.paper_watch_candidate_s3_key.as_deref(),
    ) {
        (Some(path), None, None) => read_paper_watch_candidates(path),
        (None, Some(bucket), Some(key)) => read_paper_watch_candidates_from_s3(bucket, key).await,
        _ => Err(AppError::config(
            "provide either --paper-watch-candidate-file or --paper-watch-candidate-s3-bucket/--paper-watch-candidate-s3-key",
        )),
    }
}

pub(super) async fn load_market_live_ticks(
    args: &Args,
    candidates: &[crate::model::PaperWatchCandidate],
    run_id_ms: i64,
) -> AppResult<Vec<crate::model::MarketLiveTick>> {
    if let Some(path) = args.market_live_tick_file.as_deref() {
        return read_market_live_ticks(path);
    }
    let Some(url) = args.market_live_nats_url.as_deref() else {
        return Err(AppError::config(
            "provide --market-live-tick-file or --market-live-nats-url",
        ));
    };
    let configs = market_live_nats_configs_for_candidates(args, candidates, url, run_id_ms);
    let mut ticks = Vec::new();
    for config in configs {
        ticks.extend(read_market_live_ticks_from_nats(&config).await?);
    }
    Ok(ticks)
}

pub(in crate::cli) fn market_live_nats_configs_for_candidates(
    args: &Args,
    candidates: &[crate::model::PaperWatchCandidate],
    url: &str,
    run_id_ms: i64,
) -> Vec<MarketLiveNatsConfig> {
    let base_config = MarketLiveNatsConfig {
        url: url.to_owned(),
        stream: args.market_live_nats_stream.clone(),
        subject: args.market_live_nats_subject.clone(),
        consumer: args.market_live_nats_consumer.clone(),
        deliver_policy: args.market_live_nats_deliver_policy.clone(),
        batch_size: args.market_live_nats_batch_size,
        max_messages: args.market_live_nats_max_messages,
        ack_wait_secs: args.market_live_nats_ack_wait_secs,
        delete_consumer_after_read: false,
    };
    if args.market_live_nats_subject != DEFAULT_MARKET_LIVE_NATS_SUBJECT {
        return vec![base_config];
    }

    let symbols = candidates
        .iter()
        .map(|candidate| market_live_subject_symbol_token(&candidate.symbol_canonical))
        .filter(|symbol| !symbol.is_empty())
        .collect::<BTreeSet<_>>();
    if symbols.is_empty() {
        return vec![base_config];
    }

    symbols
        .into_iter()
        .map(|symbol| MarketLiveNatsConfig {
            subject: format!("market_live_tick.created.*.{symbol}"),
            consumer: format!("{}-{run_id_ms}-{symbol}", args.market_live_nats_consumer),
            delete_consumer_after_read: true,
            ..base_config.clone()
        })
        .collect()
}

fn market_live_subject_symbol_token(value: &str) -> String {
    value
        .trim()
        .chars()
        .filter_map(|character| {
            if character.is_ascii_alphanumeric() {
                Some(character.to_ascii_lowercase())
            } else if character == '_' || character == '-' {
                Some(character)
            } else {
                None
            }
        })
        .collect()
}

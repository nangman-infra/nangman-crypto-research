use crate::cli::Args;
use crate::error::{AppError, AppResult};

pub(super) fn validate_market_live_tick_input(args: &Args) -> AppResult<()> {
    if let Some(path) = args.market_live_tick_file.as_deref()
        && !path.is_absolute()
    {
        return Err(AppError::config(
            "RESEARCH_MARKET_LIVE_TICK_FILE must be an absolute path",
        ));
    }
    if args.market_live_tick_file.is_some() && args.market_live_nats_url.is_some() {
        return Err(AppError::config(
            "use either --market-live-tick-file or --market-live-nats-url, not both",
        ));
    }
    if args.market_live_tick_file.is_none() && args.market_live_nats_url.is_none() {
        return Err(AppError::config(
            "--run-paper-watch-live-cycle requires market live tick file or NATS url",
        ));
    }
    validate_market_live_url(args)
}

pub(super) fn validate_market_live_url(args: &Args) -> AppResult<()> {
    if let Some(url) = args.market_live_nats_url.as_deref()
        && !url.starts_with("nats://")
    {
        return Err(AppError::config(
            "--market-live-nats-url must start with nats://",
        ));
    }
    Ok(())
}

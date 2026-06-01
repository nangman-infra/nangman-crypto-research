use super::super::price::valid_mark_price;
use super::*;
use crate::error::AppError;
use crate::model::{MARKET_LIVE_TICK_SCHEMA_VERSION, MarketLiveTick};

pub(in crate::paper_live) fn validate_tick(tick: &MarketLiveTick) -> AppResult<()> {
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

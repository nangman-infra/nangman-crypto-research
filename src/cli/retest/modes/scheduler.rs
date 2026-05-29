use super::super::*;
use super::focused::build_focused_retest_manifest_from_status;
use super::summary::retest_scheduler_summary;

pub(in crate::cli) async fn run_retest_cycle_scheduler_mode(args: &Args) -> AppResult<RunSummary> {
    let status = load_retest_horizon_status(args).await?;
    let validation = validate_retest_horizon_status(&status)?;
    let output_partition_at_ms = args.now_ms.unwrap_or_else(now_ms);

    if validation.scheduler_action == "WAIT_UNTIL_MARKET_L1_HORIZON_MATERIALIZES" {
        let run_not_before_ms = validation.run_not_before_ms.ok_or_else(|| {
            AppError::validation("WAIT scheduler action requires run_not_before_ms")
        })?;
        if output_partition_at_ms < run_not_before_ms {
            return Ok(retest_scheduler_summary(
                validation.scheduler_action,
                Some(run_not_before_ms),
            ));
        }
        return Ok(retest_scheduler_summary(
            "REFRESH_RETEST_HORIZON_STATUS_AFTER_WAIT_DEADLINE".to_owned(),
            Some(run_not_before_ms),
        ));
    }

    if validation.scheduler_action == "RUN_FOCUSED_RETEST_RESEARCH" {
        return build_focused_retest_manifest_from_status(
            args,
            &status,
            Some(validation.scheduler_action),
        )
        .await;
    }

    Ok(retest_scheduler_summary(
        validation.scheduler_action,
        validation.run_not_before_ms,
    ))
}

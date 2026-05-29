use crate::error::AppError;

pub(in crate::storage) fn is_missing_market_artifact(error: &AppError) -> bool {
    matches!(error, AppError::AwsNotFound(_))
}

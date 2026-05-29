use super::validation::validate_s3_location;
use crate::error::{AppError, AppResult};
use crate::storage::client::aws_error_detail;
use aws_sdk_s3::Client;
use aws_sdk_s3::error::ProvideErrorMetadata;

pub(in crate::storage) async fn get_object_bytes(
    client: &Client,
    bucket: &str,
    key: &str,
) -> AppResult<Vec<u8>> {
    validate_s3_location(bucket, key, "S3")?;
    let output = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await
        .map_err(|error| {
            if let Some(service_error) = error.as_service_error()
                && service_error.code() == Some("NoSuchKey")
            {
                return AppError::AwsNotFound(format!("s3://{bucket}/{key}"));
            }
            AppError::Aws(format!(
                "s3 get_object s3://{bucket}/{key}: {}",
                aws_error_detail(&error)
            ))
        })?;
    let bytes = output
        .body
        .collect()
        .await
        .map_err(|error| {
            AppError::Aws(format!(
                "s3 read body s3://{bucket}/{key}: {}",
                aws_error_detail(&error)
            ))
        })?
        .into_bytes()
        .to_vec();
    Ok(bytes)
}

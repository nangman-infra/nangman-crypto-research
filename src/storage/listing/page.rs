use aws_sdk_s3::Client;

use super::super::client::aws_error_detail;
use crate::error::{AppError, AppResult};

pub(super) async fn list_objects_page(
    client: &Client,
    bucket: &str,
    prefix: &str,
    continuation_token: Option<&str>,
) -> AppResult<aws_sdk_s3::operation::list_objects_v2::ListObjectsV2Output> {
    let mut request = client.list_objects_v2().bucket(bucket).prefix(prefix);
    if let Some(token) = continuation_token {
        request = request.continuation_token(token);
    }
    request.send().await.map_err(|error| {
        AppError::Aws(format!(
            "s3 list_objects_v2 s3://{bucket}/{prefix}: {}",
            aws_error_detail(&error)
        ))
    })
}

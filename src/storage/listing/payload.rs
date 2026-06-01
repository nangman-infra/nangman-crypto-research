use aws_sdk_s3::Client;

use super::limits::{PayloadListOptions, ensure_scan_limit};
use super::page::list_objects_page;
use super::selection::ListedPayloadObject;
use crate::error::AppResult;

pub(in crate::storage) async fn list_payload_objects_with_prefix(
    client: &Client,
    bucket: &str,
    prefix: &str,
    file_suffix: &str,
    scan_limit: usize,
) -> AppResult<Vec<ListedPayloadObject>> {
    let options = PayloadListOptions {
        file_suffix,
        scan_limit,
        artifact_label: "S3 payload",
    };
    list_payload_objects_with_options(client, bucket, prefix, options).await
}

pub(super) async fn list_payload_objects_with_options(
    client: &Client,
    bucket: &str,
    prefix: &str,
    options: PayloadListOptions<'_>,
) -> AppResult<Vec<ListedPayloadObject>> {
    let mut objects = Vec::new();
    let mut continuation_token: Option<String> = None;

    loop {
        let output =
            list_objects_page(client, bucket, prefix, continuation_token.as_deref()).await?;
        for object in output.contents() {
            let Some(key) = object.key() else {
                continue;
            };
            if !key.ends_with(options.file_suffix) {
                continue;
            }
            objects.push(ListedPayloadObject {
                key: key.to_owned(),
                last_modified_ms: object
                    .last_modified()
                    .and_then(|last_modified| last_modified.to_millis().ok())
                    .unwrap_or(0),
            });
            ensure_scan_limit(objects.len(), bucket, prefix, options)?;
        }

        continuation_token = output.next_continuation_token().map(ToOwned::to_owned);
        if continuation_token.is_none() {
            break;
        }
    }

    Ok(objects)
}

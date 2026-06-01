use aws_sdk_s3::Client;

use super::page::list_objects_page;
use crate::error::AppResult;

pub(in crate::storage) async fn latest_key_with_prefix(
    client: &Client,
    bucket: &str,
    prefix: &str,
    file_suffix: &str,
) -> AppResult<Option<String>> {
    let mut latest: Option<String> = None;
    let mut continuation_token: Option<String> = None;

    loop {
        let output =
            list_objects_page(client, bucket, prefix, continuation_token.as_deref()).await?;
        for object in output.contents() {
            let Some(key) = object.key() else {
                continue;
            };
            if !key.ends_with(file_suffix) {
                continue;
            }
            if latest
                .as_deref()
                .is_none_or(|current_latest| key > current_latest)
            {
                latest = Some(key.to_owned());
            }
        }

        continuation_token = output.next_continuation_token().map(ToOwned::to_owned);
        if continuation_token.is_none() {
            break;
        }
    }

    Ok(latest)
}

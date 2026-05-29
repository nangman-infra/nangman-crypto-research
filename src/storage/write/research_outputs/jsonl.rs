use aws_sdk_s3::Client;

use crate::error::AppResult;
use crate::storage::objects::put_jsonl_object;

use super::keys::{ResearchOutputS3Keys, s3_uri};

pub(super) async fn write_jsonl_dataset<T>(
    client: &Client,
    bucket: &str,
    keys: &ResearchOutputS3Keys,
    dataset: &str,
    schema_version: &str,
    records: &[T],
    written: &mut Vec<String>,
) -> AppResult<()>
where
    T: serde::Serialize,
{
    let key = keys.jsonl_dataset(dataset, schema_version);
    put_jsonl_object(client, bucket, &key, records).await?;
    written.push(s3_uri(bucket, &key));
    Ok(())
}

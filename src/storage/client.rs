use crate::error::AppResult;
use aws_config::BehaviorVersion;
use aws_sdk_s3::Client;
use aws_types::region::Region;
use std::env;

pub(super) async fn s3_client() -> AppResult<Client> {
    let mut loader = aws_config::defaults(BehaviorVersion::latest());
    if let Some(region) = env_string("AWS_REGION").or_else(|| env_string("AWS_DEFAULT_REGION")) {
        loader = loader.region(Region::new(region));
    }
    let config = loader.load().await;
    Ok(Client::new(&config))
}

fn env_string(name: &str) -> Option<String> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

pub(super) fn aws_error_detail(error: &(impl std::fmt::Debug + std::fmt::Display)) -> String {
    let display = error.to_string();
    let debug = format!("{error:?}");
    if debug == display {
        display
    } else {
        format!("{display}; debug={debug}")
    }
}

use std::collections::HashMap;
use std::time::Duration;

use anyhow::{Context, Result};
use gcloud_pubsub::client::Client;
use gcloud_pubsub::subscription::SubscriptionConfig;

#[derive(Debug, Clone)]
pub struct PulledMessage {
    pub ack_id: String,
    pub data: String,
    pub attributes: HashMap<String, String>,
    pub message_id: String,
    pub publish_time: Option<String>,
}

pub async fn create_subscription(
    client: &Client,
    topic_name: &str,
    sub_name: &str,
) -> Result<String> {
    let sub = client
        .create_subscription(sub_name, topic_name, SubscriptionConfig::default(), None)
        .await
        .with_context(|| format!("failed to create subscription: {sub_name}"))?;
    Ok(sub.fully_qualified_name().to_string())
}

pub async fn delete_subscription(client: &Client, sub_name: &str) -> Result<()> {
    let sub = client.subscription(sub_name);
    sub.delete(None)
        .await
        .with_context(|| format!("failed to delete subscription: {sub_name}"))?;
    Ok(())
}

pub async fn pull_messages(
    client: &Client,
    sub_name: &str,
    max_messages: i32,
    timeout_secs: u64,
) -> Result<Vec<PulledMessage>> {
    let sub = client.subscription(sub_name);
    let pull_fut = sub.pull(max_messages, None);
    let messages = tokio::time::timeout(Duration::from_secs(timeout_secs), pull_fut)
        .await
        .with_context(|| format!("pull timed out after {timeout_secs}s on: {sub_name}"))?
        .with_context(|| format!("failed to pull messages from: {sub_name}"))?;

    Ok(messages
        .into_iter()
        .map(|m| {
            let publish_time = m
                .message
                .publish_time
                .as_ref()
                .map(|ts| format!("{}.{}s", ts.seconds, ts.nanos));
            PulledMessage {
                ack_id: m.ack_id,
                data: String::from_utf8_lossy(&m.message.data).to_string(),
                attributes: m.message.attributes,
                message_id: m.message.message_id,
                publish_time,
            }
        })
        .collect())
}

pub async fn acknowledge(client: &Client, sub_name: &str, ack_ids: Vec<String>) -> Result<()> {
    let sub = client.subscription(sub_name);
    sub.ack(ack_ids)
        .await
        .with_context(|| format!("failed to acknowledge messages on: {sub_name}"))?;
    Ok(())
}

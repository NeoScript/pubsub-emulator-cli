use std::collections::HashMap;

use anyhow::{Context, Result};
use gcloud_googleapis::pubsub::v1::PubsubMessage;
use gcloud_pubsub::client::Client;

pub async fn list_topics(client: &Client) -> Result<Vec<String>> {
    client
        .get_topics(None)
        .await
        .context("failed to get topics")
        .map_err(Into::into)
}

pub async fn create_topic(client: &Client, name: &str) -> Result<String> {
    let topic = client.topic(name);
    let fqn = topic.fully_qualified_name().to_string();
    topic
        .create(None, None)
        .await
        .with_context(|| format!("failed to create topic: {fqn}"))?;
    Ok(fqn)
}

pub async fn delete_topic(client: &Client, name: &str) -> Result<()> {
    let topic = client.topic(name);
    topic.delete(None).await.context("failed to delete topic")?;
    Ok(())
}

pub async fn get_topic_subscriptions(client: &Client, topic_name: &str) -> Result<Vec<String>> {
    let topic = client.topic(topic_name);
    let subscriptions = topic
        .subscriptions(None)
        .await
        .context("failed to get subscriptions")?;
    Ok(subscriptions
        .iter()
        .map(|s| s.fully_qualified_name().to_string())
        .collect())
}

pub async fn publish_message(
    client: &Client,
    topic_name: &str,
    data: &str,
    attributes: HashMap<String, String>,
) -> Result<String> {
    let topic = client.topic(topic_name);
    let message = PubsubMessage {
        data: data.into(),
        attributes,
        ..Default::default()
    };
    let publisher = topic.new_publisher(None);
    let awaiter = publisher.publish_blocking(message);
    let result = awaiter.get().await.context("failed to publish")?;
    Ok(result)
}

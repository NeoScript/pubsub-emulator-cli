use std::collections::HashMap;

use crate::cli::{PubSubAttribute, TopicCommands};
use crate::services;
use anyhow::Result;
use gcloud_pubsub::client::Client;
use tokio::task::JoinSet;

pub async fn handle_topic_commands(cmd: &TopicCommands, client: &Client) -> Result<()> {
    match cmd {
        TopicCommands::Create { names } => create_topics(names, client).await?,
        TopicCommands::List => list_topics(client).await?,
        TopicCommands::Info { name } => get_topic_info(name, client).await?,
        TopicCommands::Delete { name } => delete_topic(name, client).await?,
        TopicCommands::Publish {
            topic_id,
            message,
            attributes,
        } => publish_message(topic_id, message, attributes, client).await?,
    };
    Ok(())
}

async fn create_topics(names: &[String], client: &Client) -> Result<()> {
    let mut set = JoinSet::new();

    names.iter().for_each(|n| {
        let client = client.clone();
        let topic_name = n.clone();

        set.spawn(async move { services::topics::create_topic(&client, &topic_name).await });
    });

    while let Some(task_result) = set.join_next().await {
        match task_result {
            Ok(topic_result) => match topic_result {
                Ok(name) => println!("topic_created: {name}"),
                Err(e) => eprintln!("failed creating topic: {:?}\n", e),
            },
            Err(e) => eprintln!("Task panicked: {e}"),
        }
    }
    Ok(())
}

async fn list_topics(client: &Client) -> Result<()> {
    let topics = services::topics::list_topics(client).await?;
    topics.iter().for_each(|n| {
        println!("{}", n);
    });
    Ok(())
}

async fn get_topic_info(name: &str, client: &Client) -> Result<()> {
    let topic = client.topic(name);
    let topic_name = topic.fully_qualified_name();
    println!("topic: {}", topic_name);

    let subscriptions = services::topics::get_topic_subscriptions(client, name).await?;

    if subscriptions.is_empty() {
        println!("no subscriptions found");
    } else {
        subscriptions.iter().for_each(|s| {
            println!("{}", s);
        });
    }

    Ok(())
}

async fn delete_topic(name: &str, client: &Client) -> Result<()> {
    let topic = client.topic(name);
    let topic_name = topic.fully_qualified_name();
    services::topics::delete_topic(client, name).await?;
    println!("topic deleted: {}", topic_name);
    Ok(())
}

async fn publish_message(
    topic_id: &str,
    message: &str,
    attributes: &Option<Vec<PubSubAttribute>>,
    client: &Client,
) -> Result<()> {
    let mut attrs_to_send = HashMap::new();
    if let Some(attrs) = attributes {
        attrs.iter().for_each(|a| {
            attrs_to_send.insert(a.key.clone(), a.value.clone());
        });
    }
    println!("publishing to topic: {}", topic_id);
    let result =
        services::topics::publish_message(client, topic_id, message, attrs_to_send).await?;
    println!("received: {result} after publish");
    Ok(())
}

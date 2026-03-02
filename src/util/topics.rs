use crate::cli::TopicCommands;
use anyhow::{Context, Result};
use google_cloud_pubsub::{client::Client, topic::Topic};

pub async fn handle_topic_commands(cmd: &TopicCommands, client: &Client) -> Result<()> {
    match cmd {
        TopicCommands::Create { name } => create_topic(name, client).await?,
        TopicCommands::List => list_topics(client).await?,
        TopicCommands::Info { name } => get_topic_info(name, client).await?,
        TopicCommands::Delete { name } => delete_topic(name, client).await?,
    };
    Ok(())
}

async fn create_topic(name: &str, client: &Client) -> Result<()> {
    let topic = client.topic(name);
    topic
        .create(None, None)
        .await
        .context("failed to create topic")?;

    println!("topic created: {}", topic.fully_qualified_name());
    Ok(())
}

async fn list_topics(client: &Client) -> Result<()> {
    let topic_list = client
        .get_topics(None)
        .await
        .context("failed to get topics")?;

    topic_list.iter().for_each(|n| {
        println!("{}", n);
    });

    Ok(())
}

async fn get_topic_info(name: &str, client: &Client) -> Result<()> {
    let topic: Topic = client.topic(name);
    let topic_name = topic.fully_qualified_name();
    println!("topic: {}", topic_name);

    let subscriptions = topic
        .subscriptions(None)
        .await
        .context("failed to get subscriptions")?;

    if subscriptions.is_empty() {
        println!("no subscriptions found");
    } else {
        subscriptions.iter().for_each(|s| {
            let sub_name = s.fully_qualified_name();
            println!("{}", sub_name);
        });
    }

    Ok(())
}

async fn delete_topic(name: &str, client: &Client) -> Result<()> {
    let topic = client.topic(name);
    let topic_name = topic.fully_qualified_name();

    topic.delete(None).await.context("failed to delete topic")?;

    println!("topic deleted: {}", topic_name);
    Ok(())
}

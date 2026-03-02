use crate::cli::TopicCommands;
use anyhow::{Context, Result};
use google_cloud_pubsub::{client::Client, topic::Topic};
use tokio::task::JoinSet;

pub async fn handle_topic_commands(cmd: &TopicCommands, client: &Client) -> Result<()> {
    match cmd {
        TopicCommands::Create { names } => create_topics(names, client).await?,
        TopicCommands::List => list_topics(client).await?,
        TopicCommands::Info { name } => get_topic_info(name, client).await?,
        TopicCommands::Delete { name } => delete_topic(name, client).await?,
    };
    Ok(())
}

async fn create_topics(names: &[String], client: &Client) -> Result<()> {
    let mut set = JoinSet::new();

    names.iter().for_each(|n| {
        let client = client.clone();
        let topic_name = n.clone();

        set.spawn(async move {
            let topic = client.topic(&topic_name);
            let topic_id = topic.fully_qualified_name();
            topic
                .create(None, None)
                .await
                .with_context(|| format!("failed to create topic: {}", topic_id))
                .map(|_| topic_id.to_string())
        });
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

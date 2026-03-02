use clap::Parser;

use anyhow::Result;
use cli::Cli;
use google_cloud_gax::conn::Environment;
use google_cloud_pubsub::client::{Client as PubSubClient, ClientConfig};

mod cli;
mod util;

use util::topics::handle_topic_commands;

use crate::cli::PubsubCommands;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    // TODO: swap to pulling config from env or config.json in the future
    let pubsub_config = ClientConfig {
        environment: Environment::Emulator("localhost:8681".to_string()),
        project_id: Some("my-project".to_string()),
        ..Default::default()
    };
    let pubsub_client = PubSubClient::new(pubsub_config).await.unwrap();

    let cli = Cli::parse();
    match &cli.commands {
        PubsubCommands::Topics(cmd) => handle_topic_commands(cmd, &pubsub_client).await,
        PubsubCommands::Init(init_args) => todo!(),
    }?;
    Ok(())
}

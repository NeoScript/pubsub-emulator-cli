use clap::Parser;

use anyhow::Result;
use cli::Cli;
use gcloud_gax::conn::Environment;
use gcloud_pubsub::client::{Client as PubSubClient, ClientConfig};

mod cli;
mod config;
mod util;

use crate::cli::PubsubCommands;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let conf_path = confy::get_configuration_file_path("pubsub-emulator-cli", "config")?;
    println!("loading configuration from: {:?}", conf_path);
    let conf: config::AppConfig = confy::load("pubsub-emulator-cli", "config")?;

    let project_id = &conf.active_project;
    let host = conf.get_host(project_id)?;

    let pubsub_config = ClientConfig {
        environment: Environment::Emulator(host),
        project_id: Some(project_id.to_string()),
        ..Default::default()
    };
    let pubsub_client = PubSubClient::new(pubsub_config).await.unwrap();

    let cli = Cli::parse();
    match &cli.commands {
        PubsubCommands::Topics(cmd) => util::handle_topic_commands(cmd, &pubsub_client).await,
        PubsubCommands::Init(_init_args) => todo!(),
        PubsubCommands::Projects(cmd) => util::handle_project_commands(cmd, conf).await,
    }?;
    Ok(())
}

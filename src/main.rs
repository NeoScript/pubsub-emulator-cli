use clap::Parser;

use anyhow::Result;
use cli::Cli;
use gcloud_gax::conn::Environment;
use gcloud_pubsub::client::{Client as PubSubClient, ClientConfig};

mod cli;
mod config;
mod services;
mod web;
mod util;

use crate::cli::PubsubCommands;

#[tokio::main]
async fn main() -> Result<()> {
    let conf_path = confy::get_configuration_file_path("pubsub-emulator-cli", "config")?;
    println!("loading configuration from: {:?}", conf_path);
    let conf: config::AppConfig = confy::load("pubsub-emulator-cli", "config")?;

    let cli = Cli::parse();
    match cli.commands {
        PubsubCommands::Topics(ref cmd) => {
            let pubsub_client = create_pubsub_client(&conf).await?;
            util::handle_topic_commands(cmd, &pubsub_client).await
        }
        PubsubCommands::Init(_init_args) => todo!(),
        PubsubCommands::Projects(ref cmd) => {
            let _pubsub_client = create_pubsub_client(&conf).await?;
            util::handle_project_commands(cmd, conf).await
        }
        PubsubCommands::Serve(args) => {
            web::server::start(conf, args).await
        }
    }?;
    Ok(())
}

async fn create_pubsub_client(conf: &config::AppConfig) -> Result<PubSubClient> {
    let project_id = &conf.active_project;
    let host = conf.get_host(project_id)?;
    let pubsub_config = ClientConfig {
        environment: Environment::Emulator(host),
        project_id: Some(project_id.to_string()),
        ..Default::default()
    };
    Ok(PubSubClient::new(pubsub_config).await.unwrap())
}

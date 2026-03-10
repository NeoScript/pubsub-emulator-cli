use std::collections::HashMap;

use clap::Parser;
use serde::{Deserialize, Serialize};

use anyhow::{Context, Result};
use cli::Cli;
use gcloud_gax::conn::Environment;
use gcloud_pubsub::client::{Client as PubSubClient, ClientConfig};

mod cli;
mod util;

use crate::cli::PubsubCommands;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    active_project: String,

    /// mapping of project_id -> host_address (without http://)
    projects: HashMap<String, String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            active_project: "my-project".to_string(),
            projects: HashMap::from([("my-project".to_string(), "localhost:8681".to_string())]),
        }
    }
}

impl AppConfig {
    fn get_host(&self, project: &str) -> Result<String> {
        let host = self
            .projects
            .get(project)
            .with_context(|| format!("Project's address not found: {}", project))?;

        Ok(host.to_string())
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let conf_path = confy::get_configuration_file_path("pubsub-emulator-cli", "config")?;
    println!("loading configuration from: {:?}", conf_path);
    let conf: AppConfig = confy::load("pubsub-emulator-cli", "config")?;

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

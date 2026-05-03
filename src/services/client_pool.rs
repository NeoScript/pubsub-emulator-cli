use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{Context, Result};
use gcloud_gax::conn::Environment;
use gcloud_pubsub::client::{Client, ClientConfig};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct ClientPool {
    clients: Arc<RwLock<HashMap<String, Client>>>,
}

impl ClientPool {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get(&self, project: &str, host: &str) -> Result<Client> {
        {
            let cache = self.clients.read().await;
            if let Some(client) = cache.get(project) {
                return Ok(client.clone());
            }
        }
        let config = ClientConfig {
            environment: Environment::Emulator(host.to_string()),
            project_id: Some(project.to_string()),
            ..Default::default()
        };
        let client = Client::new(config)
            .await
            .context("failed to create PubSub client")?;
        self.clients
            .write()
            .await
            .insert(project.to_string(), client.clone());
        Ok(client)
    }
}

impl Default for ClientPool {
    fn default() -> Self {
        Self::new()
    }
}

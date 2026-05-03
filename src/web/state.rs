use std::sync::Arc;
use tokio::sync::RwLock;

use minijinja::Environment;

use crate::config::AppConfig;
use crate::services::client_pool::ClientPool;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<RwLock<AppConfig>>,
    pub clients: ClientPool,
    pub jinja: Arc<Environment<'static>>,
}

impl AppState {
    pub fn new(config: AppConfig, jinja: Environment<'static>) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            clients: ClientPool::new(),
            jinja: Arc::new(jinja),
        }
    }

    pub async fn client_for_project(
        &self,
        project: &str,
    ) -> anyhow::Result<gcloud_pubsub::client::Client> {
        let config = self.config.read().await;
        let host = config.get_host(project)?;
        self.clients.get(project, &host).await
    }

    pub async fn project_ids(&self) -> Vec<String> {
        let config = self.config.read().await;
        config.projects.keys().cloned().collect()
    }
}

pub fn build_template_env(template_dir: &str) -> Environment<'static> {
    let mut env = Environment::new();
    env.set_loader(minijinja::path_loader(template_dir));
    env
}

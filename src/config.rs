use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use anyhow::{Context, Result};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub active_project: String,

    /// mapping of project_id -> host_address (without http://)
    pub projects: HashMap<String, String>,
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
    pub fn get_host(&self, project: &str) -> Result<String> {
        let host = self
            .projects
            .get(project)
            .with_context(|| format!("Project's address not found: {}", project))?;

        Ok(host.to_string())
    }
}

use crate::AppConfig;
use crate::cli::{ProjectArgs, ProjectCommands};
use anyhow::{Context, Result};

pub async fn handle_project_commands(cmd: &ProjectCommands, config: AppConfig) -> Result<()> {
    match cmd {
        ProjectCommands::Add(project_args) => add_project(project_args, config)?,
        ProjectCommands::Delete { name } => delete_project(name, config)?,
        ProjectCommands::List => list_projects(&config)?,
        ProjectCommands::SetActive { name } => set_active(name, config)?,
        ProjectCommands::GetActive => get_active(&config)?,
    };
    Ok(())
}

fn add_project(project: &ProjectArgs, mut config: AppConfig) -> Result<()> {
    config
        .projects
        .entry(project.name.to_string())
        .insert_entry(project.host.to_string());
    confy::store("pubsub-emulator-cli", "config", config).context("saving configuration file")?;
    println!("Added project {0} bound to {1}", project.name, project.host);
    Ok(())
}

fn delete_project(project: &str, mut config: AppConfig) -> Result<()> {
    // TODO: reset the default project if the delete occurs on the default
    let (removed_project, removed_host) = config
        .projects
        .remove_entry(project)
        .context("Failed to find project: {project}")?;

    println!("Removed: {} @ {}", &removed_project, &removed_host);
    Ok(())
}

fn list_projects(config: &AppConfig) -> Result<()> {
    println!("active project: {}", config.active_project);

    println!("projects:");
    config.projects.iter().for_each(|mapping| {
        println!("{0} @ {1}", mapping.0, mapping.1);
    });
    Ok(())
}

fn get_active(config: &AppConfig) -> Result<()> {
    let project_name = &config.active_project;
    let project_host = config
        .projects
        .get(project_name)
        .context("Failed to determine host for active project: {project_name}")?;

    println!("active project: {} @ {}", project_name, project_host);

    Ok(())
}

fn set_active(project: &str, mut config: AppConfig) -> Result<()> {
    let host = config.get_host(project)?;

    config.active_project = project.to_string();
    confy::store("pubsub-emulator-cli", "config", config).context("Failed to save config file")?;
    println!("Updated active project: {} @ {}", project, host);
    Ok(())
}

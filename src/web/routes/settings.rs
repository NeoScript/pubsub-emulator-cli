use axum::extract::State;
use axum::response::Html;
use axum::Form;
use minijinja::context;
use serde::Deserialize;

use crate::web::state::AppState;

#[derive(Deserialize)]
pub struct UpdateSettingsForm {
    pub project: String,
    pub host: String,
}

pub async fn get_settings(State(state): State<AppState>) -> Html<String> {
    let config = state.config.read().await;
    let projects: Vec<(String, String)> = config
        .projects
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    let active = config.active_project.clone();
    drop(config);

    let tmpl = state.jinja.get_template("partials/settings_modal.html").unwrap();
    Html(tmpl.render(context! {
        projects => &projects,
        active_project => &active,
    }).unwrap())
}

pub async fn update_settings(
    State(state): State<AppState>,
    Form(form): Form<UpdateSettingsForm>,
) -> Html<String> {
    let mut config = state.config.write().await;
    config.projects.insert(form.project.clone(), form.host.clone());
    let _ = confy::store("pubsub-emulator-cli", "config", &*config);
    drop(config);

    let tmpl = state.jinja.get_template("partials/toast.html").unwrap();
    Html(tmpl.render(context! {
        message => format!("Updated {} -> {}", form.project, form.host),
        is_error => false,
    }).unwrap())
}

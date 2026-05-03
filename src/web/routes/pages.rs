use axum::extract::{Path, Query, State};
use axum::response::Html;
use axum::Form;
use minijinja::context;
use serde::Deserialize;

use crate::web::state::AppState;

#[derive(Deserialize)]
pub struct ProjectQuery {
    pub project: String,
}

#[derive(Deserialize)]
pub struct AddProjectForm {
    pub project_id: String,
    #[serde(default = "default_host")]
    pub host: String,
}

#[derive(Deserialize)]
pub struct UpdateHostForm {
    pub host: String,
}

fn default_host() -> String {
    "localhost:8681".to_string()
}

pub async fn index(State(state): State<AppState>) -> Html<String> {
    let config = state.config.read().await;
    let projects: Vec<(String, String)> = {
        let mut entries: Vec<_> = config.projects.iter()
            .map(|(id, host)| (id.clone(), host.clone()))
            .collect();
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        entries
    };
    drop(config);
    let tmpl = state.jinja.get_template("index.html").unwrap();
    Html(tmpl.render(context! { projects => &projects }).unwrap())
}

pub async fn project_page(
    State(state): State<AppState>,
    Query(params): Query<ProjectQuery>,
) -> Html<String> {
    let config = state.config.read().await;
    let host = config.get_host(&params.project).unwrap_or_default();
    let tmpl = state.jinja.get_template("project.html").unwrap();
    Html(tmpl
        .render(context! {
            project => &params.project,
            emulator_host => &host,
        })
        .unwrap())
}

pub async fn add_project(
    State(state): State<AppState>,
    Form(form): Form<AddProjectForm>,
) -> Html<String> {
    let project_id = form.project_id.clone();
    let host = form.host.clone();
    let mut config = state.config.write().await;
    config.projects.insert(project_id.clone(), host.clone());
    let _ = confy::store("pubsub-emulator-cli", "config", &*config);
    drop(config);

    Html(format!(
        r#"<tr class="hover group">
  <td>
    <span class="font-mono font-medium text-sm">{0}</span>
  </td>
  <td>
    <span class="text-base-content/60 text-sm font-mono">{1}</span>
  </td>
  <td class="text-right">
    <div class="flex items-center justify-end gap-2">
      <a href="/project?project={0}" class="btn btn-primary btn-xs gap-1">
        Open
        <svg xmlns="http://www.w3.org/2000/svg" class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5">
          <path stroke-linecap="round" stroke-linejoin="round" d="M13.5 4.5L21 12m0 0l-7.5 7.5M21 12H3"/>
        </svg>
      </a>
      <button class="btn btn-ghost btn-xs text-error"
              hx-delete="/projects/{0}/delete"
              hx-target="closest tr"
              hx-swap="outerHTML"
              hx-confirm="Remove project '{0}' from config?">
        <svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
          <path stroke-linecap="round" stroke-linejoin="round" d="M14.74 9l-.346 9m-4.788 0L9.26 9m9.968-3.21c.342.052.682.107 1.022.166m-1.022-.165L18.16 19.673a2.25 2.25 0 01-2.244 2.077H8.084a2.25 2.25 0 01-2.244-2.077L4.772 5.79m14.456 0a48.108 48.108 0 00-3.478-.397m-12 .562c.34-.059.68-.114 1.022-.165m0 0a48.11 48.11 0 013.478-.397m7.5 0v-.916c0-1.18-.91-2.164-2.09-2.201a51.964 51.964 0 00-3.32 0c-1.18.037-2.09 1.022-2.09 2.201v.916m7.5 0a48.667 48.667 0 00-7.5 0"/>
        </svg>
      </button>
    </div>
  </td>
</tr>"#,
        project_id, host
    ))
}

pub async fn delete_project(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Html<String> {
    let mut config = state.config.write().await;
    config.projects.remove(&project_id);
    let _ = confy::store("pubsub-emulator-cli", "config", &*config);
    drop(config);
    Html(String::new())
}

pub async fn update_project_host(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    Form(form): Form<UpdateHostForm>,
) -> Html<String> {
    let host = form.host.clone();
    let mut config = state.config.write().await;
    config.projects.insert(project_id.clone(), host.clone());
    let _ = confy::store("pubsub-emulator-cli", "config", &*config);
    drop(config);

    let tmpl = state.jinja.get_template("partials/toast.html").unwrap();
    Html(tmpl.render(minijinja::context! {
        message => format!("Updated {project_id} → {host}"),
        is_error => false,
    }).unwrap())
}

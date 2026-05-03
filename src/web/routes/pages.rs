use axum::extract::{Query, State};
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
    <a href="/project?project={0}" class="btn btn-primary btn-xs gap-1">
      Open
      <svg xmlns="http://www.w3.org/2000/svg" class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5">
        <path stroke-linecap="round" stroke-linejoin="round" d="M13.5 4.5L21 12m0 0l-7.5 7.5M21 12H3"/>
      </svg>
    </a>
  </td>
</tr>"#,
        project_id, host
    ))
}

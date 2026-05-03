use axum::extract::{Query, State};
use axum::response::Html;
use minijinja::context;
use serde::Deserialize;

use crate::web::state::AppState;

#[derive(Deserialize)]
pub struct ProjectQuery {
    pub project: String,
}

pub async fn index(State(state): State<AppState>) -> Html<String> {
    let projects = state.project_ids().await;
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

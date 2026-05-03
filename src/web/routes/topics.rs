use axum::extract::{Path, State};
use axum::response::Html;
use axum::Form;
use minijinja::context;
use serde::Deserialize;

use crate::web::state::AppState;

#[derive(Deserialize)]
pub struct CreateTopicForm {
    pub name: String,
}

pub async fn list_topics(
    State(state): State<AppState>,
    Path(project): Path<String>,
) -> Html<String> {
    let result = match state.client_for_project(&project).await {
        Ok(client) => crate::services::topics::list_topics(&client).await,
        Err(e) => Err(e),
    };

    let _env = state.jinja.acquire_env().unwrap();
    let tmpl = _env.get_template("partials/topic_list.html").unwrap();
    match result {
        Ok(topics) => {
            let short_names: Vec<String> = topics
                .iter()
                .map(|t| t.rsplit('/').next().unwrap_or(t).to_string())
                .collect();
            Html(tmpl.render(context! { topics => &short_names, project => &project }).unwrap())
        }
        Err(e) => Html(tmpl.render(context! { topics => Vec::<String>::new(), project => &project, error => format!("{e:#}") }).unwrap()),
    }
}

pub async fn create_topic(
    State(state): State<AppState>,
    Path(project): Path<String>,
    Form(form): Form<CreateTopicForm>,
) -> Html<String> {
    let client = match state.client_for_project(&project).await {
        Ok(c) => c,
        Err(e) => return render_toast(&state, &format!("Client error: {e:#}"), true),
    };

    match crate::services::topics::create_topic(&client, &form.name).await {
        Ok(_) => list_topics(State(state), Path(project)).await,
        Err(e) => render_toast(&state, &format!("Failed to create topic: {e:#}"), true),
    }
}

pub async fn delete_topic(
    State(state): State<AppState>,
    Path((project, topic)): Path<(String, String)>,
) -> Html<String> {
    let client = match state.client_for_project(&project).await {
        Ok(c) => c,
        Err(e) => return render_toast(&state, &format!("Client error: {e:#}"), true),
    };

    match crate::services::topics::delete_topic(&client, &topic).await {
        Ok(_) => list_topics(State(state), Path(project)).await,
        Err(e) => render_toast(&state, &format!("Failed to delete topic: {e:#}"), true),
    }
}

pub async fn topic_detail(
    State(state): State<AppState>,
    Path((project, topic)): Path<(String, String)>,
) -> Html<String> {
    let _env = state.jinja.acquire_env().unwrap();
    let tmpl = _env.get_template("partials/topic_detail.html").unwrap();
    Html(tmpl.render(context! {
        topic => &topic,
        project => &project,
    }).unwrap())
}

fn render_toast(state: &AppState, message: &str, is_error: bool) -> Html<String> {
    let _env = state.jinja.acquire_env().unwrap();
    let tmpl = _env.get_template("partials/toast.html").unwrap();
    Html(tmpl.render(context! { message => message, is_error => is_error }).unwrap())
}

use axum::extract::{Path, Query, State};
use axum::response::Html;
use axum::Form;
use minijinja::context;
use serde::Deserialize;

use crate::web::state::AppState;

#[derive(Deserialize)]
pub struct CreateSubForm {
    pub name: String,
}

pub async fn list_subscriptions(
    State(state): State<AppState>,
    Path((project, topic)): Path<(String, String)>,
) -> Html<String> {
    let result = match state.client_for_project(&project).await {
        Ok(client) => crate::services::topics::get_topic_subscriptions(&client, &topic).await,
        Err(e) => Err(e),
    };

    let _env = state.jinja.acquire_env().unwrap();
    let tmpl = _env.get_template("partials/subscription_list.html").unwrap();
    match result {
        Ok(subs) => {
            let short_names: Vec<String> = subs
                .iter()
                .map(|s| s.rsplit('/').next().unwrap_or(s).to_string())
                .collect();
            Html(tmpl.render(context! {
                subscriptions => &short_names,
                topic => &topic,
                project => &project,
            }).unwrap())
        }
        Err(e) => Html(tmpl.render(context! {
            subscriptions => Vec::<String>::new(),
            topic => &topic,
            project => &project,
            error => e.to_string(),
        }).unwrap()),
    }
}

pub async fn create_subscription(
    State(state): State<AppState>,
    Path((project, topic)): Path<(String, String)>,
    Form(form): Form<CreateSubForm>,
) -> Html<String> {
    let client = match state.client_for_project(&project).await {
        Ok(c) => c,
        Err(e) => return render_toast(&state, &format!("Client error: {e}"), true),
    };

    match crate::services::subscriptions::create_subscription(&client, &topic, &form.name).await {
        Ok(_) => list_subscriptions(State(state), Path((project, topic))).await,
        Err(e) => render_toast(&state, &format!("Failed to create subscription: {e}"), true),
    }
}

pub async fn delete_subscription(
    State(state): State<AppState>,
    Path((project, subscription)): Path<(String, String)>,
) -> Html<String> {
    let client = match state.client_for_project(&project).await {
        Ok(c) => c,
        Err(e) => return render_toast(&state, &format!("Client error: {e}"), true),
    };

    match crate::services::subscriptions::delete_subscription(&client, &subscription).await {
        Ok(_) => render_toast(&state, &format!("Deleted subscription: {subscription}"), false),
        Err(e) => render_toast(&state, &format!("Failed to delete: {e}"), true),
    }
}

#[derive(Deserialize)]
pub struct SubDetailQuery {
    #[serde(default)]
    pub topic: String,
}

pub async fn subscription_detail(
    State(state): State<AppState>,
    Path((project, subscription)): Path<(String, String)>,
    Query(query): Query<SubDetailQuery>,
) -> Html<String> {
    let _env = state.jinja.acquire_env().unwrap();
    let tmpl = _env.get_template("partials/subscription_detail.html").unwrap();
    Html(tmpl.render(context! {
        subscription => &subscription,
        project => &project,
        topic => &query.topic,
    }).unwrap())
}

fn render_toast(state: &AppState, message: &str, is_error: bool) -> Html<String> {
    let _env = state.jinja.acquire_env().unwrap();
    let tmpl = _env.get_template("partials/toast.html").unwrap();
    Html(tmpl.render(context! { message => message, is_error => is_error }).unwrap())
}

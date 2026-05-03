use std::collections::HashMap;

use axum::extract::{Path, State};
use axum::response::Html;
use axum::Form;
use minijinja::context;
use serde::Deserialize;

use crate::web::state::AppState;

#[derive(Deserialize)]
pub struct PublishForm {
    pub data: String,
    #[serde(default)]
    pub attr_keys: Vec<String>,
    #[serde(default)]
    pub attr_values: Vec<String>,
}

#[derive(Deserialize)]
pub struct PullForm {
    #[serde(default = "default_max_messages")]
    pub max_messages: i32,
}

fn default_max_messages() -> i32 {
    10
}

#[derive(Deserialize)]
pub struct AckForm {
    pub ack_id: String,
}

pub async fn publish_message(
    State(state): State<AppState>,
    Path((project, topic)): Path<(String, String)>,
    Form(form): Form<PublishForm>,
) -> Html<String> {
    let client = match state.client_for_project(&project).await {
        Ok(c) => c,
        Err(e) => return render_toast(&state, &format!("Client error: {e}"), true),
    };

    let mut attributes = HashMap::new();
    for (key, value) in form.attr_keys.iter().zip(form.attr_values.iter()) {
        if !key.is_empty() {
            attributes.insert(key.clone(), value.clone());
        }
    }

    match crate::services::topics::publish_message(&client, &topic, &form.data, attributes).await {
        Ok(msg_id) => render_toast(&state, &format!("Published message: {msg_id}"), false),
        Err(e) => render_toast(&state, &format!("Publish failed: {e}"), true),
    }
}

pub async fn pull_messages(
    State(state): State<AppState>,
    Path((project, subscription)): Path<(String, String)>,
    Form(form): Form<PullForm>,
) -> Html<String> {
    let client = match state.client_for_project(&project).await {
        Ok(c) => c,
        Err(e) => return render_toast(&state, &format!("Client error: {e}"), true),
    };

    match crate::services::subscriptions::pull_messages(&client, &subscription, form.max_messages).await {
        Ok(messages) => {
            let tmpl = state.jinja.get_template("partials/message_card.html").unwrap();
            let rendered: Vec<String> = messages
                .iter()
                .map(|m| {
                    let attrs: Vec<(String, String)> = m.attributes.iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect();
                    tmpl.render(context! {
                        message_id => &m.message_id,
                        data => &m.data,
                        attributes => &attrs,
                        publish_time => &m.publish_time,
                        ack_id => &m.ack_id,
                        project => &project,
                        subscription => &subscription,
                    }).unwrap()
                })
                .collect();

            if rendered.is_empty() {
                Html(r#"<div class="flex flex-col items-center gap-2 py-8 text-center">
  <div class="w-10 h-10 rounded-full bg-base-100 flex items-center justify-center">
    <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5 text-base-content/30" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
      <path stroke-linecap="round" stroke-linejoin="round" d="M9 3.75H6.912a2.25 2.25 0 00-2.15 1.588L2.35 13.177a2.25 2.25 0 00-.1.661V18a2.25 2.25 0 002.25 2.25h15A2.25 2.25 0 0021.75 18v-4.162c0-.224-.034-.447-.1-.661L19.24 5.338a2.25 2.25 0 00-2.15-1.588H15M2.25 13.5h3.86a2.25 2.25 0 012.012 1.244l.256.512a2.25 2.25 0 002.013 1.244h3.218a2.25 2.25 0 002.013-1.244l.256-.512a2.25 2.25 0 012.013-1.244h3.859M12 3v8.25m0 0l-3-3m3 3l3-3"/>
    </svg>
  </div>
  <p class="text-sm text-base-content/40">No messages available in this subscription.</p>
</div>"#.to_string())
            } else {
                let count = rendered.len();
                let header = format!(
                    r#"<div class="flex items-center gap-2 mb-3 text-sm text-base-content/60">
  <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
    <path stroke-linecap="round" stroke-linejoin="round" d="M3 16.5v2.25A2.25 2.25 0 005.25 21h13.5A2.25 2.25 0 0021 18.75V16.5M16.5 12L12 16.5m0 0L7.5 12m4.5 4.5V3"/>
  </svg>
  Pulled <span class="font-semibold text-base-content mx-1">{count}</span> message{}</div>"#,
                    if count == 1 { "" } else { "s" }
                );
                Html(format!("{}\n<div class=\"flex flex-col gap-3\">{}</div>", header, rendered.join("\n")))
            }
        }
        Err(e) => render_toast(&state, &format!("Pull failed: {e}"), true),
    }
}

pub async fn ack_message(
    State(state): State<AppState>,
    Path((project, subscription)): Path<(String, String)>,
    Form(form): Form<AckForm>,
) -> Html<String> {
    let client = match state.client_for_project(&project).await {
        Ok(c) => c,
        Err(e) => return render_toast(&state, &format!("Client error: {e}"), true),
    };

    match crate::services::subscriptions::acknowledge(&client, &subscription, vec![form.ack_id]).await {
        Ok(_) => Html(String::new()),
        Err(e) => render_toast(&state, &format!("Ack failed: {e}"), true),
    }
}

fn render_toast(state: &AppState, message: &str, is_error: bool) -> Html<String> {
    let tmpl = state.jinja.get_template("partials/toast.html").unwrap();
    Html(tmpl.render(context! { message => message, is_error => is_error }).unwrap())
}

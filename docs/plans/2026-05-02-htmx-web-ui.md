# HTMX Web UI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an embedded web UI to the `pse` CLI so a single Rust binary serves both the CLI and a browser-based PubSub emulator management interface, replacing the Angular-based pubsub-emulator-ui.

**Architecture:** The `pse serve` subcommand starts an axum HTTP server that renders HTML server-side with MiniJinja templates (runtime-loaded for fast dev iteration; swap to askama for compile-time checking later). HTMX handles partial page updates without a JS framework. The server uses the same `gcloud-pubsub` client as the CLI through a shared service layer — business logic is written once, the CLI prints text output and the web UI renders HTML. Configuration reuses the existing `confy`-managed `AppConfig` — the same projects and emulator hosts the CLI uses.

**Tech Stack:** Rust, axum, minijinja (+ minijinja-autoreload), HTMX, DaisyUI (Tailwind CSS via CDN), gcloud-pubsub (shared), tower-http (ServeDir), tokio, tracing

> **Note:** We use MiniJinja (runtime templates) during development for a fast edit-refresh loop — no recompile needed for template changes. The Jinja2 syntax is compatible with askama, so swapping to compile-time templates later is low-friction.

---

## File Structure

```
src/
  main.rs                      -- entry point, dispatch CLI vs serve
  cli.rs                       -- clap definitions (add Serve subcommand)
  config.rs                    -- AppConfig (unchanged — reused by web UI)
  util.rs                      -- re-exports (unchanged)
  util/
    topics.rs                  -- CLI topic handlers (calls services, prints)
    projects.rs                -- CLI project handlers (calls services, prints)
  services/
    mod.rs                     -- service module re-exports
    topics.rs                  -- topic CRUD: returns data (shared by CLI + web)
    subscriptions.rs           -- subscription CRUD + pull/ack (shared)
    projects.rs                -- project config management (shared)
    client_pool.rs             -- per-project PubSubClient cache
  web/
    mod.rs                     -- axum router assembly, shared AppState
    state.rs                   -- AppState + MiniJinja Environment setup
    routes/
      mod.rs                   -- route module re-exports
      pages.rs                 -- full-page GET handlers (index, project)
      topics.rs                -- topic CRUD handlers (HTMX partials)
      subscriptions.rs         -- subscription CRUD handlers (HTMX partials)
      messages.rs              -- publish/pull/ack handlers (HTMX partials)
      settings.rs              -- runtime settings handlers
  web/
    templates/                 -- MiniJinja HTML templates (runtime-loaded, hot-reload in dev)
      base.html                -- layout shell (head, nav, DaisyUI)
      index.html               -- project selector page
      project.html             -- project workspace page
      partials/
        topic_list.html        -- topic list sidebar partial
        subscription_list.html -- subscription list partial
        topic_detail.html      -- topic detail + publish form
        subscription_detail.html -- subscription detail + pull view
        message_card.html      -- single pulled message card
        settings_modal.html    -- settings dialog partial
        toast.html             -- success/error toast partial
  web/
    static/                    -- served from disk via tower-http ServeDir (no recompile)
      htmx.min.js              -- HTMX library (~14kb gzipped)
      styles.css               -- custom CSS overrides (DaisyUI + Tailwind via CDN for dev)

Cargo.toml                     -- add axum, minijinja, etc. (no new PubSub deps)
Dockerfile                     -- (deferred to prod) single-binary image
docker-compose.yml             -- (deferred to prod) reference new image
```

---

## Task 1: Add New Dependencies to Cargo.toml

**Files:**

- Modify: `Cargo.toml`
- **Step 1: Update Cargo.toml with new dependencies**

```toml
[package]
name = "pse"
description = "a simple cli tool for interacting with google pubsub emulator"
version = "0.2.0"
edition = "2024"

[dependencies]
anyhow = "1.0.102"
axum = "0.8"
clap = { version = "4.5.47", features = ["derive", "env"] }
confy = "2.0.0"
minijinja = { version = "2", features = ["loader"] }
minijinja-autoreload = "2"
gcloud-gax = "1.3.2"
gcloud-pubsub = { version = "1.6.0" }
gcloud-googleapis = { version = "1.3.0" }
serde = { version = "1.0.228", features = ["derive"] }
tokio = { version = "1.47.1", features = ["rt-multi-thread", "macros"] }
tower-http = { version = "0.6", features = ["fs"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

No new PubSub dependencies — we reuse the existing `gcloud-pubsub` client through a shared service layer. The only new deps are for the web server (axum, minijinja, tower-http) and observability (tracing). tokio changes from `current_thread` to `rt-multi-thread` because axum needs a multi-threaded runtime. The existing `[profile.release]` section in Cargo.toml is unchanged — production optimizations will be added in a later pass.

**Dev loop tip:** Use `cargo watch -x 'run -- serve'` to auto-recompile on Rust source changes. Template and static asset changes are picked up on browser refresh with no recompile needed.

- **Step 2: Verify it compiles**

Run: `cargo check`
Expected: compiles with warnings about unused imports (OK for now)

- **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "deps: add axum, minijinja, tower-http for web UI"
```

---

## Task 2: Add Serve Subcommand to CLI

**Files:**

- Modify: `src/cli.rs`
- Modify: `src/main.rs`
- **Step 1: Add Serve variant and ServeArgs to cli.rs**

Add to the `PubsubCommands` enum:

```rust
/// Start the web UI server
Serve(ServeArgs),
```

Add the struct:

```rust
#[derive(Parser, Debug)]
pub struct ServeArgs {
    /// Port to listen on
    #[arg(short, long, default_value = "7200", env = "PSE_PORT")]
    pub port: u16,

    /// Bind address
    #[arg(short, long, default_value = "0.0.0.0", env = "PSE_BIND")]
    pub bind: String,
}
```

Emulator host and project config come from the existing `confy`-managed `AppConfig` — no need to duplicate them here.

- **Step 2: Add match arm in main.rs**

In the match block in main, add:

```rust
PubsubCommands::Serve(args) => {
    println!("serve command registered (not yet implemented)");
    println!("would listen on {}:{}", args.bind, args.port);
    Ok(())
}
```

- **Step 3: Verify it works**

Run: `cargo run -- serve --help`
Expected: shows help text with `--port`, `--bind`, `--config`, `--emulator-host` options

Run: `cargo run -- serve`
Expected: prints placeholder message about listening on 0.0.0.0:7200

- **Step 4: Commit**

```bash
git add src/cli.rs src/main.rs
git commit -m "feat: add serve subcommand skeleton"
```

---

## Task 3: ~~Extend Config~~ — REMOVED

> **Removed:** The web UI reuses the existing `AppConfig` via `confy`. No separate `WebConfig`, `pse.toml`, or seed data system needed. The `projects` HashMap already maps project IDs to emulator hosts. Server bind/port are handled by `ServeArgs` (clap CLI args + env vars).

---

## Task 4: Refactor into Shared Service Layer

**Goal:** Extract business logic from `util/topics.rs` into a `services/` module that returns data instead of printing. Both CLI and web UI call the same service functions. Add subscription and message operations the web UI needs.

**Files:**

- Create: `src/services/mod.rs`
- Create: `src/services/topics.rs`
- Create: `src/services/subscriptions.rs`
- Create: `src/services/client_pool.rs`
- Modify: `src/util/topics.rs` (thin wrapper: call services, print)
- Modify: `src/main.rs` (add `mod services`)
- **Step 1: Create src/services/client_pool.rs**

Per-project `PubSubClient` cache so we don't recreate gRPC channels per request:

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::{Context, Result};
use gcloud_gax::conn::Environment;
use gcloud_pubsub::client::{Client, ClientConfig};

#[derive(Clone)]
pub struct ClientPool {
    clients: Arc<RwLock<HashMap<String, Client>>>,
}

impl ClientPool {
    pub fn new() -> Self {
        Self { clients: Arc::new(RwLock::new(HashMap::new())) }
    }

    pub async fn get(&self, project: &str, host: &str) -> Result<Client> {
        {
            let cache = self.clients.read().await;
            if let Some(client) = cache.get(project) {
                return Ok(client.clone());
            }
        }
        let config = ClientConfig {
            environment: Environment::Emulator(host.to_string()),
            project_id: Some(project.to_string()),
            ..Default::default()
        };
        let client = Client::new(config).await
            .context("failed to create PubSub client")?;
        self.clients.write().await.insert(project.to_string(), client.clone());
        Ok(client)
    }
}
```

- **Step 2: Create src/services/topics.rs**

Service functions that return data instead of printing:

```rust
use anyhow::{Context, Result};
use gcloud_pubsub::client::Client;
use gcloud_googleapis::pubsub::v1::PubsubMessage;
use std::collections::HashMap;

pub async fn list_topics(client: &Client) -> Result<Vec<String>> {
    let topics = client.get_topics(None).await
        .context("failed to get topics")?;
    Ok(topics.iter().map(|t| t.to_string()).collect())
}

pub async fn create_topic(client: &Client, name: &str) -> Result<String> {
    let topic = client.topic(name);
    topic.create(None, None).await
        .with_context(|| format!("failed to create topic: {}", name))?;
    Ok(topic.fully_qualified_name().to_string())
}

pub async fn delete_topic(client: &Client, name: &str) -> Result<()> {
    let topic = client.topic(name);
    topic.delete(None).await.context("failed to delete topic")?;
    Ok(())
}

pub async fn get_topic_subscriptions(client: &Client, topic_name: &str) -> Result<Vec<String>> {
    let topic = client.topic(topic_name);
    let subs = topic.subscriptions(None).await
        .context("failed to get subscriptions")?;
    Ok(subs.iter().map(|s| s.fully_qualified_name().to_string()).collect())
}

pub async fn publish_message(
    client: &Client, topic_name: &str, data: &str,
    attributes: HashMap<String, String>,
) -> Result<String> {
    let topic = client.topic(topic_name);
    let message = PubsubMessage {
        data: data.into(),
        attributes,
        ..Default::default()
    };
    let publisher = topic.new_publisher(None);
    let awaiter = publisher.publish_blocking(message);
    let result = awaiter.get().await.context("failed to publish")?;
    Ok(result)
}
```

- **Step 3: Create src/services/subscriptions.rs**

New subscription operations needed by the web UI (verify exact `gcloud-pubsub` API during implementation):

```rust
use anyhow::{Context, Result};
use gcloud_pubsub::client::Client;
use std::collections::HashMap;

pub struct PulledMessage {
    pub ack_id: String,
    pub data: String,
    pub attributes: HashMap<String, String>,
    pub message_id: String,
    pub publish_time: Option<String>,
}

pub async fn create_subscription(
    client: &Client, topic_name: &str, sub_name: &str,
) -> Result<String> {
    let topic = client.topic(topic_name);
    let sub = topic.subscribe(sub_name, Default::default(), None).await
        .context("failed to create subscription")?;
    Ok(sub.fully_qualified_name().to_string())
}

pub async fn delete_subscription(client: &Client, sub_name: &str) -> Result<()> {
    let sub = client.subscription(sub_name);
    sub.delete(None).await.context("failed to delete subscription")?;
    Ok(())
}

pub async fn pull_messages(
    client: &Client, sub_name: &str, max_messages: i32,
) -> Result<Vec<PulledMessage>> {
    let sub = client.subscription(sub_name);
    let messages = sub.pull(max_messages, None).await
        .context("failed to pull messages")?;
    Ok(messages.into_iter().map(|m| PulledMessage {
        ack_id: m.ack_id().to_string(),
        data: String::from_utf8_lossy(m.message.data.as_ref()).to_string(),
        attributes: m.message.attributes.clone(),
        message_id: m.message.message_id.clone(),
        publish_time: m.message.publish_time.map(|t| t.to_string()),
    }).collect())
}

pub async fn acknowledge(
    client: &Client, sub_name: &str, ack_ids: Vec<String>,
) -> Result<()> {
    let sub = client.subscription(sub_name);
    sub.acknowledge(ack_ids, None).await.context("failed to ack")?;
    Ok(())
}
```

- **Step 4: Create src/services/mod.rs**

```rust
pub mod client_pool;
pub mod subscriptions;
pub mod topics;
```

- **Step 5: Refactor src/util/topics.rs to call services**

CLI handlers become thin wrappers — call service, print result:

```rust
use crate::services;

async fn list_topics(client: &Client) -> Result<()> {
    let topics = services::topics::list_topics(client).await?;
    topics.iter().for_each(|t| println!("{}", t));
    Ok(())
}
// ... same pattern for create, delete, info, publish
```

- **Step 6: Add `mod services` to main.rs**
- **Step 7: Verify existing CLI still works**

```bash
cargo build
cargo run -- topics list
```

- **Step 8: Commit**

```bash
git add src/services/ src/util/topics.rs src/main.rs
git commit -m "refactor: extract shared service layer from CLI handlers"
```

---

## Task 5: Static Assets (HTMX + DaisyUI CSS)

**Files:**

- Create: `src/web/static/htmx.min.js` (download from unpkg)
- Create: `src/web/static/styles.css` (placeholder — using CDN for dev)
- Modify: `src/web/mod.rs`
- **Step 1: Download HTMX**

```bash
curl -o src/web/static/htmx.min.js https://unpkg.com/htmx.org@2.0.4/dist/htmx.min.js
```

- **Step 2: Create a placeholder CSS file**

We use DaisyUI + Tailwind via CDN links in `base.html` during dev — no build step needed. The `styles.css` is a placeholder for any custom overrides.

```bash
mkdir -p src/web/static
touch src/web/static/styles.css
```

- **Step 3: Static files are served via tower-http ServeDir**

No custom handler code needed. In the router (Task 11), static files are served with a single line:

```rust
.nest_service("/static", ServeDir::new("src/web/static"))
```

This serves files directly from disk — changes are picked up on refresh with no recompile.

- **Step 4: Update src/web/mod.rs**

```rust
pub mod routes;
```

(No emulator client module — the web routes use the shared `services/` layer.)

- **Step 5: Verify and commit**

Run: `cargo check`

```bash
git add src/web/static/ src/web/mod.rs
git commit -m "feat: add static assets directory with HTMX"
```

---

## Task 6: Base Template and AppState

**Files:**

- Create: `src/web/templates/base.html`
- Create: `src/web/state.rs`
- Modify: `src/web/mod.rs`
- **Step 1: Create src/web/state.rs**

```rust
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

    pub async fn client_for_project(&self, project: &str) -> anyhow::Result<gcloud_pubsub::client::Client> {
        let config = self.config.read().await;
        let host = config.get_host(project)?;
        self.clients.get(project, &host).await
    }

    pub async fn project_ids(&self) -> Vec<String> {
        let config = self.config.read().await;
        config.projects.keys().cloned().collect()
    }
}
```

The `ClientPool` lazily creates and caches `gcloud-pubsub` gRPC clients per project. Route handlers call `state.client_for_project("my-project")` to get a client, then pass it to service functions.

- **Step 2: Create src/web/state.rs — template loader helper**

Add a helper to build the MiniJinja environment (called from server startup):

```rust
pub fn build_template_env(template_dir: &str) -> Environment<'static> {
    let mut env = Environment::new();
    env.set_loader(minijinja::path_loader(template_dir));
    env
}
```

- **Step 2: Create src/web/templates/base.html**

```html
<!DOCTYPE html>
<html lang="en" data-theme="dark">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{% block title %}PubSub Emulator{% endblock %}</title>
  <link href="https://cdn.jsdelivr.net/npm/daisyui@4/dist/full.min.css" rel="stylesheet">
  <script src="https://cdn.tailwindcss.com"></script>
  <script src="/static/htmx.min.js"></script>
</head>
<body class="min-h-screen bg-base-200">
  <div class="navbar bg-base-100 shadow-lg">
    <div class="flex-1">
      <a href="/" class="btn btn-ghost text-xl">PSE</a>
    </div>
    <div class="flex-none gap-2">
      <span class="badge badge-info">PSE Web UI</span>
      <button class="btn btn-ghost btn-sm"
              hx-get="/settings" hx-target="#modal-container" hx-swap="innerHTML">
        Settings
      </button>
    </div>
  </div>
  <main class="container mx-auto p-4">
    {% block content %}{% endblock %}
  </main>
  <div id="modal-container"></div>
  <div id="toast-container" class="toast toast-end"></div>
</body>
</html>
```

- **Step 3: Update src/web/mod.rs**

```rust
pub mod routes;
pub mod state;
```

- **Step 4: Commit**

```bash
git add src/web/state.rs src/web/templates/
git commit -m "feat: add AppState and base HTML template with DaisyUI"
```

---

## Task 7: Route Handlers — Full Pages

**Files:**

- Create: `src/web/routes/mod.rs`
- Create: `src/web/routes/pages.rs`
- Create: `src/web/templates/index.html`
- Create: `src/web/templates/project.html`
- **Step 1: Create src/web/routes/mod.rs**

```rust
pub mod pages;
pub mod topics;
pub mod subscriptions;
pub mod messages;
pub mod settings;
```

- **Step 2: Create src/web/routes/pages.rs**

```rust
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
    Html(tmpl.render(context! {
        projects => &projects,
    }).unwrap())
}

pub async fn project_page(
    State(state): State<AppState>,
    Query(params): Query<ProjectQuery>,
) -> Html<String> {
    let config = state.config.read().await;
    let host = config.get_host(&params.project).unwrap_or_default();
    let tmpl = state.jinja.get_template("project.html").unwrap();
    Html(tmpl.render(context! {
        project => &params.project,
        emulator_host => &host,
    }).unwrap())
}
```

Note: With MiniJinja, route handlers call `state.jinja.get_template()` and `.render(context! { ... })` instead of using askama derive macros. Templates are the same Jinja2 HTML files — no code changes needed in the `.html` files themselves.

- **Step 3: Create src/web/templates/index.html**

```html
{% extends "base.html" %}
{% block title %}PSE - Projects{% endblock %}
{% block content %}
<div class="flex flex-col items-center gap-6 mt-8">
  <h1 class="text-3xl font-bold">Projects</h1>
  <div id="project-list" class="flex flex-wrap gap-4 justify-center">
    {% for project in projects %}
    <a href="/project?project={{ project }}" class="card bg-base-100 shadow-md hover:shadow-xl transition-shadow cursor-pointer w-64">
      <div class="card-body">
        <h2 class="card-title">{{ project }}</h2>
      </div>
    </a>
    {% endfor %}
  </div>
  <form hx-post="/projects" hx-target="#project-list" hx-swap="beforeend" class="join">
    <input type="text" name="project_id" placeholder="New project ID"
           class="input input-bordered join-item" required>
    <button type="submit" class="btn btn-primary join-item">Add</button>
  </form>
</div>
{% endblock %}
```

- **Step 4: Create src/web/templates/project.html**

```html
{% extends "base.html" %}
{% block title %}PSE - {{ project }}{% endblock %}
{% block content %}
<div class="flex gap-4 h-[calc(100vh-10rem)]">
  <!-- Topics panel -->
  <div class="w-1/4 bg-base-100 rounded-lg p-4 overflow-y-auto">
    <h2 class="text-lg font-bold mb-2">Topics</h2>
    <div id="topic-list"
         hx-get="/projects/{{ project }}/topics"
         hx-trigger="load"
         hx-swap="innerHTML">
      <span class="loading loading-spinner"></span>
    </div>
  </div>
  <!-- Detail panel -->
  <div id="detail-panel" class="flex-1 bg-base-100 rounded-lg p-4 overflow-y-auto">
    <p class="text-base-content/50">Select a topic to get started</p>
  </div>
</div>
{% endblock %}
```

- **Step 5: Commit**

```bash
git add src/web/routes/ src/web/templates/
git commit -m "feat: add index and project page routes with templates"
```

---

## Task 8: Route Handlers — Topic HTMX Partials

**Files:**

- Create: `src/web/routes/topics.rs`
- Create: `src/web/templates/partials/topic_list.html`
- Create: `src/web/templates/partials/topic_detail.html`
- **Step 1: Create src/web/routes/topics.rs**

Handlers: `list_topics` (GET partial), `create_topic` (POST returns updated list), `delete_topic` (DELETE returns updated list), `topic_detail` (GET detail panel with publish form).

Each returns an `Html<String>` rendered via MiniJinja (not a full page — just a partial fragment).

- **Step 2: Create topic_list.html partial**

Renders a list of topic cards. Each card uses `hx-get` to load topic detail into the detail panel. Includes a create-topic form at the bottom.

- **Step 3: Create topic_detail.html partial**

Shows topic name, subscription list (loaded via `hx-get` on load), and an inline publish-message form with textarea + attributes key/value inputs.

- **Step 4: Verify and commit**

```bash
cargo check
git add src/web/routes/topics.rs src/web/templates/partials/
git commit -m "feat: add topic list and detail HTMX partials"
```

---

## Task 9: Route Handlers — Subscription HTMX Partials

**Files:**

- Create: `src/web/routes/subscriptions.rs`
- Create: `src/web/templates/partials/subscription_list.html`
- Create: `src/web/templates/partials/subscription_detail.html`
- **Step 1: Create src/web/routes/subscriptions.rs**

Handlers: `list_subscriptions` (GET partial for topic), `create_subscription` (POST with pull/push type selector), `delete_subscription` (DELETE), `subscription_detail` (GET detail with pull UI).

- **Step 2: Create subscription_list.html partial**

List of subscription items for a topic. Each clickable to load detail. Create form at bottom with name input and push/pull toggle (DaisyUI radio buttons). Push shows an endpoint URL input.

- **Step 3: Create subscription_detail.html partial**

Shows subscription info, pull button (`hx-post` to pull endpoint), pulled messages container.

- **Step 4: Verify and commit**

```bash
cargo check
git add src/web/routes/subscriptions.rs src/web/templates/partials/
git commit -m "feat: add subscription list and detail HTMX partials"
```

---

## Task 10: Route Handlers — Messages (Publish/Pull/Ack)

**Files:**

- Create: `src/web/routes/messages.rs`
- Create: `src/web/templates/partials/message_card.html`
- Create: `src/web/templates/partials/toast.html`
- **Step 1: Create src/web/routes/messages.rs**

Handlers:

- `publish_message` — POST, reads form data (message body + attributes), calls `services::topics::publish_message`, returns toast partial.
- `pull_messages` — POST, calls `services::subscriptions::pull_messages` (data already decoded by service layer), returns list of message_card partials.
- `ack_message` — POST, calls `services::subscriptions::acknowledge`, returns empty response (HTMX `hx-swap="delete"` removes the card from DOM).
- **Step 2: Create message_card.html partial**

DaisyUI card showing: message ID, publish time, decoded data, attributes as badges, ack button that uses `hx-post` with `hx-swap="delete"` to remove itself.

- **Step 3: Create toast.html partial**

DaisyUI alert component for success/error feedback, auto-dismisses via HTMX `hx-swap="outerHTML settle:3s"` or a small inline script.

- **Step 4: Verify and commit**

```bash
cargo check
git add src/web/routes/messages.rs src/web/templates/partials/
git commit -m "feat: add publish, pull, and ack message handlers"
```

---

## Task 11: Router Assembly and Server Startup

**Files:**

- Create: `src/web/server.rs`
- Modify: `src/web/mod.rs`
- Modify: `src/main.rs` (wire up `pse serve`)
- **Step 1: Create src/web/server.rs**

```rust
use axum::{routing::{get, post, put, delete}, Router};
use tower_http::services::ServeDir;
use tracing_subscriber::EnvFilter;

use crate::cli::ServeArgs;
use crate::config::AppConfig;
use crate::web::{routes, state::{AppState, build_template_env}};
pub async fn start(config: AppConfig, args: ServeArgs) -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let jinja = build_template_env("src/web/templates");
    let state = AppState::new(config, jinja);

    let app = Router::new()
        .route("/", get(routes::pages::index))
        .route("/project", get(routes::pages::project_page))
        .route("/projects", post(routes::pages::add_project))
        .route("/projects/{project}/topics", get(routes::topics::list_topics))
        .route("/projects/{project}/topics", post(routes::topics::create_topic))
        .route("/topics/{*fqn}/detail", get(routes::topics::topic_detail))
        .route("/topics/{*fqn}/delete", delete(routes::topics::delete_topic))
        .route("/topics/{*fqn}/subscriptions",
            get(routes::subscriptions::list_subscriptions))
        .route("/topics/{*fqn}/subscriptions",
            post(routes::subscriptions::create_subscription))
        .route("/subscriptions/{*fqn}/detail",
            get(routes::subscriptions::subscription_detail))
        .route("/subscriptions/{*fqn}/delete",
            delete(routes::subscriptions::delete_subscription))
        .route("/topics/{*fqn}/publish", post(routes::messages::publish_message))
        .route("/subscriptions/{*fqn}/pull", post(routes::messages::pull_messages))
        .route("/subscriptions/{*fqn}/ack", post(routes::messages::ack_message))
        .route("/settings", get(routes::settings::get_settings))
        .route("/settings", put(routes::settings::update_settings))
        .nest_service("/static", ServeDir::new("src/web/static"))
        .with_state(state);

    let addr = format!("{}:{}", args.bind, args.port);
    tracing::info!("starting pse web UI on {}", addr);
    tracing::info!("projects: {:?}", state.project_ids().await);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
```

- **Step 2: Wire up serve command in main.rs**

Replace the placeholder serve match arm. The serve command reuses the same `confy` config the CLI already loads:

```rust
PubsubCommands::Serve(args) => {
    web::server::start(conf, args).await?;
    Ok(())
}
```

That's it — `conf` is the existing `AppConfig` loaded at the top of `main()`. No separate config parsing needed.

- **Step 3: Update tokio runtime to multi-thread**

Change `#[tokio::main(flavor = "current_thread")]` to `#[tokio::main]` in main.rs.

- **Step 4: Verify full startup**

Run: `cargo run -- serve`
Expected: prints "starting pse web UI on 0.0.0.0:7200"

- **Step 5: Commit**

```bash
git add src/web/server.rs src/web/mod.rs src/main.rs Cargo.toml
git commit -m "feat: assemble axum router and wire up pse serve"
```

---

## Task 12: Dockerfile and Docker Compose

**Files:**

- Create: `Dockerfile`
- Modify: `docker-compose.yml`
- **Step 1: Create Dockerfile**

```dockerfile
FROM rust:1-bookworm AS build
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12
COPY --from=build /app/target/release/pse /pse
EXPOSE 7200
ENTRYPOINT ["/pse", "serve"]
```

- **Step 2: Update docker-compose.yml**

```yaml
version: "3.9"

services:
  pubsub_emulator:
    image: thekevjames/gcloud-pubsub-emulator:latest
    ports:
      - "8681:8681"

  pse:
    build: .
    ports:
      - "7200:7200"
    command: ["serve"]
    depends_on:
      - pubsub_emulator
```

- **Step 3: Commit**

```bash
git add Dockerfile docker-compose.yml
git commit -m "feat: add Dockerfile and docker-compose for pse serve"
```

---

## Task 13: Settings Route and Integration Polish

**Files:**

- Create: `src/web/routes/settings.rs`
- Create: `src/web/templates/partials/settings_modal.html`
- **Step 1: Create settings route**

GET returns a DaisyUI modal partial showing the current project config. PUT accepts updates, modifies the AppState's config, returns a toast confirming the change.

- **Step 2: Create settings_modal.html**

DaisyUI modal with input for emulator URL, save button using `hx-put="/settings"`.

- **Step 3: Commit**

```bash
git add src/web/routes/settings.rs src/web/templates/partials/settings_modal.html
git commit -m "feat: add settings modal for runtime emulator URL change"
```

---

## Task 14: End-to-End Smoke Test

- **Step 1: Start emulator and pse serve**

```bash
docker compose up pubsub_emulator -d
cargo run -- serve --emulator-host localhost:8681
```

- **Step 2: Manual smoke test checklist**

1. Open [http://localhost:7200](http://localhost:7200) — see project list
2. Add a project — project card appears without page reload
3. Click project — see topics panel, empty
4. Create a topic — appears in list
5. Click topic — see detail with publish form and subscriptions
6. Create a pull subscription — appears in subscription list
7. Publish a message — see success toast
8. Click subscription — see pull button
9. Pull messages — message cards appear with decoded data
10. Ack a message — card disappears from DOM
11. Delete subscription — removed from list
12. Change emulator URL in settings — badge updates

- **Step 3: Commit any fixes from smoke test**

```bash
git add -A
git commit -m "fix: smoke test fixes"
```


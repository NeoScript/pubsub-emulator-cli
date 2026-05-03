use axum::routing::{delete, get, post};
use axum::Router;
use tower_http::services::ServeDir;
use tracing_subscriber::EnvFilter;

use crate::cli::ServeArgs;
use crate::config::AppConfig;
use crate::web::routes;
use crate::web::state::{build_template_env, AppState};

pub async fn start(config: AppConfig, args: ServeArgs) -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let jinja = build_template_env("src/web/templates");
    let state = AppState::new(config, jinja);

    let project_ids = state.project_ids().await;

    let app = Router::new()
        .route("/", get(routes::pages::index))
        .route("/project", get(routes::pages::project_page))
        .route("/projects", post(routes::pages::add_project))
        .route("/projects/{project}/delete", delete(routes::pages::delete_project))
        .route("/projects/{project}/ping", get(routes::pages::ping_project))
        .route(
            "/projects/{project}/topics",
            get(routes::topics::list_topics).post(routes::topics::create_topic),
        )
        .route(
            "/projects/{project}/topics/{topic}/detail",
            get(routes::topics::topic_detail),
        )
        .route(
            "/projects/{project}/topics/{topic}/delete",
            delete(routes::topics::delete_topic),
        )
        .route(
            "/projects/{project}/topics/{topic}/subscriptions",
            get(routes::subscriptions::list_subscriptions)
                .post(routes::subscriptions::create_subscription),
        )
        .route(
            "/projects/{project}/subscriptions/{subscription}/detail",
            get(routes::subscriptions::subscription_detail),
        )
        .route(
            "/projects/{project}/subscriptions/{subscription}/delete",
            delete(routes::subscriptions::delete_subscription),
        )
        .route(
            "/projects/{project}/topics/{topic}/publish",
            post(routes::messages::publish_message),
        )
        .route(
            "/projects/{project}/subscriptions/{subscription}/pull",
            post(routes::messages::pull_messages),
        )
        .route(
            "/projects/{project}/subscriptions/{subscription}/ack",
            post(routes::messages::ack_message),
        )
        .nest_service("/static", ServeDir::new("src/web/static"))
        .with_state(state);

    let addr = format!("{}:{}", args.bind, args.port);
    tracing::info!("starting pse web UI on http://{}", addr);
    tracing::info!("configured projects: {:?}", project_ids);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

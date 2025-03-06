use std::{sync::Arc, time::Duration};

use axum::{
    error_handling::HandleErrorLayer,
    http::StatusCode,
    routing::{delete, get, post},
    Router,
};
use cache::{TodoCache, TodoCacheMessage};
use ractor::{Actor, ActorRef};
use routes::{crash_todo, get_test, get_todo, post_todo};
use tokio::sync::Mutex;
use tower::{BoxError, ServiceBuilder};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod cache;
mod database;
mod entry;
mod list;
mod routes;
mod server;

#[derive(Clone)]
pub struct AppState {
    todo_cache: ActorRef<TodoCacheMessage>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .try_init()
        .expect("Could not start tracing");

    let (actor, _handle) = Actor::spawn(None, TodoCache, ())
        .await
        .expect("Failed to start cache actor");

    let state = Arc::new(Mutex::new(AppState { todo_cache: actor }));

    let app = Router::new()
        .route("/todo/:name/:date", get(get_todo))
        .route("/crash-todo/:name", delete(crash_todo))
        .route("/test", get(get_test))
        .route("/todo", post(post_todo))
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|error: BoxError| async move {
                    if error.is::<tower::timeout::error::Elapsed>() {
                        Ok(StatusCode::REQUEST_TIMEOUT)
                    } else {
                        Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Unhandled internal error: {error}"),
                        ))
                    }
                }))
                .timeout(Duration::from_secs(10))
                .layer(TraceLayer::new_for_http())
                .into_inner(),
        )
        .with_state(state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

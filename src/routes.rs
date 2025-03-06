use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::{cache::TodoCacheMessage, entry::TodoEntry, server::TodoServerMessage, AppState};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TodoEntryRequest {
    pub name: String,
    pub date: NaiveDate,
    pub title: String,
}

pub async fn post_todo(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(entry): Json<TodoEntryRequest>,
) -> impl IntoResponse {
    let state = state.lock().await;

    let server_process = state
        .todo_cache
        .call(
            |reply| TodoCacheMessage::ServerProcess {
                name: entry.name,
                reply,
            },
            None,
        )
        .await
        .expect("could not send")
        .expect("could not get rpc reply");

    let _ = server_process.cast(TodoServerMessage::Post {
        entry: TodoEntry {
            date: entry.date,
            title: entry.title,
        },
    });

    "Entry added"
}

pub async fn crash_todo(
    Path(name): Path<String>,
    State(state): State<Arc<Mutex<AppState>>>,
) -> impl IntoResponse {
    let state = state.lock().await;

    let server_process = state
        .todo_cache
        .call(
            |reply| TodoCacheMessage::ServerProcess { name, reply },
            None,
        )
        .await
        .expect("could not send")
        .expect("could not get rpc reply");

    let _ = server_process.cast(TodoServerMessage::Crash);

    "Entry added"
}

pub async fn get_todo(
    Path((name, date)): Path<(String, NaiveDate)>,
    State(state): State<Arc<Mutex<AppState>>>,
) -> impl IntoResponse {
    let state = state.lock().await;

    let server_process = state
        .todo_cache
        .call(
            |reply| TodoCacheMessage::ServerProcess { name, reply },
            None,
        )
        .await
        .expect("could not send")
        .expect("could not get rpc reply");

    let entries = server_process
        .call(|reply| TodoServerMessage::Get { date, reply }, None)
        .await
        .expect("couldnt send")
        .expect("couldnt retrieve");

    Json(entries)
}

pub async fn get_test(State(_state): State<Arc<Mutex<AppState>>>) -> impl IntoResponse {
    "here"
}

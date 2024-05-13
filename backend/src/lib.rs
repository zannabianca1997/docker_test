use std::{fmt::Debug, path::Path, sync::Arc};

use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::{OwnedRwLockReadGuard, RwLock};
use ts_rs::{ExportError, TS};

#[derive(Clone)]
pub struct ServerStatus {
    messages: Arc<RwLock<Vec<StoredMessage>>>,
    started_at: DateTime<Utc>,
}
impl ServerStatus {
    fn new() -> Self {
        Self {
            messages: Arc::new(RwLock::new(vec![])),
            started_at: chrono::Local::now().to_utc(),
        }
    }

    async fn get_board(&self) -> BoardLock {
        BoardLock {
            time: Local::now().to_utc(),
            started_at: self.started_at,
            messages: self.messages.clone().read_owned().await,
        }
    }
}

#[derive(Debug, Clone, Serialize, TS)]
pub struct StoredMessage {
    /// The time this message was received by the server
    time: DateTime<Utc>,
    #[serde(flatten)]
    msg: Message,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct Message {
    /// The user that posted this message
    user: String,
    /// The message posted
    content: String,
}

/// A locked state of the board, ready to be serialized
struct BoardLock {
    /// The time the board was locked
    time: DateTime<Utc>,
    /// The time the server was started
    started_at: DateTime<Utc>,
    /// The messages at that time
    messages: OwnedRwLockReadGuard<Vec<StoredMessage>>,
}

#[derive(Serialize, TS)]
#[ts(rename = "Board")]
struct BorrowedBoardLock<'a> {
    /// The time the board was locked
    time: DateTime<Utc>,
    /// The time the server was started
    started_at: DateTime<Utc>,
    /// The messages at that time
    messages: &'a [StoredMessage],
}

impl Serialize for BoardLock {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        BorrowedBoardLock {
            time: self.time,
            started_at: self.started_at,
            messages: &self.messages,
        }
        .serialize(serializer)
    }
}

async fn get_board(State(state): State<ServerStatus>) -> Json<BoardLock> {
    Json(state.get_board().await)
}

async fn post_message(State(state): State<ServerStatus>, Json(msg): Json<Message>) -> () {
    state.messages.write().await.push(StoredMessage {
        time: Local::now().to_utc(),
        msg,
    })
}

pub fn build() -> Router {
    Router::new()
        .route("/", get(get_board))
        .route("/", post(post_message))
        .with_state(ServerStatus::new())
}

pub fn bindgen(path: impl AsRef<Path>) -> Result<(), Vec<ExportError>> {
    [
        BorrowedBoardLock::export_all_to(&path),
        Message::export_all_to(&path),
        StoredMessage::export_all_to(&path),
    ]
    .into_iter()
    .fold(Ok(()), |acc, res| match (acc, res) {
        (Ok(()), Ok(())) => Ok(()),
        (Ok(()), Err(err)) => Err(vec![err]),
        (Err(errs), Ok(())) => Err(errs),
        (Err(mut errs), Err(err)) => {
            errs.push(err);
            Err(errs)
        }
    })
}

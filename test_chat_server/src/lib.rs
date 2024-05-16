use std::{fmt::Debug, sync::Arc};

use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::{OwnedRwLockReadGuard, RwLock};

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

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "bindgen", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
/// A stored message
pub struct StoredMessage {
    /// The time this message was received by the server
    time: DateTime<Utc>,
    /// The user that posted this message
    user: String,
    /// The message posted
    content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "bindgen", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
/// A message
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

#[derive(Serialize)]
#[cfg_attr(feature = "bindgen", derive(schemars::JsonSchema))]
#[serde(rename = "Board", deny_unknown_fields)]
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

#[cfg(feature = "bindgen")]
impl schemars::JsonSchema for BoardLock {
    fn schema_name() -> String {
        BorrowedBoardLock::schema_name()
    }

    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        BorrowedBoardLock::json_schema(gen)
    }

    fn is_referenceable() -> bool {
        BorrowedBoardLock::is_referenceable()
    }

    fn schema_id() -> std::borrow::Cow<'static, str> {
        BorrowedBoardLock::schema_id()
    }
}

async fn get_board(State(state): State<ServerStatus>) -> Json<BoardLock> {
    Json(state.get_board().await)
}

async fn post_message(
    State(state): State<ServerStatus>,
    Json(Message { user, content }): Json<Message>,
) -> () {
    state.messages.write().await.push(StoredMessage {
        time: Local::now().to_utc(),
        user,
        content,
    })
}

pub fn build() -> Router {
    Router::new()
        .route("/", get(get_board))
        .route("/", post(post_message))
        .with_state(ServerStatus::new())
}

#[cfg(feature = "bindgen")]
pub fn bindgen() {
    use std::io::stdout;

    use schemars::schema_for;
    use serde_json::to_writer;

    to_writer(stdout(), &schema_for!((BoardLock, Message))).expect("Cannot write json schema")
}
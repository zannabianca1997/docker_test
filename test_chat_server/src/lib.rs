use std::sync::Arc;

use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Local, NaiveDateTime, Utc};
use http::StatusCode;
use tokio::task::JoinHandle;
use tokio_postgres::{connect, Client, NoTls, Statement};

use common_types::{Board, Message, StoredMessage};

pub struct ServerStatus {
    started_at: DateTime<Utc>,
    title: String,

    client: Client,
    get_messages: Statement,
    send_message: Statement,
}
impl ServerStatus {
    async fn new(client: Client) -> Result<Arc<Self>, tokio_postgres::Error> {
        // first, activating the database
        // client.execute(r"\c messages", &[]).await?;

        // fetching the title
        let title = "TestTitle".to_string(); //TODO

        // preparing the queries
        let get_messages = client
            .prepare("SELECT time, \"user\", content FROM messages;")
            .await?;
        let send_message = client
            .prepare("INSERT INTO messages (time, \"user\", content) VALUES ($1, $2, $3);")
            .await?;

        Ok(Arc::new(Self {
            started_at: chrono::Local::now().to_utc(),

            title,

            client,
            get_messages,
            send_message,
        }))
    }

    async fn get_board(self: Arc<Self>) -> Result<Board, tokio_postgres::Error> {
        let messages = self
            .client
            .query(&self.get_messages, &[])
            .await?
            .into_iter()
            .map(|row| StoredMessage {
                time: row.get::<_, NaiveDateTime>("time").and_utc(),
                user: row.get("user"),
                content: row.get("content"),
            })
            .collect();
        Ok(Board {
            title: self.title.clone(),
            time: Local::now().to_utc(),
            started_at: self.started_at,
            messages,
        })
    }
}

async fn get_board(State(state): State<Arc<ServerStatus>>) -> Result<Json<Board>, StatusCode> {
    match state.get_board().await {
        Ok(board) => Ok(Json(board)),
        Err(err) => {
            tracing::error!("Error in obtaining board: {err}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn post_message(
    State(state): State<Arc<ServerStatus>>,
    Json(Message { user, content }): Json<Message>,
) -> StatusCode {
    if !is_valid_user(&user) || content.is_empty() {
        tracing::warn!("An invalid message reached the server");
        return StatusCode::BAD_REQUEST;
    }

    let ServerStatus {
        client,
        send_message,
        ..
    } = &*state;

    match client
        .execute(send_message, &[&Utc::now().naive_utc(), &user, &content])
        .await
    {
        Ok(1) => StatusCode::OK,
        Ok(n) => {
            tracing::error!(
                "Message inserting query affected strange number of row: {n} instead of 1"
            );
            StatusCode::INTERNAL_SERVER_ERROR
        }
        Err(err) => {
            tracing::error!("Error in obtaining board: {err}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

fn is_valid_user(user: &str) -> bool {
    !user.is_empty() && user.len() <= 32
}

pub async fn build(database: &str) -> Result<(Router, JoinHandle<()>), tokio_postgres::Error> {
    // obtain a connection to the database
    let (client, connection) = connect(database, NoTls).await?;

    let connection_closed = tokio::spawn(async {
        if let Err(err) = connection.await {
            tracing::error!("The connection closed with an error: {err}");
        }
    });

    let router = Router::new()
        .route("/", get(get_board))
        .route("/", post(post_message))
        .with_state(ServerStatus::new(client).await?);

    Ok((router, connection_closed))
}

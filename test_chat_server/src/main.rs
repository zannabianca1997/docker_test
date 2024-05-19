use std::env::{self, VarError};
use std::net::SocketAddr;
use std::time::Duration;

use axum::http::Method;
use clap::Parser;
use http::header::CONTENT_TYPE;
use tokio::signal;
use tower_http::cors::Any;
use tower_http::trace::TraceLayer;
use tower_http::{cors::CorsLayer, timeout::TimeoutLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Parser)]
pub struct Config {
    #[clap(long, short, default_value = "127.0.0.1:3000")]
    /// The server will listen to this address
    addr: SocketAddr,
    #[clap(long, short)]
    /// Address of the database
    database: Option<String>,
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

fn main() -> Result<(), ()> {
    // read configs
    let Config { database, addr } = Config::parse();

    // Enable tracing.
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "backend=debug,tower_http=debug,axum=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer().without_time())
        .init();

    let database = match database {
        Some(conn_string) => conn_string,
        None => match env::var("DB_CONN_STRING") {
            Ok(conn_string) => conn_string,
            Err(err) => {
                match err {
                        VarError::NotPresent => tracing::error!("Must provide a database connection string, either by param or by the env var DB_CONN_STRING"),
                        VarError::NotUnicode(_) => tracing::error!("The database connection string is invalid unicode"),
                    }
                return Err(());
            }
        },
    };

    let layers = (
        TraceLayer::new_for_http(),
        // Graceful shutdown will wait for outstanding requests to complete. Add a timeout so
        // requests don't hang forever.
        TimeoutLayer::new(Duration::from_secs(10)),
        // Accept CORS requests
        CorsLayer::new()
            // allow `GET` and `POST` when accessing the resource
            .allow_methods([Method::GET, Method::POST])
            // allow only JSON data
            .allow_headers([CONTENT_TYPE])
            // allow requests from any origin
            .allow_origin(Any),
    );

    // create the tokio runtime
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            // build our api
            let (app, connection_closed) = test_chat_server::build(&database)
                .await
                .map_err(|err| tracing::error!("Cannot build app: {err}"))?;
            // run our app with hyper
            let listener = tokio::net::TcpListener::bind(addr)
                .await
                .expect("failed to bind to address");

            // serve indefinetly
            axum::serve(listener, app.layer(layers))
                .with_graceful_shutdown(async move {
                    // stop either on shutdown or connection closed
                    tokio::select! {
                        _ = shutdown_signal() => {},
                        _ = connection_closed => {}
                    }
                })
                .await
                .map_err(|err| tracing::error!("runtime closed with error: {err}"))
        })
}

use std::{net::Ipv4Addr, time::Duration};

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
    #[clap(long, short, default_value_t = 3000)]
    /// The server will listen to this address
    port: u16,
    #[clap(long, short, default_value = "TestChat")]
    /// The title of the server
    title: String,
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

fn main() -> std::io::Result<()> {
    // read configs
    let Config { port, title } = Config::parse();

    // Enable tracing.
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                (if cfg!(debug_assertions) {
                    "backend=debug,tower_http=debug,axum=trace"
                } else {
                    "backend=warn,tower_http=warn,axum=warn"
                })
                .into()
            }),
        )
        .with(tracing_subscriber::fmt::layer().without_time())
        .init();

    // build our api
    let app = test_chat_server::build(title).layer((
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
    ));

    // create the tokio runtime
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            // run our app with hyper
            let listener = tokio::net::TcpListener::bind((Ipv4Addr::new(0, 0, 0, 0), port))
                .await
                .expect("failed to bind to address");

            // serve indefinetly
            axum::serve(listener, app)
                .with_graceful_shutdown(shutdown_signal())
                .await
        })
}

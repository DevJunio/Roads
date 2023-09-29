use std::{env, net::SocketAddr, num};

use axum::{http::StatusCode, Router, routing::get};
use thiserror::Error;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::router::path_routes;

mod router;

#[tokio::main]
async fn main() -> Result<(), ServerError> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "roads=trace, tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let client = redis::Client::open("redis://0.0.0.0:6379/")?;
    let con = client.get_connection()?;

    let router_param = path_routes(con);

    let router_svc = Router::new()
        .route("/ping", get(|| async { "Pong" }))
        .merge(router_param)
        .fallback(route_not_found);

    let port: u16 = env::var("PORT").unwrap_or_else(|_| "3000".into()).parse()?;
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    axum::Server::bind(&addr)
        .serve(router_svc.into_make_service())
        .await?;

    tracing::debug!("listening on {}", addr);
    Ok(())
}

async fn route_not_found() -> (StatusCode, String) {
    (
        StatusCode::NOT_FOUND,
        "Ops, this route doesn't exist!".to_string(),
    )
}

#[derive(Debug, Error)]
enum ServerError {
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Error while getting environment variables")]
    Var(#[from] env::VarError),

    #[error(transparent)]
    ParseError(#[from] num::ParseIntError),

    #[error(transparent)]
    HttpError(#[from] hyper::Error),
}

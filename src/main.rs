use std::{env, net::SocketAddr};

use anyhow::Result;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Router, routing::{get, post},
};
use hyper::body::HttpBody;
use sqlx::{PgPool, pool::PoolConnection};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use roads::router::path_routes;

pub type Db = PgPool;

pub struct DatabaseConnection(PoolConnection<'static, PgPool>);

#[async_trait]
impl<S> FromRequestParts<S> for DatabaseConnection
    where
        Db: FromRef<S>,
        S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let pool = Db::from_ref(state);
        let conn = pool.to_owned().await.map_err(internal_error)?;

        Ok(Self(conn))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "roads=trace, tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let pool =
        PgPool::connect(&env::var("DATABASE_URL").expect("env variable $DATABASE_URL needed"))
            .await?;

    let router_svc = Router::new()
        .route("/", get(redirect))
        .route("/ping", get(|| async { "Pong" }))
        .merge(path_routes)
        .route(
            "/*custom_path",
            post(add_route)
                .get(get_route),
        )
        .with_state(pool)
        .fallback(not_found);

    let env_port = std::env::var("PORT");
    let port: u16 = env_port.unwrap_or_else(|_| "3000".into()).parse().unwrap();

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    tracing::debug!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(router_svc.into_make_service())
        .await?;
    Ok(())
}

async fn redirect() -> impl IntoResponse {
    (
        Response::builder()
            .status(StatusCode::PERMANENT_REDIRECT)
            .header("Location", "https://bento.me/devjunio")
            .body(())
            .unwrap(),
        ("Redirecting to website..."),
    );
}

async fn not_found() -> (StatusCode, String) {
    (StatusCode::NOT_FOUND, "Ops, this route doesn't exist!".to_string())
}

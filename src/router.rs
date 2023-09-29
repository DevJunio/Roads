use std::sync::Arc;

use axum::{
    extract::{Path, State},
    Json,
    response::{IntoResponse, Response},
    Router, routing::get,
};
use hyper::{Body, StatusCode};
use redis::{Commands, Connection};
use serde::Deserialize;
use tokio::sync::Mutex;
use tracing::log::debug;

#[derive(Deserialize)]
struct Route {
    redirect_to: String,
}

type Db = Arc<Mutex<Connection>>;

// FIXME: add Patch, List and Delete actions
pub fn path_routes(con: Connection) -> Router {
    Router::new()
        .route("/*custom_path", get(get_route).post(add_route))
        .with_state(Arc::new(Mutex::new(con)))
}

async fn get_route(State(con): State<Db>, Path(user_path): Path<String>) -> impl IntoResponse {
    debug!("Getting key from route: {}", &user_path);
    let val: Result<String, _> = con.lock().await.get(user_path);

    // TODO: set this to show `val` correctly
    debug!("got value from route: {:#?}", &val);
    if val.is_err() {
        return Response::builder()
            .body(Body::empty())
            .map_err(internal_error);
    }

    Response::builder()
        .status(StatusCode::PERMANENT_REDIRECT)
        .header("Location", val.unwrap())
        .body(Body::empty())
        .map_err(internal_error)
}

async fn add_route(
    State(con): State<Db>,
    Path(custom_path): Path<String>,
    Json(req): Json<Route>,
) -> impl IntoResponse {
    let val: Result<String, _> = con.lock().await.set(&custom_path, req.redirect_to);

    if val.is_err() {
        return internal_error(val.err().unwrap());
    }

    debug!("inserted route: {}", &custom_path);
    (StatusCode::OK, "Route inserted".into())
}

/// Utility function for mapping any error into a `500 Internal Server Error`
/// response.
fn internal_error<E>(_err: E) -> (StatusCode, String)
    where
        E: std::error::Error,
{
    // TODO: add a debug log to print `err` value
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "unknown error has been reported".into(),
    )
}

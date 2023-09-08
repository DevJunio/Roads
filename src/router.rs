use axum::{
    debug_handler,
    extract::{Path, State},
    Json,
    response::{IntoResponse, Response},
    Router, routing::get,
};
use hyper::{Body, StatusCode};
use serde::Deserialize;

#[derive(Deserialize)]
struct Route {
    redirect_to: String,
}

pub fn path_routes() -> Router {
    Router::new()
        .route("/*custom_path",
               get(get_route)
                   .post(add_route)
                   .with_state(lol))
}

#[debug_handler]
async fn get_route(
    State(pool): State<Db>,
    Path(custom_path): Path<String>,
) -> impl IntoResponse {
    tracing::debug!("Getting key from route: {}", &custom_path);

    let thing = sqlx::query!(
r#"
SELECT redirect_to
FROM routes
WHERE route = $1
"#, &custom_path)
        .fetch_one(&pool)
        .await;

    tracing::debug!("got value from route: {:?}", &thing);

    if thing.is_err() {
        return Response::builder()
            .body(Body::empty())
            .map_err(internal_error);
    }

    Response::builder()
        .status(StatusCode::PERMANENT_REDIRECT)
        .header("Location", &thing.unwrap().redirect_to)
        .body(Body::empty())
        .map_err(internal_error)
}

#[debug_handler]
async fn add_route(
    State(pool): State<Db>,
    Path(custom_path): Path<String>,
    Json(req): Json<Route>,
) -> impl IntoResponse {
    let _ = sqlx::query!(
r#"
INSERT INTO routes ( route, redirect_to, id )
VALUES ( $1, $2, $3 )
"#,
        &custom_path,
        req.redirect_to,
        uuid::Uuid::new_v4()
    )
        .fetch_one(&pool)
        .await
        .map_err(internal_error);

    tracing::debug!("inserted route: {}", &custom_path);

    (StatusCode::OK, "Route added successfully!")
}

/// Utility function for mapping any error into a `500 Internal Server Error`
/// response.
fn internal_error<E>(err: E) -> (StatusCode, String)
    where
        E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

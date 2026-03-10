use axum::{
    Json, Router,
    body::Bytes,
    extract::OriginalUri,
    http::{Method, Request, StatusCode},
    response::IntoResponse,
    routing::get,
};

use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

use serde::Serialize;
use std::net::SocketAddr;
use tracing::info_span;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

async fn health() -> impl IntoResponse {
    return (StatusCode::OK, Json(HealthResponse { status: "ok" }));
}

async fn fallback_handler(
    method: Method,
    uri: OriginalUri,
    headers: axum::http::HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    eprintln!("--- INCOMING REQUEST ---");
    eprintln!("Method: {}", method);
    eprintln!("URI: {}", uri.0);

    eprintln!("Headers:");
    for (k, v) in headers.iter() {
        eprintln!("  {}: {:?}", k, v);
    }

    if !body.is_empty() {
        eprintln!("Body:\n{}", String::from_utf8_lossy(&body));
    }

    eprintln!("------------------------");

    return StatusCode::NOT_FOUND;
}

pub async fn start_http_server() {
    let addr = SocketAddr::from(([0, 0, 0, 0], 80));
    eprintln!("HTTP server listening on {}", addr);

    let app = Router::new()
        .route_service("/", ServeFile::new("ui/index.html"))
        .nest_service("/static", ServeDir::new("ui"))
        .route("/health", get(health))
        .fallback(fallback_handler)
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                info_span!(
                    "http_request",
                    method = ?request.method(),
                    // matched_path,
                    request_uri = request.uri().to_string(),
                    some_other_field = tracing::field::Empty,
                )
            }),
        );

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind HTTP server");

    return axum::serve(listener, app)
        .await
        .expect("HTTP server crashed");
}

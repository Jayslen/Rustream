use axum::{Router, routing};

mod routes;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/video/{id}", routing::get(routes::stream_route));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

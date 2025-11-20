use product_service::create_router;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let app = create_router();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8001")
        .await
        .unwrap();

    tracing::info!("listening on http://0.0.0.0:8001");

    axum::serve(listener, app).await.unwrap()
}

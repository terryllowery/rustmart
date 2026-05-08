use axum::{
    response::IntoResponse,
    extract::{Path, State},
    routing::get,
    Json, Router,
};

use shared::{ApiError, CreateProductRequest};
use sqlx::PgPool;
use uuid::Uuid;

use serde::{Deserialize, Serialize};

mod repository;
use repository::ProductRepository;
pub mod db;

#[derive(Clone)]
pub struct AppState {
    pub repo: ProductRepository,
}

#[derive(Serialize, Deserialize)]
struct HealthResponse {
    status: String,
    service: String,
}

pub fn create_router(pool: PgPool) -> Router {
    let repo = ProductRepository::new(pool);
    let state = AppState { repo };

    Router::new()
        .route("/health", get(health_check))
        .route("/products", get(get_products).post(create_product))
        .route("/products/:id", get(get_product_by_id).delete(delete_product))
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .with_state(state)
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "OK".to_string(),
        service: "product-service".to_string(),
    })
}

#[tracing::instrument(skip(state))]
async fn get_products(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    let products = state.repo.get_all().await?;
    Ok(Json(products))
}

#[tracing::instrument(skip(state))]
async fn get_product_by_id(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let product = state.repo.get_by_id(id).await?;
    Ok(Json(product))
}

#[tracing::instrument(skip(state))]
async fn create_product(
    State(state): State<AppState>,
    Json(req): Json<CreateProductRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let product = state.repo.create(req).await?;
    Ok((axum::http::StatusCode::CREATED, Json(product)))
}

#[tracing::instrument(skip(state))]
async fn delete_product(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    state.repo.delete(id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}



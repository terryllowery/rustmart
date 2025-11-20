use axum::{
    extract::Path,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use shared::{Product, ApiError};

#[derive(Serialize, Deserialize)]
struct HealthResponse {
    status: String,
    service: String,
}
pub fn create_router() -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/products", get(list_products))
        .route("/products/:id", get(get_product))
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "OK".to_string(),
        service: "product-service".to_string(),
    })
}

async fn list_products() -> Json<Vec<Product>> {
    // TODO: We are mocking data for now this needs to be real stuff

    let products = vec![
        Product {
            id: "1".to_string(),
            name: "Laptop".to_string(),
            description: "High-performance laptop".to_string(),
            price: 999.99,
            stock: 10,
        },
        Product {
            id: "2".to_string(),
            name: "Mouse".to_string(),
            description: "Wireless mouse".to_string(),
            price: 29.99,
            stock: 50,
        },
    ];

    Json(products)
}

async fn get_product(Path(id): Path<String>) -> Result<Json<Product>, ApiError> {
    println!("get product: {}", id);
    if id == "999" {
        return Err(ApiError::NotFound(format!("Product {} not found", id)));
    }
    // TODO: remove this mock and make it real
    Ok(Json(Product {
        id,
        name: "Laptop".to_string(),
        description: "High-performance laptop".to_string(),
        price: 999.99,
        stock: 10,
    }))
}



use chrono::{DateTime, Utc};
use rust_decimal;
use serde::{Deserialize, Serialize};
use sqlx;
use uuid;

#[derive(Serialize, Deserialize, Debug, Clone, sqlx::FromRow)]
pub struct Product {
    pub id: uuid::Uuid,
    pub name: String,
    pub price: rust_decimal::Decimal,
    pub inventory_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateProductRequest {
    pub name: String,
    pub price: f64,
    pub inventory_count: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub id: String,
    pub email: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrderItem {
    pub product_id: String,
    pub quantity: i32,
    pub price: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Order {
    pub id: String,
    pub user_id: String,
    pub items: Vec<OrderItem>,
    pub total: f64,
    pub status: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_product_creation() {
        let product = Product {
            id: Uuid::new_v4(),
            name: "Test Product".to_string(),
            price: rust_decimal::Decimal::new(9999, 2), // 99.99
            inventory_count: 10,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert_eq!(product.name, "Test Product");
        assert_eq!(product.price, rust_decimal::Decimal::new(9999, 2));
        assert_eq!(product.inventory_count, 10);
    }

    #[test]
    fn test_product_json_serialization() {
        let product = Product {
            id: Uuid::new_v4(),
            name: "Test Product".to_string(),
            price: rust_decimal::Decimal::new(9999, 2), // 99.99
            inventory_count: 10,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        // Serialize to JSON
        let json = serde_json::to_string(&product).unwrap();
        assert!(json.contains("Test Product"));

        // Deserialize from JSON
        let deserialized: Product = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, product.id);
    }
}

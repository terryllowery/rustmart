use serde::{Deserialize, Serialize };

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub description: String,
    pub price: f64,
    pub stock: i32,
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

    #[test]
    fn test_product_creation() {
        let product = Product {
            id: "1".to_string(),
            name: "Test Product".to_string(),
            description: "A Test Product".to_string(),
            price: 99.99,
            stock: 10,
        };

        assert_eq!(product.id, "1".to_string());
        assert_eq!(product.name, "Test Product".to_string());
        assert_eq!(product.description, "A Test Product".to_string());
        assert_eq!(product.price, 99.99);
        assert_eq!(product.stock, 10);
    }

    #[test]
    fn test_product_json_serialization() {
        let product = Product {
            id: "1".to_string(),
            name: "Test Product".to_string(),
            description: "A Test Product".to_string(),
            price: 99.99,
            stock: 10,
        };

        // Serialize to JSON
        let json = serde_json::to_string(&product).unwrap();
        assert!(json.contains("Test Product"));

        // Deserialize from JSON
        let deserialized: Product = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, product.id);

    }
}
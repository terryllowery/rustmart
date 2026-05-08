use shared::models::{Product, CreateProductRequest};
use shared::error::ApiError;
use sqlx::PgPool;
use uuid::Uuid;
use rust_decimal;


#[derive(Clone)]
pub struct ProductRepository {
    pool: PgPool,
}

impl ProductRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_all(&self) -> Result<Vec<Product>, ApiError> {
        let products = sqlx::query_as::<_, Product>("SELECT * FROM products")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| ApiError::Database(e.to_string()))?;
        Ok(products)
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_by_id(&self, id: Uuid) -> Result<Product, ApiError> {
        let product = sqlx::query_as::<_, Product>("SELECT * FROM products WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| ApiError::Database(e.to_string()))?
            .ok_or_else(|| ApiError::NotFound(format!("Product {} not found", id)))?;

        Ok(product)
    }

    #[tracing::instrument(skip(self))]
    pub async fn create(&self, req: CreateProductRequest ) -> Result<Product, ApiError> {
        let product = sqlx::query_as::<_, Product>(
            r#"
            INSERT INTO products (name, price, inventory_count)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
        )
            .bind(req.name)
            .bind(rust_decimal::Decimal::from_f64_retain(req.price).unwrap())
            .bind(req.inventory_count)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| ApiError::InternalServer(e.to_string()))?;
        Ok(product)
    }

    #[tracing::instrument(skip(self))]
    pub async fn delete(&self, id: Uuid) -> Result<(), ApiError> {
        let result = sqlx::query("DELETE FROM products WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| ApiError::InternalServer(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(ApiError::NotFound(format!("Product {} not found", id)));
        }

        Ok(())
    }
}
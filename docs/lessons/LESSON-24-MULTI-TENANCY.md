# Lesson 24: Multi-Tenancy Architecture

## Overview
Design and implement multi-tenant architecture for RustMart to support multiple organizations with data isolation, tenant-specific customization, and efficient resource sharing.

## Why This Matters
Multi-tenancy enables:
- **SaaS Business Model** - Serve multiple customers from single deployment
- **Cost Efficiency** - Share infrastructure across tenants
- **Scalability** - Add new tenants without new infrastructure
- **Customization** - Per-tenant features, branding, configuration

Common in B2B SaaS: Shopify (stores), Slack (workspaces), GitHub (organizations).

## Multi-Tenancy Models

### 1. Shared Database, Shared Schema (Highest Density)

**Structure**: Single database, `tenant_id` column in every table.

```sql
CREATE TABLE products (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    name TEXT NOT NULL,
    price NUMERIC NOT NULL,
    CONSTRAINT fk_tenant FOREIGN KEY (tenant_id) REFERENCES tenants(id)
);

CREATE INDEX idx_products_tenant ON products(tenant_id);
```

**Rust Implementation**:
```rust
#[derive(sqlx::FromRow)]
struct Product {
    id: Uuid,
    tenant_id: Uuid,
    name: String,
    price: Decimal,
}

async fn list_products(
    pool: &PgPool,
    tenant_id: Uuid,
) -> Result<Vec<Product>, sqlx::Error> {
    sqlx::query_as::<_, Product>(
        "SELECT * FROM products WHERE tenant_id = $1"
    )
    .bind(tenant_id)
    .fetch_all(pool)
    .await
}
```

**Pros**: Simplest, lowest cost, easy maintenance
**Cons**: Noisy neighbor risk, data isolation concerns, query complexity

### 2. Shared Database, Separate Schema (Balanced)

**Structure**: One database, one schema per tenant.

```sql
-- Tenant A
CREATE SCHEMA tenant_a;
CREATE TABLE tenant_a.products (...);

-- Tenant B
CREATE SCHEMA tenant_b;
CREATE TABLE tenant_b.products (...);
```

**Rust Router**:
```rust
struct SchemaRouter {
    pool: PgPool,
}

impl SchemaRouter {
    async fn with_tenant_schema<F, T>(&self, tenant_id: Uuid, f: F) -> Result<T, Error>
    where
        F: FnOnce(&PgPool) -> BoxFuture<'_, Result<T, Error>>,
    {
        let schema = format!("tenant_{}", tenant_id.simple());
        
        sqlx::query(&format!("SET search_path TO {}", schema))
            .execute(&self.pool)
            .await?;
        
        let result = f(&self.pool).await;
        
        // Reset
        sqlx::query("SET search_path TO public")
            .execute(&self.pool)
            .await?;
        
        result
    }
}
```

**Pros**: Better isolation, independent migrations possible
**Cons**: Schema management overhead, connection pooling complexity

### 3. Separate Database per Tenant (Maximum Isolation)

**Rust Connection Pool Manager**:
```rust
use dashmap::DashMap;

struct TenantDatabaseManager {
    pools: DashMap<Uuid, PgPool>,
    base_url: String,
}

impl TenantDatabaseManager {
    async fn get_pool(&self, tenant_id: Uuid) -> Result<PgPool, Error> {
        if let Some(pool) = self.pools.get(&tenant_id) {
            return Ok(pool.clone());
        }
        
        let db_name = format!("tenant_{}", tenant_id.simple());
        let url = format!("{}/{}", self.base_url, db_name);
        
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(&url)
            .await?;
        
        self.pools.insert(tenant_id, pool.clone());
        Ok(pool)
    }
}
```

**Pros**: Complete isolation, independent scaling, easy per-tenant backups
**Cons**: Higher cost, operational complexity

## Tenant Identification & Resolution

### Subdomain-Based Routing

```rust
use axum::{
    extract::{Host, Request},
    middleware::Next,
};

#[derive(Clone)]
struct TenantContext {
    tenant_id: Uuid,
    tenant_slug: String,
}

async fn tenant_middleware(
    Host(hostname): Host,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract subdomain: acme.rustmart.com -> "acme"
    let subdomain = hostname
        .split('.')
        .next()
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    // Lookup tenant
    let tenant = lookup_tenant_by_slug(subdomain)
        .await
        .ok_or(StatusCode::NOT_FOUND)?;
    
    // Inject tenant context
    req.extensions_mut().insert(TenantContext {
        tenant_id: tenant.id,
        tenant_slug: subdomain.to_string(),
    });
    
    Ok(next.run(req).await)
}
```

### Header-Based Identification

```rust
async fn tenant_from_header_middleware(
    headers: HeaderMap,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let tenant_id_str = headers
        .get("X-Tenant-ID")
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let tenant_id = Uuid::parse_str(tenant_id_str)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    req.extensions_mut().insert(TenantContext {
        tenant_id,
        tenant_slug: String::new(),
    });
    
    Ok(next.run(req).await)
}
```

### JWT Claims

```rust
#[derive(Deserialize)]
struct Claims {
    sub: String,
    tenant_id: Uuid,
    role: String,
    exp: usize,
}

async fn extract_tenant_from_jwt(
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
) -> Result<TenantContext, StatusCode> {
    let token = auth.token();
    
    let claims = decode::<Claims>(
        token,
        &DecodingKey::from_secret(SECRET.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| StatusCode::UNAUTHORIZED)?
    .claims;
    
    Ok(TenantContext {
        tenant_id: claims.tenant_id,
        tenant_slug: String::new(),
    })
}
```

## Data Isolation with Row-Level Security

### PostgreSQL RLS Setup

```sql
-- Enable RLS
ALTER TABLE products ENABLE ROW LEVEL SECURITY;

-- Create policy
CREATE POLICY tenant_isolation ON products
    USING (tenant_id = current_setting('app.current_tenant')::uuid);

-- Grant access
GRANT ALL ON products TO app_user;
```

### Rust with RLS

```rust
async fn set_tenant_context(pool: &PgPool, tenant_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT set_config('app.current_tenant', $1, false)")
        .bind(tenant_id.to_string())
        .execute(pool)
        .await?;
    Ok(())
}

async fn list_products_with_rls(
    pool: &PgPool,
    tenant_id: Uuid,
) -> Result<Vec<Product>, Error> {
    // Set tenant context
    set_tenant_context(pool, tenant_id).await?;
    
    // Query without WHERE clause - RLS enforces isolation
    let products = sqlx::query_as::<_, Product>("SELECT * FROM products")
        .fetch_all(pool)
        .await?;
    
    Ok(products)
}
```

## Tenant-Specific Configuration

### Configuration Storage

```sql
CREATE TABLE tenant_config (
    tenant_id UUID PRIMARY KEY,
    features JSONB NOT NULL DEFAULT '{}',
    settings JSONB NOT NULL DEFAULT '{}',
    branding JSONB NOT NULL DEFAULT '{}',
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

### Rust Configuration Management

```rust
#[derive(Serialize, Deserialize)]
struct TenantConfig {
    features: HashMap<String, bool>,
    settings: HashMap<String, serde_json::Value>,
    branding: BrandingConfig,
}

#[derive(Serialize, Deserialize)]
struct BrandingConfig {
    logo_url: Option<String>,
    primary_color: String,
    company_name: String,
}

async fn get_tenant_config(
    pool: &PgPool,
    tenant_id: Uuid,
) -> Result<TenantConfig, Error> {
    let row = sqlx::query!(
        "SELECT features, settings, branding FROM tenant_config WHERE tenant_id = $1",
        tenant_id
    )
    .fetch_one(pool)
    .await?;
    
    Ok(TenantConfig {
        features: serde_json::from_value(row.features)?,
        settings: serde_json::from_value(row.settings)?,
        branding: serde_json::from_value(row.branding)?,
    })
}

// Feature flag check
async fn feature_enabled(
    pool: &PgPool,
    tenant_id: Uuid,
    feature: &str,
) -> Result<bool, Error> {
    let config = get_tenant_config(pool, tenant_id).await?;
    Ok(config.features.get(feature).copied().unwrap_or(false))
}
```

## Rate Limiting Per Tenant

### Redis-Based Rate Limiter

```rust
use redis::AsyncCommands;

struct TenantRateLimiter {
    redis: redis::Client,
}

impl TenantRateLimiter {
    async fn check_rate_limit(
        &self,
        tenant_id: Uuid,
        limit: u32,
        window_secs: u64,
    ) -> Result<bool, Error> {
        let mut conn = self.redis.get_async_connection().await?;
        let key = format!("ratelimit:{}:{}", tenant_id, window_secs);
        
        let count: u32 = conn.incr(&key, 1).await?;
        
        if count == 1 {
            conn.expire(&key, window_secs as usize).await?;
        }
        
        Ok(count <= limit)
    }
}

// Middleware
async fn rate_limit_middleware(
    Extension(limiter): Extension<TenantRateLimiter>,
    Extension(tenant): Extension<TenantContext>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let allowed = limiter
        .check_rate_limit(tenant.tenant_id, 1000, 60)  // 1000 req/min
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    if !allowed {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }
    
    Ok(next.run(req).await)
}
```

## Tenant Provisioning

### Automated Onboarding

```rust
async fn provision_tenant(
    pool: &PgPool,
    req: CreateTenantRequest,
) -> Result<Tenant, Error> {
    let mut tx = pool.begin().await?;
    
    // 1. Create tenant record
    let tenant_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO tenants (id, slug, company_name, created_at) VALUES ($1, $2, $3, now())",
        tenant_id,
        req.slug,
        req.company_name
    )
    .execute(&mut *tx)
    .await?;
    
    // 2. Initialize configuration
    sqlx::query!(
        "INSERT INTO tenant_config (tenant_id, features, settings, branding) VALUES ($1, $2, $3, $4)",
        tenant_id,
        json!({"advanced_analytics": false}),
        json!({"max_users": 10}),
        json!({"company_name": req.company_name, "primary_color": "#3B82F6"})
    )
    .execute(&mut *tx)
    .await?;
    
    // 3. Create admin user
    let admin_id = Uuid::new_v4();
    let password_hash = hash_password(&req.admin_password)?;
    sqlx::query!(
        "INSERT INTO users (id, tenant_id, email, password_hash, role) VALUES ($1, $2, $3, $4, 'admin')",
        admin_id,
        tenant_id,
        req.admin_email,
        password_hash
    )
    .execute(&mut *tx)
    .await?;
    
    tx.commit().await?;
    
    // 4. Send welcome email (async)
    tokio::spawn(send_welcome_email(req.admin_email.clone(), tenant_id));
    
    Ok(Tenant {
        id: tenant_id,
        slug: req.slug,
        company_name: req.company_name,
    })
}
```

## Best Practices

- **Validate Tenant Context** - Check on every request
- **Use RLS** - Database-level isolation reduces bugs
- **Monitor Per-Tenant** - Track usage, errors, latency by tenant
- **Test Isolation** - Verify cross-tenant data leakage impossible
- **Plan for Scale** - Design sharding strategy early
- **Document Onboarding** - Clear tenant provisioning process
- **Implement Quotas** - Prevent resource abuse

## Official Documentation

- [PostgreSQL Row-Level Security](https://www.postgresql.org/docs/current/ddl-rowsecurity.html)
- [AWS Multi-Tenancy](https://docs.aws.amazon.com/whitepapers/latest/saas-architecture-fundamentals/tenant-isolation.html)
- [Multi-Tenancy Patterns](https://docs.microsoft.com/en-us/azure/architecture/guide/multitenant/overview)

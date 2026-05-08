# Lesson 20: Security Hardening

## Overview
Secure RustMart against common attacks using OWASP Top 10 guidelines, implement proper authentication/authorization, manage secrets securely, and harden containers for production deployment.

## OWASP Top 10 for Microservices

### 1. Broken Authentication
```rust
// ❌ BAD: Weak JWT validation
fn decode_jwt_weak(token: &str) -> Claims {
    decode(token, &DecodingKey::from_secret(b"secret"), &Validation::default())
}

// ✅ GOOD: Strong validation
fn decode_jwt_secure(token: &str) -> Result<Claims, JwtError> {
    let mut validation = Validation::new(Algorithm::HS512);
    validation.set_audience(&["rustmart-api"]);
    validation.set_issuer(&["rustmart-auth"]);
    validation.leeway = 0; // No clock skew tolerance
    
    decode(token, &DecodingKey::from_secret(SECRET.as_bytes()), &validation)
}
```

### 2. SQL Injection Prevention
```rust
// ❌ DANGEROUS: String concatenation
let query = format!("SELECT * FROM products WHERE name = '{}'", user_input);

// ✅ SAFE: Parameterized queries
sqlx::query_as::<_, Product>("SELECT * FROM products WHERE name = $1")
    .bind(user_input)
    .fetch_all(&pool)
    .await?
```

### 3. Input Validation
```rust
use validator::Validate;

#[derive(Deserialize, Validate)]
struct CreateProductRequest {
    #[validate(length(min = 1, max = 255))]
    name: String,
    
    #[validate(range(min = 0.01, max = 1000000.0))]
    price: f64,
    
    #[validate(range(min = 0, max = 1000000))]
    inventory_count: i32,
}

async fn create_product(Json(req): Json<CreateProductRequest>) -> Result<Json<Product>, ApiError> {
    req.validate().map_err(|e| ApiError::BadRequest(e.to_string()))?;
    // Process validated input...
}
```

### 4. Rate Limiting
```rust
use governor::{Quota, RateLimiter};
use std::num::NonZeroU32;

// 100 requests per minute per IP
let limiter = RateLimiter::direct(
    Quota::per_minute(NonZeroU32::new(100).unwrap())
);

async fn rate_limit_middleware(req: Request, next: Next) -> Result<Response, StatusCode> {
    let ip = extract_ip(&req);
    
    if limiter.check_key(&ip).is_err() {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }
    
    Ok(next.run(req).await)
}
```

### 5. Secure Headers
```rust
use tower_http::set_header::SetResponseHeaderLayer;

Router::new()
    .layer(SetResponseHeaderLayer::if_not_present(
        header::X_FRAME_OPTIONS,
        HeaderValue::from_static("DENY"),
    ))
    .layer(SetResponseHeaderLayer::if_not_present(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    ))
    .layer(SetResponseHeaderLayer::if_not_present(
        header::STRICT_TRANSPORT_SECURITY,
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    ))
```

## Secrets Management

### Using Environment Variables (Development)
```rust
use std::env;

fn get_secret(key: &str) -> Result<String, Error> {
    env::var(key).map_err(|_| Error::MissingSecret(key.to_string()))
}

let jwt_secret = get_secret("JWT_SECRET")?;
let db_password = get_secret("DATABASE_PASSWORD")?;
```

### Using HashiCorp Vault (Production)
```rust
use vaultrs::client::{VaultClient, VaultClientSettingsBuilder};

async fn fetch_secrets() -> Result<Secrets, Error> {
    let client = VaultClient::new(
        VaultClientSettingsBuilder::default()
            .address("https://vault.company.com")
            .token(env::var("VAULT_TOKEN")?)
            .build()?
    )?;
    
    let secret = vaultrs::kv2::read(&client, "rustmart", "database").await?;
    
    Ok(Secrets {
        db_password: secret["password"].as_str().unwrap().to_string(),
        jwt_secret: secret["jwt_secret"].as_str().unwrap().to_string(),
    })
}
```

### Using AWS Secrets Manager
```rust
use aws_sdk_secretsmanager::Client;

async fn get_aws_secret(secret_name: &str) -> Result<String, Error> {
    let config = aws_config::load_from_env().await;
    let client = Client::new(&config);
    
    let response = client
        .get_secret_value()
        .secret_id(secret_name)
        .send()
        .await?;
    
    Ok(response.secret_string().unwrap().to_string())
}
```

## Container Security

### Multi-stage Dockerfile with Security
```dockerfile
# Build stage
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

# Runtime stage - minimal and secure
FROM debian:bookworm-slim

# Create non-root user
RUN useradd -m -u 1000 rustmart && \
    apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary
COPY --from=builder /app/target/release/product-service /app/

# Change ownership
RUN chown -R rustmart:rustmart /app

# Switch to non-root user
USER rustmart

# Drop capabilities
RUN setcap cap_net_bind_service=+ep /app/product-service

EXPOSE 8001

CMD ["/app/product-service"]
```

### Security Scanning
```bash
# Scan Docker image for vulnerabilities
docker scan rustmart/product-service:latest

# Use trivy for comprehensive scanning
trivy image rustmart/product-service:latest

# Scan dependencies for known vulnerabilities
cargo audit
```

## Authentication & Authorization

### Role-Based Access Control (RBAC)
```rust
#[derive(Debug, Clone)]
enum Role {
    Admin,
    Manager,
    Customer,
}

#[derive(Debug, Clone)]
struct Claims {
    sub: String,
    email: String,
    role: Role,
    exp: usize,
}

fn require_role(required: Role) -> impl Fn(Request, Next) -> Future<Output = Result<Response, StatusCode>> {
    move |req: Request, next: Next| async move {
        let claims = req.extensions().get::<Claims>().ok_or(StatusCode::UNAUTHORIZED)?;
        
        if !has_permission(&claims.role, &required) {
            return Err(StatusCode::FORBIDDEN);
        }
        
        Ok(next.run(req).await)
    }
}

// Usage
Router::new()
    .route("/admin/users", get(list_users))
    .layer(middleware::from_fn(require_role(Role::Admin)))
```

## Network Security

### TLS Configuration
```rust
use axum_server::tls_rustls::RustlsConfig;

#[tokio::main]
async fn main() {
    let config = RustlsConfig::from_pem_file("cert.pem", "key.pem").await?;
    
    axum_server::bind_rustls("0.0.0.0:443".parse()?, config)
        .serve(app.into_make_service())
        .await?;
}
```

### CORS Configuration
```rust
use tower_http::cors::{CorsLayer, Any};

let cors = CorsLayer::new()
    .allow_origin("https://rustmart.com".parse::<HeaderValue>()?)
    .allow_methods([Method::GET, Method::POST])
    .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
    .max_age(Duration::from_secs(3600));

Router::new().layer(cors)
```

## Kubernetes Security

### Pod Security Policy
```yaml
apiVersion: policy/v1beta1
kind: PodSecurityPolicy
metadata:
  name: rustmart-psp
spec:
  privileged: false
  allowPrivilegeEscalation: false
  requiredDropCapabilities:
    - ALL
  runAsUser:
    rule: MustRunAsNonRoot
  seLinux:
    rule: RunAsAny
  fsGroup:
    rule: RunAsAny
  volumes:
    - 'configMap'
    - 'secret'
```

### Network Policies
```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: product-service-netpol
spec:
  podSelector:
    matchLabels:
      app: product-service
  policyTypes:
    - Ingress
    - Egress
  ingress:
    - from:
        - podSelector:
            matchLabels:
              app: api-gateway
      ports:
        - protocol: TCP
          port: 8001
  egress:
    - to:
        - podSelector:
            matchLabels:
              app: postgres
      ports:
        - protocol: TCP
          port: 5432
```

## Security Checklist

- [ ] All inputs validated
- [ ] SQL injection prevented (parameterized queries)
- [ ] Authentication implemented (JWT)
- [ ] Authorization implemented (RBAC)
- [ ] Rate limiting configured
- [ ] Secrets in vault (not env vars)
- [ ] TLS/HTTPS enabled
- [ ] Security headers set
- [ ] Container runs as non-root
- [ ] Container scanned for vulnerabilities
- [ ] Dependencies audited (`cargo audit`)
- [ ] Network policies configured
- [ ] CORS properly configured
- [ ] Sensitive data encrypted at rest
- [ ] Audit logging enabled

## Official Documentation

- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [Rust Secure Code](https://anssi-fr.github.io/rust-guide/)
- [cargo-audit](https://github.com/rustsec/rustsec)
- [HashiCorp Vault](https://www.vaultproject.io/)

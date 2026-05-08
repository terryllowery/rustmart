use opentelemetry::global;
use opentelemetry::trace::TracerProvider as _; // Trait for .tracer() method
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig; // Trait for .with_endpoint()
use opentelemetry_sdk::Resource;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
// use std::time::Duration;
// use std::net::TcpStream;

//TODO: Implement endpoint reachability check
// fn is_endpoint_reachable(endpoint: &str, timeout_ms: u64) -> bool {
//     // let addr = endpoint
//     //     .replace("http://", "")
//     //     .replace("https://", "");
//     let addr = match endpoint {
//         endpoint if endpoint.contains("http://") => endpoint.replace("http://", ""),
//         endpoint if endpoint.contains("https://") => endpoint.replace("https://", ""),
//         _ => return false,
//     };
//     match addr.parse() {
//         Ok(socket_addr) => {
//             TcpStream::connect_timeout(&socket_addr, Duration::from_millis(timeout_ms)).is_ok()
//         }
//         Err(_) => false,
//     }
//
// }

#[tokio::main]
async fn main() {
    // Get backend configuration
    let backend = std::env::var("TRACING_BACKEND").unwrap_or_else(|_| "console".to_string());

    // Initialize tracing based on backend (case-insensitive)
    // Falls back to simple tracing if OpenTelemetry fails
    match backend.as_str() {
        "jaeger" | "JAEGER" | "Jaeger" => {
            match opentelemetry_otlp::new_pipeline()
                .tracing()
                .with_exporter(
                    opentelemetry_otlp::new_exporter().tonic().with_endpoint(
                        std::env::var("JAEGER_ENDPOINT")
                            .unwrap_or_else(|_| "http://localhost:4317".to_string()),
                    ),
                )
                .with_trace_config(
                    opentelemetry_sdk::trace::config().with_resource(Resource::new(vec![
                        KeyValue::new("service.name", "product-service"),
                        KeyValue::new("service.version", "0.1.0"),
                    ])),
                )
                .install_batch(opentelemetry_sdk::runtime::Tokio)
            {
                Ok(tracer) => {
                    // Initialize tracing subscriber with OpenTelemetry
                    if tracing_subscriber::registry()
                        .with(tracing_subscriber::EnvFilter::new(
                            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
                        ))
                        .with(tracing_subscriber::fmt::layer())
                        .with(tracing_opentelemetry::layer().with_tracer(tracer))
                        .try_init()
                        .is_err()
                    {
                        eprintln!("Warning: Failed to initialize tracing subscriber");
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to initialize Jaeger tracer: {}. Falling back to simple logging.", e);
                    // Fallback to simple fmt logging
                    let _ = tracing_subscriber::fmt()
                        .with_env_filter(
                            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
                        )
                        .try_init();
                }
            }
        }
        "console" | "CONSOLE" | "Console" | _ => {
            // Console with OpenTelemetry stdout exporter (prints JSON traces)
            let tracer = opentelemetry_sdk::trace::TracerProvider::builder()
                .with_simple_exporter(opentelemetry_stdout::SpanExporter::default())
                .with_config(
                    opentelemetry_sdk::trace::config().with_resource(Resource::new(vec![
                        KeyValue::new("service.name", "product-service"),
                        KeyValue::new("service.version", "0.1.0"),
                    ])),
                )
                .build();

            let tracer = tracer.tracer("product-service");

            // Initialize tracing subscriber with OpenTelemetry
            if tracing_subscriber::registry()
                .with(tracing_subscriber::EnvFilter::new(
                    std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
                ))
                .with(tracing_subscriber::fmt::layer())
                .with(tracing_opentelemetry::layer().with_tracer(tracer))
                .try_init()
                .is_err()
            {
                eprintln!("Warning: Failed to initialize tracing subscriber. Using fallback.");
                // Fallback to simple fmt logging
                let _ = tracing_subscriber::fmt()
                    .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()))
                    .try_init();
            }
        }
    }

    tracing::info!(backend = %backend, "Starting product-service");

    // Connect to PostgreSQL
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/rustmart".to_string());
    let pool = product_service::db::create_pool(&database_url)
        .await
        .expect("Failed to connect to database");

    // Create the Axum router from lib.rs
    let app = product_service::create_router(pool);

    // Bind and serve
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8001")
        .await
        .expect("Failed to bind to port 8001");

    tracing::info!("Server listening on http://127.0.0.1:8001");

    axum::serve(listener, app).await.expect("Server error");

    // Shutdown tracer on exit
    global::shutdown_tracer_provider();
}

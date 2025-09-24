//! Comprehensive tracing utilities for observability and monitoring
//! 
//! This module provides centralized tracing configuration and HTTP request instrumentation
//! to enhance observability across the auth service.

use std::time::Duration;
use http::{Request, Response};
use hyper::Body;
use tracing::{Level, Span};
use color_eyre::eyre::Result;
use tracing_error::ErrorLayer;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, EnvFilter};

/// Initializes the tracing subscriber with environment-aware configuration and error reporting
/// 
/// This function sets up comprehensive tracing for the application with:
/// - Environment-based filtering
/// - Compact format for better readability
/// - Error layer for enhanced error reporting with span traces
/// - Registry-based subscriber for multiple layers
pub fn init_tracing() -> Result<()> {
    // Create a formatting layer for tracing output with a compact format
    let fmt_layer = fmt::layer().compact();

    // Create a filter layer to control the verbosity of logs
    // Try to get the filter configuration from the environment variables
    // If it fails, default to the "info" log level
    let filter_layer = EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info"))?;

    // Build the tracing subscriber registry with the formatting layer,
    // the filter layer, and the error layer for enhanced error reporting
    tracing_subscriber::registry()
        .with(filter_layer) // Add the filter layer to control log verbosity
        .with(fmt_layer) // Add the formatting layer for compact log output
        .with(ErrorLayer::default()) // Add the error layer to capture error contexts
        .init(); // Initialize the tracing subscriber

    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        environment = %std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()),
        "🔍 Tracing initialized successfully with error reporting"
    );

    Ok(())
}

/// Creates a new tracing span with a unique request ID for each incoming HTTP request
/// 
/// This function generates comprehensive request tracking by creating spans that include:
/// - Unique request correlation ID (UUID v4)
/// - HTTP method, URI, and version
/// - User agent and client IP (when available)
/// - Request start timestamp
/// 
/// # Arguments
/// 
/// * `request` - The incoming HTTP request to instrument
/// 
/// # Returns
/// 
/// A new `Span` configured for request tracking
/// 
/// # Examples
/// 
/// ```rust
/// use auth_service::utils::tracing::make_span_with_request_id;
/// use http::Request;
/// use hyper::Body;
/// 
/// let request = Request::builder()
///     .method("GET")
///     .uri("/test")
///     .body(Body::empty())
///     .unwrap();
/// let span = make_span_with_request_id(&request);
/// // Span contains request_id, method, uri, etc.
/// ```
pub fn make_span_with_request_id(request: &Request<Body>) -> Span {
    let request_id = uuid::Uuid::new_v4();
    
    // Extract additional request metadata for enhanced observability
    let user_agent = request
        .headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");
    
    let forwarded_for = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .or_else(|| {
            request
                .headers()
                .get("x-real-ip")
                .and_then(|h| h.to_str().ok())
        })
        .unwrap_or("unknown");

    tracing::span!(
        Level::INFO,
        "🌐 [HTTP REQUEST]",
        request_id = %request_id,
        method = %request.method(),
        uri = %request.uri(),
        version = ?request.version(),
        user_agent = %user_agent,
        client_ip = %forwarded_for,
        // Fields to be filled later
        status_code = tracing::field::Empty,
        latency_ms = tracing::field::Empty,
        response_size = tracing::field::Empty,
    )
}

/// Logs an event indicating the start of an HTTP request
/// 
/// This function creates a structured log entry when a request begins processing,
/// providing visibility into request flow and timing.
/// 
/// # Arguments
/// 
/// * `request` - The HTTP request being processed
/// * `span` - The tracing span associated with this request
pub fn on_request(request: &Request<Body>, _span: &Span) {
    let content_length = request
        .headers()
        .get("content-length")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("0");

    tracing::event!(
        Level::INFO,
        content_length = %content_length,
        "📥 Request processing started"
    );
}

/// Logs an event indicating the completion of an HTTP request
/// 
/// This function creates comprehensive logging for request completion, including:
/// - Response status code and classification
/// - Request latency in milliseconds
/// - Error-level logging for 4xx/5xx responses
/// - Performance warnings for slow requests
/// 
/// # Arguments
/// 
/// * `response` - The HTTP response being sent
/// * `latency` - The total request processing time
/// * `span` - The tracing span associated with this request
pub fn on_response(response: &Response<Body>, latency: Duration, _span: &Span) {
    let status = response.status();
    let status_code = status.as_u16();
    let status_code_class = status_code / 100;
    let latency_ms = latency.as_millis();
    
    // Extract response size if available
    let response_size = response
        .headers()
        .get("content-length")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");

    // Determine log level and message based on response status
    match status_code_class {
        4 => {
            tracing::event!(
                Level::WARN,
                status_code = %status_code,
                status_text = %status.canonical_reason().unwrap_or("Unknown"),
                latency_ms = %latency_ms,
                response_size = %response_size,
                "⚠️  Client error response"
            );
        }
        5 => {
            tracing::event!(
                Level::ERROR,
                status_code = %status_code,
                status_text = %status.canonical_reason().unwrap_or("Unknown"),
                latency_ms = %latency_ms,
                response_size = %response_size,
                "❌ Server error response"
            );
        }
        _ => {
            // Check for slow requests (>1 second)
            if latency_ms > 1000 {
                tracing::event!(
                    Level::WARN,
                    status_code = %status_code,
                    latency_ms = %latency_ms,
                    response_size = %response_size,
                    "🐌 Slow request detected"
                );
            } else {
                tracing::event!(
                    Level::INFO,
                    status_code = %status_code,
                    latency_ms = %latency_ms,
                    response_size = %response_size,
                    "✅ Request completed successfully"
                );
            }
        }
    }
}

/// Creates a child span for database operations with enhanced metadata
/// 
/// This helper function creates specialized spans for database operations,
/// helping track query performance and database-related issues.
/// 
/// # Arguments
/// 
/// * `operation` - The type of database operation (e.g., "SELECT", "INSERT")
/// * `table` - The database table being accessed
/// 
/// # Returns
/// 
/// A new `Span` configured for database operation tracking
pub fn database_span(operation: &str, table: &str) -> Span {
    tracing::span!(
        Level::DEBUG,
        "🗄️  [DATABASE]",
        operation = %operation,
        table = %table,
        rows_affected = tracing::field::Empty,
        query_time_ms = tracing::field::Empty,
    )
}

/// Creates a child span for authentication operations
/// 
/// This helper function creates specialized spans for auth-related operations,
/// helping track authentication flow and security events.
/// 
/// # Arguments
/// 
/// * `operation` - The type of auth operation (e.g., "login", "token_validation")
/// 
/// # Returns
/// 
/// A new `Span` configured for authentication operation tracking
pub fn auth_span(operation: &str) -> Span {
    tracing::span!(
        Level::INFO,
        "🔐 [AUTH]",
        operation = %operation,
        user_id = tracing::field::Empty,
        success = tracing::field::Empty,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::{Method, Version};
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn ensure_test_tracing() {
        INIT.call_once(|| {
            let _ = tracing_subscriber::fmt()
                .with_test_writer()
                .with_max_level(tracing::Level::DEBUG)
                .try_init();
        });
    }

    #[test]
    fn test_make_span_with_request_id_creates_valid_span() {
        ensure_test_tracing();
        
        let request = Request::builder()
            .method(Method::GET)
            .uri("/test")
            .version(Version::HTTP_11)
            .header("user-agent", "test-agent")
            .header("x-forwarded-for", "192.168.1.1")
            .body(Body::empty())
            .unwrap();

        let span = make_span_with_request_id(&request);
        
        // Verify span is created and has correct metadata
        assert!(span.metadata().is_some());
        assert_eq!(span.metadata().unwrap().level(), &Level::INFO);
    }

    #[test]
    fn test_database_span_creation() {
        ensure_test_tracing();
        
        let span = database_span("SELECT", "users");
        assert!(span.metadata().is_some());
        assert_eq!(span.metadata().unwrap().level(), &Level::DEBUG);
    }

    #[test]
    fn test_auth_span_creation() {
        ensure_test_tracing();
        
        let span = auth_span("login");
        assert!(span.metadata().is_some());
        assert_eq!(span.metadata().unwrap().level(), &Level::INFO);
    }
}
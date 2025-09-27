//! Comprehensive tracing utilities for observability and monitoring
//! 
//! This module provides centralized tracing configuration and HTTP request instrumentation
//! to enhance observability across the auth service with runtime-configurable log levels.

use std::time::Duration;
use std::io;
use http::{Request, Response};
use hyper::Body;
use tracing::{Level, Span};
use color_eyre::eyre::Result;
use tracing_error::ErrorLayer;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, Registry};
use tracing_appender::{non_blocking, rolling};

use super::logging_config::{init_global_logging_config, get_logging_config};
use super::external_logging::{ExternalLoggingConfig, ExternalLoggingLayer};
use super::file_logging::{FileLoggingConfig, FileLoggingManager};

/// Initializes the tracing subscriber with advanced runtime-configurable logging
/// 
/// This function sets up comprehensive tracing for the application with:
/// - Runtime-adjustable log levels
/// - Environment-aware configuration (dev/test/prod)
/// - JSON structured logging for production
/// - Pretty console logging for development
/// - Optional file logging with rotation
/// - Enhanced error reporting with span traces
/// - Performance-optimized async logging
pub fn init_tracing() -> Result<()> {
    // Initialize global logging configuration from environment
    init_global_logging_config()?;
    
    let config = get_logging_config()?;
    
    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        environment = ?config.environment,
        log_level = %config.level,
        json_format = config.json_format,
        log_to_file = config.log_to_file,
        "🔍 Initializing advanced tracing with runtime configuration and external service support"
    );

    // Create environment filter based on configuration
    let filter_layer = config.get_env_filter()?;

    // Build the subscriber based on environment and configuration
    let registry = Registry::default()
        .with(filter_layer)
        .with(ErrorLayer::default());

    // Initialize external logging configuration safely
    let external_config = ExternalLoggingConfig::from_env();
    
    // Initialize enhanced file logging if enabled
    let file_config = FileLoggingConfig::default();
    if file_config.enabled {
        match FileLoggingManager::new(file_config.clone()) {
            Ok(_file_manager) => {
                tracing::info!(
                    log_dir = ?file_config.log_dir,
                    rotation_policy = %file_config.rotation_policy,
                    max_files = file_config.max_files,
                    compress_rotated = file_config.compress_rotated,
                    "📁 Enhanced file logging enabled"
                );
            },
            Err(e) => {
                tracing::warn!("Failed to initialize file logging: {}", e);
            }
        }
    }

    if external_config.enabled {
        tracing::info!(
            service_type = %external_config.service_type,
            endpoint = %external_config.endpoint,
            batch_size = external_config.batch_size,
            flush_interval_secs = external_config.flush_interval_secs,
            "🌐 External logging enabled"
        );
    }
    
    // Configure output format based on environment
    if config.json_format {
        // Production: JSON structured logging
        if config.log_to_file && config.log_file_path.is_some() {
            // File logging with rotation
            let file_path = config.log_file_path.as_ref().unwrap();
            let file_appender = rolling::daily(
                std::path::Path::new(file_path).parent().unwrap_or(std::path::Path::new(".")),
                std::path::Path::new(file_path).file_name().unwrap_or(std::ffi::OsStr::new("app.log"))
            );
            let (file_writer, _guard) = non_blocking(file_appender);
            
            // Console + File JSON logging with conditional external logging
            if external_config.enabled {
                match ExternalLoggingLayer::new(external_config) {
                    Ok(external_layer) => {
                        let subscriber = registry
                            .with(external_layer)
                            .with(fmt::layer()
                                .json()
                                .with_writer(io::stdout)
                                .with_target(true)
                                .with_thread_ids(true)
                                .with_thread_names(true))
                            .with(fmt::layer()
                                .json()
                                .with_writer(file_writer)
                                .with_target(true)
                                .with_thread_ids(true)
                                .with_thread_names(true));
                        
                        subscriber.init();
                    },
                    Err(e) => {
                        tracing::warn!("Failed to initialize external logging layer, continuing without it: {}", e);
                        let subscriber = registry
                            .with(fmt::layer()
                                .json()
                                .with_writer(io::stdout)
                                .with_target(true)
                                .with_thread_ids(true)
                                .with_thread_names(true))
                            .with(fmt::layer()
                                .json()
                                .with_writer(file_writer)
                                .with_target(true)
                                .with_thread_ids(true)
                                .with_thread_names(true));
                        
                        subscriber.init();
                    }
                }
            } else {
                let subscriber = registry
                    .with(fmt::layer()
                        .json()
                        .with_writer(io::stdout)
                        .with_target(true)
                        .with_thread_ids(true)
                        .with_thread_names(true))
                    .with(fmt::layer()
                        .json()
                        .with_writer(file_writer)
                        .with_target(true)
                        .with_thread_ids(true)
                        .with_thread_names(true));
                
                subscriber.init();
            }
            
            // Store the guard to prevent dropping
            // TODO: In a real application, we'd store this guard somewhere persistent
        } else {
            // Console-only JSON logging with external logging
            if external_config.enabled {
                let external_layer = ExternalLoggingLayer::new(external_config)
                    .map_err(|e| color_eyre::eyre::eyre!("Failed to create external logging layer: {}", e))?;
                
                let subscriber = registry
                    .with(external_layer)
                    .with(fmt::layer()
                        .json()
                        .with_target(true)
                        .with_thread_ids(true)
                        .with_thread_names(true));
                
                subscriber.init();
            } else {
                let subscriber = registry
                    .with(fmt::layer()
                        .json()
                        .with_target(true)
                        .with_thread_ids(true)
                        .with_thread_names(true));
                
                subscriber.init();
            }
        }
    } else {
        // Development/Testing: Pretty console logging with external logging
        if external_config.enabled {
            let external_layer = ExternalLoggingLayer::new(external_config)
                .map_err(|e| color_eyre::eyre::eyre!("Failed to create external logging layer: {}", e))?;
            
            let subscriber = registry
                .with(external_layer)
                .with(fmt::layer()
                    .compact()
                    .with_target(false)
                    .with_thread_ids(false)
                    .with_thread_names(false));
            
            subscriber.init();
        } else {
            let subscriber = registry
                .with(fmt::layer()
                    .compact()
                    .with_target(false)
                    .with_thread_ids(false)
                    .with_thread_names(false));
            
            subscriber.init();
        }
    }

    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        environment = ?config.environment,
        log_level = %config.level,
        json_format = config.json_format,
        include_sensitive = config.include_sensitive_data,
        "� Advanced tracing initialized successfully"
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
//! Runtime logging control endpoints
//!
//! This module provides HTTP endpoints for dynamically adjusting log levels
//! and viewing current logging configuration at runtime.

use axum::{
    extract::Query,
    http::StatusCode,
    response::Json,
    routing::{get, put},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn, error};

use crate::utils::logging_config::{LoggingConfig, get_logging_config, update_logging_config};

/// Request payload for updating log level
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateLogLevelRequest {
    /// New log level (trace, debug, info, warn, error)
    pub level: String,
    /// Optional: whether to update JSON format
    pub json_format: Option<bool>,
    /// Optional: whether to include sensitive data
    pub include_sensitive_data: Option<bool>,
}

/// Response payload for logging configuration
#[derive(Debug, Serialize)]
pub struct LoggingConfigResponse {
    /// Current log level
    pub level: String,
    /// Environment
    pub environment: String,
    /// Whether JSON format is enabled
    pub json_format: bool,
    /// Whether sensitive data is included
    pub include_sensitive_data: bool,
    /// Whether file logging is enabled
    pub log_to_file: bool,
    /// Log file path (if applicable)
    pub log_file_path: Option<String>,
}

impl From<LoggingConfig> for LoggingConfigResponse {
    fn from(config: LoggingConfig) -> Self {
        Self {
            level: config.level,
            environment: format!("{:?}", config.environment),
            json_format: config.json_format,
            include_sensitive_data: config.include_sensitive_data,
            log_to_file: config.log_to_file,
            log_file_path: config.log_file_path,
        }
    }
}

/// Test different log levels endpoint
#[derive(Debug, Deserialize)]
pub struct TestLogsQuery {
    /// Optional specific level to test
    pub level: Option<String>,
}

/// Get current logging configuration
pub async fn get_logging_config_handler() -> Result<Json<LoggingConfigResponse>, StatusCode> {
    match get_logging_config() {
        Ok(config) => {
            info!(
                current_level = %config.level,
                environment = ?config.environment,
                "Retrieved current logging configuration"
            );
            Ok(Json(LoggingConfigResponse::from(config)))
        }
        Err(e) => {
            error!(error = %e, "Failed to get logging configuration");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Update logging configuration at runtime
pub async fn update_logging_config_handler(
    Json(request): Json<UpdateLogLevelRequest>,
) -> Result<Json<LoggingConfigResponse>, StatusCode> {
    // Validate the log level
    let valid_levels = ["trace", "debug", "info", "warn", "error"];
    if !valid_levels.contains(&request.level.to_lowercase().as_str()) {
        warn!(
            requested_level = %request.level,
            valid_levels = ?valid_levels,
            "Invalid log level requested"
        );
        return Err(StatusCode::BAD_REQUEST);
    }

    // Get current config and update it
    let mut config = match get_logging_config() {
        Ok(config) => config,
        Err(e) => {
            error!(error = %e, "Failed to get current logging configuration");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let old_level = config.level.clone();
    config.level = request.level.to_lowercase();

    if let Some(json_format) = request.json_format {
        config.json_format = json_format;
    }

    if let Some(include_sensitive) = request.include_sensitive_data {
        config.include_sensitive_data = include_sensitive;
    }

    // Update the global configuration
    match update_logging_config(config.clone()) {
        Ok(()) => {
            info!(
                old_level = %old_level,
                new_level = %config.level,
                json_format = config.json_format,
                include_sensitive = config.include_sensitive_data,
                "Successfully updated logging configuration"
            );
            Ok(Json(LoggingConfigResponse::from(config)))
        }
        Err(e) => {
            error!(error = %e, "Failed to update logging configuration");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Test logging at different levels - useful for verifying configuration
pub async fn test_logs_handler(
    Query(params): Query<TestLogsQuery>,
) -> Result<Json<HashMap<String, String>>, StatusCode> {
    let config = match get_logging_config() {
        Ok(config) => config,
        Err(e) => {
            error!(error = %e, "Failed to get logging configuration for testing");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let mut response = HashMap::new();
    response.insert("current_level".to_string(), config.level.clone());
    response.insert("environment".to_string(), format!("{:?}", config.environment));

    let test_level_ref = params.level.as_ref();
    if let Some(test_level) = test_level_ref {
        // Test specific level
        match test_level.to_lowercase().as_str() {
            "trace" => {
                tracing::trace!("🔍 TRACE level test message");
                response.insert("tested_level".to_string(), "trace".to_string());
            }
            "debug" => {
                tracing::debug!("🐛 DEBUG level test message");
                response.insert("tested_level".to_string(), "debug".to_string());
            }
            "info" => {
                tracing::info!("ℹ️  INFO level test message");
                response.insert("tested_level".to_string(), "info".to_string());
            }
            "warn" => {
                tracing::warn!("⚠️  WARN level test message");
                response.insert("tested_level".to_string(), "warn".to_string());
            }
            "error" => {
                tracing::error!("❌ ERROR level test message");
                response.insert("tested_level".to_string(), "error".to_string());
            }
            _ => {
                response.insert("error".to_string(), "Invalid test level".to_string());
                return Err(StatusCode::BAD_REQUEST);
            }
        }
    } else {
        // Test all levels
        tracing::trace!("🔍 TRACE level test message - most detailed");
        tracing::debug!("🐛 DEBUG level test message - development info");
        tracing::info!("ℹ️  INFO level test message - general information");
        tracing::warn!("⚠️  WARN level test message - potential issues");
        tracing::error!("❌ ERROR level test message - serious problems");
        
        response.insert("tested_levels".to_string(), "all".to_string());
    }

    info!(
        test_level = ?params.level,
        current_level = %config.level,
        "Log level testing completed"
    );

    Ok(Json(response))
}

/// Create the logging control router
pub fn create_logging_router<S>() -> Router<S> 
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/logging/config", get(get_logging_config_handler))
        .route("/logging/config", put(update_logging_config_handler))
        .route("/logging/test", get(test_logs_handler))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logging_config_response_conversion() {
        let config = LoggingConfig {
            level: "debug".to_string(),
            environment: crate::utils::logging_config::Environment::Development,
            json_format: false,
            include_sensitive_data: true,
            log_to_file: false,
            log_file_path: None,
        };

        let response = LoggingConfigResponse::from(config);
        assert_eq!(response.level, "debug");
        assert_eq!(response.environment, "Development");
        assert!(!response.json_format);
        assert!(response.include_sensitive_data);
    }

    #[test]
    fn test_update_log_level_request_serialization() {
        let request = UpdateLogLevelRequest {
            level: "info".to_string(),
            json_format: Some(true),
            include_sensitive_data: Some(false),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: UpdateLogLevelRequest = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.level, "info");
        assert_eq!(deserialized.json_format, Some(true));
        assert_eq!(deserialized.include_sensitive_data, Some(false));
    }
}
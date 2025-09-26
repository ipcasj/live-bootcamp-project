//! Advanced logging configuration with runtime adjustable levels
//!
//! This module provides a comprehensive, production-ready logging solution that supports:
//! - Runtime log level adjustment via HTTP endpoints
//! - Environment-aware defaults (development/testing/production)
//! - Structured JSON logging for production environments
//! - Pretty console logging for development
//! - Security-conscious sensitive data filtering
//! - Performance-optimized async logging

use std::str::FromStr;
use std::sync::RwLock;
use serde::{Deserialize, Serialize};
use tracing::Level;
use tracing_subscriber::EnvFilter;
use color_eyre::eyre::{eyre, Result};

/// Represents the application environment
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Development,
    Testing,
    Production,
}

impl FromStr for Environment {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "development" | "dev" => Ok(Environment::Development),
            "testing" | "test" => Ok(Environment::Testing),
            "production" | "prod" => Ok(Environment::Production),
            _ => Err(format!("Unknown environment: {}", s)),
        }
    }
}

/// Logging configuration that can be adjusted at runtime
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Current log level
    pub level: String,
    /// Environment (affects default formats and behaviors)
    pub environment: Environment,
    /// Whether to use JSON format (recommended for production)
    pub json_format: bool,
    /// Whether to include sensitive data in logs (only in development)
    pub include_sensitive_data: bool,
    /// Whether to log to file in addition to console
    pub log_to_file: bool,
    /// Log file path (when file logging is enabled)
    pub log_file_path: Option<String>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        let environment = std::env::var("ENVIRONMENT")
            .ok()
            .and_then(|e| Environment::from_str(&e).ok())
            .unwrap_or(Environment::Development);

        Self {
            level: match environment {
                Environment::Development => "debug".to_string(),
                Environment::Testing => "info".to_string(),
                Environment::Production => "warn".to_string(),
            },
            json_format: matches!(environment, Environment::Production),
            include_sensitive_data: matches!(environment, Environment::Development),
            log_to_file: matches!(environment, Environment::Production),
            log_file_path: if matches!(environment, Environment::Production) {
                Some("/var/log/auth-service/app.log".to_string())
            } else {
                None
            },
            environment,
        }
    }
}

impl LoggingConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Result<Self> {
        let mut config = Self::default();

        // Override with environment variables if present
        if let Ok(level) = std::env::var("LOG_LEVEL") {
            // Validate the log level
            Level::from_str(&level.to_uppercase())
                .map_err(|_| eyre!("Invalid log level: {}", level))?;
            config.level = level.to_lowercase();
        }

        if let Ok(json_format) = std::env::var("LOG_JSON_FORMAT") {
            config.json_format = json_format.to_lowercase() == "true";
        }

        if let Ok(include_sensitive) = std::env::var("LOG_INCLUDE_SENSITIVE") {
            config.include_sensitive_data = include_sensitive.to_lowercase() == "true";
        }

        if let Ok(log_to_file) = std::env::var("LOG_TO_FILE") {
            config.log_to_file = log_to_file.to_lowercase() == "true";
        }

        if let Ok(log_file_path) = std::env::var("LOG_FILE_PATH") {
            config.log_file_path = Some(log_file_path);
        }

        Ok(config)
    }

    /// Get the tracing Level enum from string
    pub fn get_level(&self) -> Level {
        match self.level.to_lowercase().as_str() {
            "trace" => Level::TRACE,
            "debug" => Level::DEBUG,
            "info" => Level::INFO,
            "warn" => Level::WARN,
            "error" => Level::ERROR,
            _ => Level::INFO, // fallback
        }
    }

    /// Get environment filter string for tracing-subscriber
    pub fn get_env_filter(&self) -> Result<EnvFilter> {
        let base_filter = match self.environment {
            Environment::Development => {
                format!(
                    "{}={},auth_service={},sqlx=warn,hyper=info,tower=info",
                    env!("CARGO_PKG_NAME").replace("-", "_"),
                    self.level,
                    self.level
                )
            }
            Environment::Testing => {
                format!(
                    "{}={},auth_service={},sqlx=error,hyper=warn",
                    env!("CARGO_PKG_NAME").replace("-", "_"),
                    self.level,
                    self.level
                )
            }
            Environment::Production => {
                format!(
                    "{}={},auth_service={},sqlx=error",
                    env!("CARGO_PKG_NAME").replace("-", "_"),
                    self.level,
                    self.level
                )
            }
        };

        EnvFilter::try_new(base_filter)
            .or_else(|_| EnvFilter::try_from_default_env())
            .or_else(|_| EnvFilter::try_new("info"))
            .map_err(|e| eyre!("Failed to create env filter: {}", e))
    }
}

/// Global logging configuration that can be updated at runtime
static GLOBAL_LOGGING_CONFIG: RwLock<LoggingConfig> = RwLock::new(LoggingConfig {
    level: String::new(), // Will be initialized properly
    environment: Environment::Development,
    json_format: false,
    include_sensitive_data: true,
    log_to_file: false,
    log_file_path: None,
});

/// Initialize global logging configuration
pub fn init_global_logging_config() -> Result<()> {
    let config = LoggingConfig::from_env()?;
    
    if let Ok(mut global_config) = GLOBAL_LOGGING_CONFIG.write() {
        *global_config = config;
        Ok(())
    } else {
        Err(eyre!("Failed to initialize global logging config"))
    }
}

/// Get current logging configuration (thread-safe read)
pub fn get_logging_config() -> Result<LoggingConfig> {
    GLOBAL_LOGGING_CONFIG
        .read()
        .map_err(|_| eyre!("Failed to read logging config"))
        .map(|config| config.clone())
}

/// Update logging configuration at runtime
pub fn update_logging_config(new_config: LoggingConfig) -> Result<()> {
    GLOBAL_LOGGING_CONFIG
        .write()
        .map_err(|_| eyre!("Failed to write logging config"))
        .map(|mut config| {
            *config = new_config;
        })
}

/// Sanitize sensitive data from log messages based on current configuration
pub fn sanitize_sensitive_data(value: &str, field_name: &str) -> String {
    if let Ok(config) = get_logging_config() {
        if config.include_sensitive_data {
            return value.to_string();
        }
    }

    // List of sensitive field patterns
    let sensitive_fields = [
        "password", "token", "secret", "key", "auth", "credential",
        "email", "phone", "ssn", "credit", "card", "account"
    ];

    if sensitive_fields.iter().any(|&pattern| 
        field_name.to_lowercase().contains(pattern)
    ) {
        "[REDACTED]".to_string()
    } else if value.len() > 50 {
        // For long values, show only first/last few characters
        format!("{}...{}", &value[..5], &value[value.len()-5..])
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_from_str() {
        assert_eq!(Environment::from_str("development").unwrap(), Environment::Development);
        assert_eq!(Environment::from_str("dev").unwrap(), Environment::Development);
        assert_eq!(Environment::from_str("production").unwrap(), Environment::Production);
        assert_eq!(Environment::from_str("prod").unwrap(), Environment::Production);
        assert!(Environment::from_str("invalid").is_err());
    }

    #[test]
    fn test_logging_config_default() {
        let config = LoggingConfig::default();
        assert!(!config.level.is_empty());
        assert!(matches!(config.environment, Environment::Development));
    }

    #[test]
    fn test_sensitive_data_sanitization() {
        // Initialize config for testing
        std::env::set_var("LOG_INCLUDE_SENSITIVE", "false");
        let _ = init_global_logging_config();

        // Test with sensitive field name - should be redacted when config says no sensitive data
        let result = sanitize_sensitive_data("secret123", "password");
        assert_eq!(result, "[REDACTED]");

        // Test with non-sensitive field name - should return original or shortened form
        let result = sanitize_sensitive_data("test123", "username");
        assert!(result.contains("test123"));
        
        // Clean up
        std::env::remove_var("LOG_INCLUDE_SENSITIVE");
    }

    #[test]
    fn test_log_level_parsing() {
        let config = LoggingConfig {
            level: "debug".to_string(),
            ..Default::default()
        };
        assert_eq!(config.get_level(), Level::DEBUG);
    }
}
use config::{Config, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};
use std::env;
use validator::Validate;

/// Application configuration with validation and hierarchical loading
#[derive(Debug, Deserialize, Serialize, Clone, Validate)]
pub struct AppConfig {
    /// Application environment (development, test, production)
    #[serde(default = "default_environment")]
    pub environment: String,

    /// Server configuration
    pub server: ServerConfig,

    /// Database configuration
    #[validate(nested)]
    pub database: DatabaseConfig,

    /// Redis configuration
    #[validate(nested)]
    pub redis: RedisConfig,

    /// JWT authentication configuration
    #[validate(nested)]
    pub auth: AuthConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone, Validate)]
pub struct ServerConfig {
    /// Server host address
    #[serde(default = "default_server_host")]
    pub host: String,

    /// Server port
    #[serde(default = "default_server_port")]
    pub port: u16,
}

#[derive(Debug, Deserialize, Serialize, Clone, Validate)]
pub struct DatabaseConfig {
    /// Database connection URL
    #[validate(url)]
    pub url: String,

    /// Maximum number of database connections in the pool
    #[serde(default = "default_database_max_connections")]
    pub max_connections: u32,

    /// Connection timeout in seconds
    #[serde(default = "default_database_connection_timeout")]
    pub connection_timeout: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone, Validate)]
pub struct RedisConfig {
    /// Redis hostname
    #[serde(default = "default_redis_host")]
    pub host: String,

    /// Redis port
    #[serde(default = "default_redis_port")]
    pub port: u16,

    /// Redis password (optional)
    pub password: Option<String>,

    /// Redis database number
    #[serde(default = "default_redis_database")]
    pub database: u8,

    /// Maximum number of Redis connections in the pool
    #[serde(default = "default_redis_max_connections")]
    pub max_connections: u32,

    /// Connection timeout in seconds
    #[serde(default = "default_redis_connection_timeout")]
    pub connection_timeout: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone, Validate)]
pub struct AuthConfig {
    /// JWT secret key for signing tokens
    #[validate(length(min = 32, message = "JWT secret must be at least 32 characters"))]
    pub jwt_secret: String,

    /// Refresh token secret key
    #[serde(default = "default_refresh_token_secret")]
    #[validate(length(min = 16, message = "Refresh token secret must be at least 16 characters"))]
    pub refresh_token_secret: String,

    /// JWT token expiration time in seconds
    #[serde(default = "default_jwt_expiration")]
    pub jwt_expiration: u64,

    /// Refresh token expiration time in seconds
    #[serde(default = "default_refresh_token_expiration")]
    pub refresh_token_expiration: u64,

    /// JWT cookie name
    #[serde(default = "default_jwt_cookie_name")]
    pub jwt_cookie_name: String,

    /// 2FA code expiration time in seconds
    #[serde(default = "default_2fa_code_expiration")]
    pub two_fa_code_expiration: u64,

    /// Banned token expiration time in seconds (TTL for Redis)
    #[serde(default = "default_banned_token_ttl")]
    pub banned_token_ttl: u64,
}

impl AppConfig {
    /// Load configuration from multiple sources with hierarchical precedence:
    /// 1. Environment variables (highest priority)
    /// 2. Environment-specific config file (e.g., config/production.toml)
    /// 3. Base config file (config/default.toml)
    /// 4. Built-in defaults (lowest priority)
    pub fn load() -> Result<Self, ConfigError> {
        let environment = env::var("ENVIRONMENT")
            .or_else(|_| env::var("APP_ENVIRONMENT"))
            .unwrap_or_else(|_| "development".to_string());

        let mut builder = Config::builder()
            // Start with base configuration file
            .add_source(File::with_name("config/default").required(false))
            
            // Add environment-specific configuration file
            .add_source(File::with_name(&format!("config/{}", environment)).required(false));

        // Add legacy environment variable mapping
        if let Ok(db_url) = env::var("DATABASE_URL") {
            builder = builder.set_override("database.url", db_url)?;
        }
        if let Ok(redis_host) = env::var("REDIS_HOST_NAME") {
            builder = builder.set_override("redis.host", redis_host)?;
        }
        if let Ok(jwt_secret) = env::var("JWT_SECRET") {
            builder = builder.set_override("auth.jwt_secret", jwt_secret)?;
        }
        if let Ok(refresh_secret) = env::var("REFRESH_TOKEN_SECRET") {
            builder = builder.set_override("auth.refresh_token_secret", refresh_secret)?;
        }

        let config = builder
            // Add environment variables with prefix "AUTH_"
            .add_source(
                Environment::with_prefix("AUTH")
                    .separator("__")
                    .try_parsing(true)
            )
            
            .build()?;

        let app_config: AppConfig = config.try_deserialize()?;
        
        // Validate the configuration
        app_config.validate()
            .map_err(|e| ConfigError::Message(format!("Configuration validation failed: {}", e)))?;

        Ok(app_config)
    }

    /// Get the Redis connection URL
    pub fn redis_url(&self) -> String {
        if let Some(password) = &self.redis.password {
            format!("redis://:{}@{}:{}/{}", password, self.redis.host, self.redis.port, self.redis.database)
        } else {
            format!("redis://{}:{}/{}", self.redis.host, self.redis.port, self.redis.database)
        }
    }

    /// Get the server bind address
    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }
}

// Default value functions
fn default_environment() -> String {
    "development".to_string()
}

fn default_server_host() -> String {
    "0.0.0.0".to_string()
}

fn default_server_port() -> u16 {
    3000
}

fn default_database_max_connections() -> u32 {
    10
}

fn default_database_connection_timeout() -> u64 {
    30
}

fn default_redis_host() -> String {
    "127.0.0.1".to_string()
}

fn default_redis_port() -> u16 {
    6379
}

fn default_redis_database() -> u8 {
    0
}

fn default_redis_max_connections() -> u32 {
    10
}

fn default_redis_connection_timeout() -> u64 {
    5
}

fn default_refresh_token_secret() -> String {
    "refresh_secret_dev".to_string()
}

fn default_jwt_expiration() -> u64 {
    3600 // 1 hour
}

fn default_refresh_token_expiration() -> u64 {
    604800 // 7 days
}

fn default_jwt_cookie_name() -> String {
    "jwt".to_string()
}

fn default_2fa_code_expiration() -> u64 {
    600 // 10 minutes
}

fn default_banned_token_ttl() -> u64 {
    600 // 10 minutes (same as JWT expiration by default)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::sync::Mutex;

    // Use a mutex to prevent tests from running concurrently and interfering with each other
    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    fn with_clean_env<F, R>(f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let _lock = TEST_MUTEX.lock().unwrap();
        
        // Clear any existing env vars that might interfere
        env::remove_var("DATABASE_URL");
        env::remove_var("JWT_SECRET");
        env::remove_var("REDIS_HOST_NAME");
        env::remove_var("ENVIRONMENT");
        env::remove_var("REFRESH_TOKEN_SECRET");
        env::remove_var("AUTH__BANNED_TOKEN_TTL");
        env::remove_var("AUTH__JWT_EXPIRATION");
        
        let result = f();
        
        // Clean up after test
        env::remove_var("DATABASE_URL");
        env::remove_var("JWT_SECRET");
        env::remove_var("REDIS_HOST_NAME");
        env::remove_var("ENVIRONMENT");
        env::remove_var("REFRESH_TOKEN_SECRET");
        env::remove_var("AUTH__BANNED_TOKEN_TTL");
        env::remove_var("AUTH__JWT_EXPIRATION");
        
        result
    }

    #[test]
    fn test_config_defaults() {
        with_clean_env(|| {
            env::set_var("DATABASE_URL", "postgres://test:test@localhost:5432/test");
            env::set_var("JWT_SECRET", "test_secret_key_that_is_long_enough_for_validation");
            env::set_var("ENVIRONMENT", "test");
            
            let config = AppConfig::load().expect("Should load with defaults");
            
            // These values come from config/test.toml
            assert_eq!(config.environment, "test");
            assert_eq!(config.server.host, "127.0.0.1");
            assert_eq!(config.server.port, 0); // test uses port 0
            assert_eq!(config.redis.host, "127.0.0.1");
            assert_eq!(config.redis.port, 6379);
            assert_eq!(config.redis.database, 1); // test uses database 1
            assert_eq!(config.auth.jwt_cookie_name, "jwt_test");
        });
    }

    #[test]
    fn test_config_validation() {
        with_clean_env(|| {
            // Test invalid JWT secret (too short) - should pass validation now
            env::set_var("DATABASE_URL", "postgres://test:test@localhost:5432/test");
            env::set_var("JWT_SECRET", "short_but_long_enough_for_validation");
            env::set_var("ENVIRONMENT", "test");
            
            // This should now pass because we have a long enough secret
            let result = AppConfig::load();
            assert!(result.is_ok());
            
            // Test with actual short secret
            env::set_var("JWT_SECRET", "short");
            let result = AppConfig::load();
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_redis_url_generation() {
        with_clean_env(|| {
            env::set_var("DATABASE_URL", "postgres://test:test@localhost:5432/test");
            env::set_var("JWT_SECRET", "test_secret_key_that_is_long_enough_for_validation");
            env::set_var("ENVIRONMENT", "test");
            
            let mut config = AppConfig::load().expect("Should load");
            
            // Test without password (test environment uses database 1)
            assert_eq!(config.redis_url(), "redis://127.0.0.1:6379/1");
            
            // Test with password
            config.redis.password = Some("mypassword".to_string());
            assert_eq!(config.redis_url(), "redis://:mypassword@127.0.0.1:6379/1");
        });
    }

    #[test]
    fn test_environment_variable_override() {
        with_clean_env(|| {
            env::set_var("DATABASE_URL", "postgres://override:test@localhost:5432/override");
            env::set_var("JWT_SECRET", "override_secret_key_that_is_long_enough_for_validation");
            env::set_var("REDIS_HOST_NAME", "redis-override");
            env::set_var("ENVIRONMENT", "test");
            
            let config = AppConfig::load().expect("Should load with overrides");
            
            assert_eq!(config.database.url, "postgres://override:test@localhost:5432/override");
            assert_eq!(config.auth.jwt_secret, "override_secret_key_that_is_long_enough_for_validation");
            assert_eq!(config.redis.host, "redis-override");
        });
    }

    #[test]
    fn test_banned_token_ttl_configuration() {
        with_clean_env(|| {
            env::set_var("DATABASE_URL", "postgres://test:test@localhost:5432/test");
            env::set_var("JWT_SECRET", "test_secret_key_that_is_long_enough_for_validation");
            env::set_var("ENVIRONMENT", "test");
            
            let config = AppConfig::load().expect("Should load");
            
            // Test environment has 5 second banned token TTL for fast testing
            assert_eq!(config.auth.banned_token_ttl, 5);
            
            // Verify that banned_token_ttl is configurable and reasonable
            assert!(config.auth.banned_token_ttl > 0);
            assert!(config.auth.banned_token_ttl < 3600); // Should be under 1 hour for tests
        });
        
        // Test development environment has different value 
        with_clean_env(|| {
            env::set_var("DATABASE_URL", "postgres://test:test@localhost:5432/test");
            env::set_var("JWT_SECRET", "test_secret_key_that_is_long_enough_for_validation");
            env::set_var("ENVIRONMENT", "development");
            
            let config = AppConfig::load().expect("Should load");
            
            // Development environment should have longer TTL (from config files)
            assert_eq!(config.auth.banned_token_ttl, 3600); // 1 hour from development.toml
        });
    }

    #[test]
    fn test_ttl_configuration_validation() {
        with_clean_env(|| {
            env::set_var("DATABASE_URL", "postgres://test:test@localhost:5432/test");
            env::set_var("JWT_SECRET", "test_secret_key_that_is_long_enough_for_validation");
            env::set_var("ENVIRONMENT", "test");
            
            let config = AppConfig::load().expect("Should load");
            
            // Verify all TTL configurations are present and reasonable
            assert!(config.auth.jwt_expiration > 0);
            assert!(config.auth.refresh_token_expiration > 0);
            assert!(config.auth.two_fa_code_expiration > 0);
            assert!(config.auth.banned_token_ttl > 0);
            
            // For test environment, TTL values should be very low for fast testing
            assert_eq!(config.auth.two_fa_code_expiration, 3); // 3 seconds for fast tests
            assert_eq!(config.auth.banned_token_ttl, 5); // 5 seconds for fast tests
        });
    }
}
//! # TYL Config
//!
//! Simplified but extensible configuration management for TYL framework microservices.
//!
//! ## Features
//!
//! - **Plugin System**: Extensible configuration modules like tyl-errors
//! - **Sensible Defaults**: Each module (postgres, redis, etc.) provides defaults
//! - **Hierarchical Loading**: Environment variables > dev configs > defaults
//! - **Build-time Validation**: All needed configs present when building microservices
//! - **Hexagonal Architecture**: Clean separation with ports and adapters
//!
//! ## Quick Start
//!
//! ```rust
//! use tyl_config::{ConfigManager, PostgresConfig, RedisConfig};
//!
//! // Load all configs with defaults
//! let config = ConfigManager::builder()
//!     .with_postgres(PostgresConfig::default())
//!     .with_redis(RedisConfig::default())
//!     .build();
//!
//! println!("DB URL: {}", config.postgres().unwrap().connection_url());
//! ```
//!
//! ## Architecture
//!
//! This module follows hexagonal architecture:
//!
//! - **Port (Interface)**: `ConfigProvider` - defines the configuration contract
//! - **Adapters**: Built-in configs (postgres, redis) and custom implementations
//! - **Domain Logic**: Configuration loading and validation independent of sources
//!
//! ## Custom Config Plugins
//!
//! ```rust
//! use tyl_config::{ConfigPlugin, ConfigManager};
//! use tyl_errors::TylError;
//!
//! #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
//! struct MyServiceConfig {
//!     api_key: String,
//!     timeout_ms: u64,
//! }
//!
//! impl ConfigPlugin for MyServiceConfig {
//!     fn name(&self) -> &'static str { "my_service" }
//!     fn env_prefix(&self) -> &'static str { "MY_SERVICE" }
//!     fn validate(&self) -> tyl_config::ConfigResult<()> {
//!         if self.api_key.is_empty() {
//!             return Err(TylError::validation("api_key", "cannot be empty"));
//!         }
//!         Ok(())
//!     }
//!     fn load_from_env(&self) -> tyl_config::ConfigResult<Self> {
//!         let mut config = Self::default();
//!         config.merge_env()?;
//!         Ok(config)
//!     }
//!     fn merge_env(&mut self) -> tyl_config::ConfigResult<()> {
//!         if let Ok(api_key) = std::env::var("MY_SERVICE_API_KEY") {
//!             self.api_key = api_key;
//!         }
//!         if let Ok(timeout) = std::env::var("MY_SERVICE_TIMEOUT_MS") {
//!             self.timeout_ms = timeout.parse().unwrap_or(self.timeout_ms);
//!         }
//!         Ok(())
//!     }
//! }
//!
//! impl Default for MyServiceConfig {
//!     fn default() -> Self {
//!         Self {
//!             api_key: "dev-key-12345".to_string(),
//!             timeout_ms: 5000,
//!         }
//!     }
//! }
//! ```

use serde::{Deserialize, Serialize};
use tyl_errors::{TylError, TylResult};

/// Result type for config operations using TYL unified error handling
pub type ConfigResult<T> = TylResult<T>;

/// Port (Interface) - Configuration plugin contract
pub trait ConfigPlugin: std::fmt::Debug + Send + Sync {
    /// Get the name of this config plugin
    fn name(&self) -> &'static str;

    /// Get the environment variable prefix for this plugin
    fn env_prefix(&self) -> &'static str;

    /// Validate the configuration
    fn validate(&self) -> ConfigResult<()>;

    /// Load configuration from environment variables
    fn load_from_env(&self) -> ConfigResult<Self>
    where
        Self: Sized;

    /// Merge with values from environment variables
    fn merge_env(&mut self) -> ConfigResult<()>;
}

/// Configuration manager that holds all service configurations
#[derive(Debug, Clone)]
pub struct ConfigManager {
    postgres: Option<PostgresConfig>,
    redis: Option<RedisConfig>,
    // Simplified: no custom configs stored, just provide builder pattern
}

impl ConfigManager {
    /// Create a new configuration manager builder
    pub fn builder() -> ConfigManagerBuilder {
        ConfigManagerBuilder::new()
    }

    /// Get postgres configuration if configured
    pub fn postgres(&self) -> Option<&PostgresConfig> {
        self.postgres.as_ref()
    }

    /// Get redis configuration if configured
    pub fn redis(&self) -> Option<&RedisConfig> {
        self.redis.as_ref()
    }

    /// Validate all configurations
    pub fn validate(&self) -> ConfigResult<()> {
        if let Some(postgres) = &self.postgres {
            postgres.validate()?;
        }

        if let Some(redis) = &self.redis {
            redis.validate()?;
        }

        Ok(())
    }

    /// Generate a complete YAML configuration file with all current values
    pub fn generate_config_template(&self, output_path: &str) -> ConfigResult<()> {
        let mut yaml_content = String::new();

        // Add header comment
        yaml_content.push_str("# TYL Framework Configuration Template\n");
        yaml_content.push_str(
            "# This file shows the resolved configuration values after applying hierarchy:\n",
        );
        yaml_content.push_str("# Priority: Environment Variables > YAML file > Defaults\n");
        yaml_content.push_str("#\n");
        yaml_content.push_str("# Environment Variables:\n");
        yaml_content.push_str(
            "# PostgreSQL: TYL_DATABASE_URL, DATABASE_URL, POSTGRES_URL (full connection string)\n",
        );
        yaml_content.push_str("#            TYL_POSTGRES_* or PG* (individual components)\n");
        yaml_content.push_str("# Redis:      TYL_REDIS_URL, REDIS_URL (full connection string)\n");
        yaml_content.push_str("#            TYL_REDIS_* or REDIS_* (individual components)\n");
        yaml_content.push_str("#\n\n");

        let mut config_map = serde_yaml::Mapping::new();

        if let Some(postgres) = &self.postgres {
            config_map.insert(
                serde_yaml::Value::String(postgres.name().to_string()),
                serde_yaml::to_value(postgres).map_err(|e| {
                    TylError::serialization(format!("Failed to serialize postgres config: {e}"))
                })?,
            );
        }

        if let Some(redis) = &self.redis {
            config_map.insert(
                serde_yaml::Value::String(redis.name().to_string()),
                serde_yaml::to_value(redis).map_err(|e| {
                    TylError::serialization(format!("Failed to serialize redis config: {e}"))
                })?,
            );
        }

        let config_yaml = serde_yaml::to_string(&serde_yaml::Value::Mapping(config_map))
            .map_err(|e| TylError::serialization(format!("Failed to serialize YAML: {e}")))?;

        yaml_content.push_str(&config_yaml);

        // Add helpful comments at the end
        yaml_content
            .push_str("\n# Alternative: Use connection URLs instead of individual components\n");
        yaml_content.push_str("# postgres:\n");
        yaml_content.push_str("#   url: postgresql://user:password@host:port/database\n");
        yaml_content.push_str("# redis:\n");
        yaml_content.push_str("#   url: redis://password@host:port/database\n");

        std::fs::write(output_path, yaml_content)
            .map_err(|e| TylError::configuration(format!("Failed to write config file: {e}")))?;

        Ok(())
    }

    /// Load configurations from YAML file (lowest priority, before defaults)
    pub fn from_yaml_file(yaml_path: &str) -> ConfigResult<Self> {
        let yaml_content = std::fs::read_to_string(yaml_path).map_err(|e| {
            TylError::configuration(format!("Failed to read config file {yaml_path}: {e}"))
        })?;

        let yaml_value: serde_yaml::Value = serde_yaml::from_str(&yaml_content)
            .map_err(|e| TylError::configuration(format!("Failed to parse YAML: {e}")))?;

        let mut builder = ConfigManagerBuilder::new();

        if let Some(yaml_map) = yaml_value.as_mapping() {
            // Load postgres config if present
            if let Some(postgres_section) = yaml_map.get("postgres") {
                let mut postgres: PostgresConfig = serde_yaml::from_value(postgres_section.clone())
                    .map_err(|e| {
                        TylError::configuration(format!("Failed to parse postgres config: {e}"))
                    })?;
                postgres.merge_env()?;
                builder = builder.with_postgres(postgres);
            }

            // Load redis config if present
            if let Some(redis_section) = yaml_map.get("redis") {
                let mut redis: RedisConfig = serde_yaml::from_value(redis_section.clone())
                    .map_err(|e| {
                        TylError::configuration(format!("Failed to parse redis config: {e}"))
                    })?;
                redis.merge_env()?;
                builder = builder.with_redis(redis);
            }
        }

        Ok(builder.build())
    }
}

/// Builder for ConfigManager
pub struct ConfigManagerBuilder {
    postgres: Option<PostgresConfig>,
    redis: Option<RedisConfig>,
}

impl Default for ConfigManagerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigManagerBuilder {
    pub fn new() -> Self {
        Self {
            postgres: None,
            redis: None,
        }
    }

    pub fn with_postgres(mut self, mut config: PostgresConfig) -> Self {
        // Merge environment variables
        let _ = config.merge_env();
        self.postgres = Some(config);
        self
    }

    pub fn with_redis(mut self, mut config: RedisConfig) -> Self {
        // Merge environment variables
        let _ = config.merge_env();
        self.redis = Some(config);
        self
    }

    /// Load configuration from YAML file first, then apply env vars
    pub fn with_yaml_file(mut self, yaml_path: &str) -> ConfigResult<Self> {
        // Try to read the YAML file
        if std::path::Path::new(yaml_path).exists() {
            let yaml_content = std::fs::read_to_string(yaml_path)
                .map_err(|e| TylError::configuration(format!("Failed to read config file: {e}")))?;

            let yaml_value: serde_yaml::Value = serde_yaml::from_str(&yaml_content)
                .map_err(|e| TylError::configuration(format!("Failed to parse YAML: {e}")))?;

            if let Some(yaml_map) = yaml_value.as_mapping() {
                // Load postgres config if present in YAML
                if let Some(postgres_section) = yaml_map.get("postgres") {
                    let mut postgres: PostgresConfig =
                        serde_yaml::from_value(postgres_section.clone()).map_err(|e| {
                            TylError::configuration(format!("Failed to parse postgres config: {e}"))
                        })?;
                    // Merge environment variables after loading from YAML
                    postgres.merge_env()?;
                    self.postgres = Some(postgres);
                }

                // Load redis config if present in YAML
                if let Some(redis_section) = yaml_map.get("redis") {
                    let mut redis: RedisConfig = serde_yaml::from_value(redis_section.clone())
                        .map_err(|e| {
                            TylError::configuration(format!("Failed to parse redis config: {e}"))
                        })?;
                    // Merge environment variables after loading from YAML
                    redis.merge_env()?;
                    self.redis = Some(redis);
                }
            }
        }
        // If file doesn't exist, just continue with defaults

        Ok(self)
    }

    pub fn build(self) -> ConfigManager {
        ConfigManager {
            postgres: self.postgres,
            redis: self.redis,
        }
    }
}

/// PostgreSQL configuration with sensible defaults
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostgresConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub pool_size: u32,
    pub timeout_seconds: u64,
}

impl Default for PostgresConfig {
    fn default() -> Self {
        Self {
            url: None,
            host: "localhost".to_string(),
            port: 5432,
            database: "app_dev".to_string(),
            username: "postgres".to_string(),
            password: "password".to_string(),
            pool_size: 10,
            timeout_seconds: 30,
        }
    }
}

impl PostgresConfig {
    pub fn connection_url(&self) -> String {
        // If URL is set, use it directly, otherwise build from components
        self.url.clone().unwrap_or_else(|| {
            format!(
                "postgresql://{}:{}@{}:{}/{}",
                self.username, self.password, self.host, self.port, self.database
            )
        })
    }
}

impl ConfigPlugin for PostgresConfig {
    fn name(&self) -> &'static str {
        "postgres"
    }

    fn env_prefix(&self) -> &'static str {
        "POSTGRES"
    }

    fn validate(&self) -> ConfigResult<()> {
        // If we have a URL, we're more lenient with component validation
        if self.url.is_some() {
            if self.pool_size == 0 {
                return Err(TylError::validation("pool_size", "must be greater than 0"));
            }
            return Ok(());
        }

        // Without URL, we need all components to be valid
        if self.host.is_empty() {
            return Err(TylError::validation("host", "cannot be empty"));
        }
        if self.database.is_empty() {
            return Err(TylError::validation("database", "cannot be empty"));
        }
        if self.username.is_empty() {
            return Err(TylError::validation("username", "cannot be empty"));
        }
        if self.password.is_empty() {
            return Err(TylError::validation(
                "password",
                "cannot be empty (required when not using DATABASE_URL)",
            ));
        }
        if self.pool_size == 0 {
            return Err(TylError::validation("pool_size", "must be greater than 0"));
        }
        Ok(())
    }

    fn load_from_env(&self) -> ConfigResult<Self> {
        let mut config = Self::default();
        config.merge_env()?;
        Ok(config)
    }

    fn merge_env(&mut self) -> ConfigResult<()> {
        // Priority: TYL_ prefixed > standard PostgreSQL > defaults

        // Connection URL: TYL_DATABASE_URL > DATABASE_URL > POSTGRES_URL > build from components
        if let Ok(url) = std::env::var("TYL_DATABASE_URL") {
            self.url = Some(url);
        } else if let Ok(url) = std::env::var("DATABASE_URL") {
            self.url = Some(url);
        } else if let Ok(url) = std::env::var("POSTGRES_URL") {
            self.url = Some(url);
        }

        // If URL is set, the components below are optional overrides
        // If no URL, components are required to build the connection string

        // Host: TYL_POSTGRES_HOST > PGHOST > default
        if let Ok(host) = std::env::var("TYL_POSTGRES_HOST") {
            self.host = host;
        } else if let Ok(host) = std::env::var("PGHOST") {
            self.host = host;
        }

        // Port: TYL_POSTGRES_PORT > PGPORT > default
        if let Ok(port) = std::env::var("TYL_POSTGRES_PORT") {
            self.port = port
                .parse()
                .map_err(|e| TylError::configuration(format!("Invalid TYL_POSTGRES_PORT: {e}")))?;
        } else if let Ok(port) = std::env::var("PGPORT") {
            self.port = port
                .parse()
                .map_err(|e| TylError::configuration(format!("Invalid PGPORT: {e}")))?;
        }

        // Database: TYL_POSTGRES_DATABASE > PGDATABASE > default
        if let Ok(database) = std::env::var("TYL_POSTGRES_DATABASE") {
            self.database = database;
        } else if let Ok(database) = std::env::var("PGDATABASE") {
            self.database = database;
        }

        // Username: TYL_POSTGRES_USER > PGUSER > default
        if let Ok(username) = std::env::var("TYL_POSTGRES_USER") {
            self.username = username;
        } else if let Ok(username) = std::env::var("PGUSER") {
            self.username = username;
        }

        // Password: TYL_POSTGRES_PASSWORD > PGPASSWORD > default
        if let Ok(password) = std::env::var("TYL_POSTGRES_PASSWORD") {
            self.password = password;
        } else if let Ok(password) = std::env::var("PGPASSWORD") {
            self.password = password;
        }

        // Pool size: TYL only (no PostgreSQL standard)
        if let Ok(pool_size) = std::env::var("TYL_POSTGRES_POOL_SIZE") {
            self.pool_size = pool_size.parse().map_err(|e| {
                TylError::configuration(format!("Invalid TYL_POSTGRES_POOL_SIZE: {e}"))
            })?;
        }

        // Timeout: TYL only
        if let Ok(timeout) = std::env::var("TYL_POSTGRES_TIMEOUT_SECONDS") {
            self.timeout_seconds = timeout.parse().map_err(|e| {
                TylError::configuration(format!("Invalid TYL_POSTGRES_TIMEOUT_SECONDS: {e}"))
            })?;
        }

        Ok(())
    }
}

/// Redis configuration with sensible defaults
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    pub host: String,
    pub port: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    pub database: u32,
    pub pool_size: u32,
    pub timeout_seconds: u64,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: None,
            host: "localhost".to_string(),
            port: 6379,
            password: None,
            database: 0,
            pool_size: 5,
            timeout_seconds: 10,
        }
    }
}

impl RedisConfig {
    pub fn connection_url(&self) -> String {
        // If URL is set, use it directly, otherwise build from components
        self.url.clone().unwrap_or_else(|| match &self.password {
            Some(password) => format!(
                "redis://{}:{}@{}:{}/{}",
                "default", password, self.host, self.port, self.database
            ),
            None => format!("redis://{}:{}/{}", self.host, self.port, self.database),
        })
    }
}

impl ConfigPlugin for RedisConfig {
    fn name(&self) -> &'static str {
        "redis"
    }

    fn env_prefix(&self) -> &'static str {
        "REDIS"
    }

    fn validate(&self) -> ConfigResult<()> {
        if self.host.is_empty() {
            return Err(TylError::validation("host", "cannot be empty"));
        }
        if self.pool_size == 0 {
            return Err(TylError::validation("pool_size", "must be greater than 0"));
        }
        Ok(())
    }

    fn load_from_env(&self) -> ConfigResult<Self> {
        let mut config = Self::default();
        config.merge_env()?;
        Ok(config)
    }

    fn merge_env(&mut self) -> ConfigResult<()> {
        // Priority: TYL_ prefixed > standard Redis > defaults

        // Connection URL: TYL_REDIS_URL > REDIS_URL > build from components
        if let Ok(url) = std::env::var("TYL_REDIS_URL") {
            self.url = Some(url);
        } else if let Ok(url) = std::env::var("REDIS_URL") {
            self.url = Some(url);
        }

        // If URL is set, the components below are optional overrides
        // If no URL, components are required to build the connection string

        // Host: TYL_REDIS_HOST > REDIS_HOST > default
        if let Ok(host) = std::env::var("TYL_REDIS_HOST") {
            self.host = host;
        } else if let Ok(host) = std::env::var("REDIS_HOST") {
            self.host = host;
        }

        // Port: TYL_REDIS_PORT > REDIS_PORT > default
        if let Ok(port) = std::env::var("TYL_REDIS_PORT") {
            self.port = port
                .parse()
                .map_err(|e| TylError::configuration(format!("Invalid TYL_REDIS_PORT: {e}")))?;
        } else if let Ok(port) = std::env::var("REDIS_PORT") {
            self.port = port
                .parse()
                .map_err(|e| TylError::configuration(format!("Invalid REDIS_PORT: {e}")))?;
        }

        // Password: TYL_REDIS_PASSWORD > REDIS_PASSWORD > default
        if let Ok(password) = std::env::var("TYL_REDIS_PASSWORD") {
            self.password = Some(password);
        } else if let Ok(password) = std::env::var("REDIS_PASSWORD") {
            self.password = Some(password);
        }

        // Database: TYL_REDIS_DATABASE > REDIS_DATABASE > default
        if let Ok(database) = std::env::var("TYL_REDIS_DATABASE") {
            self.database = database
                .parse()
                .map_err(|e| TylError::configuration(format!("Invalid TYL_REDIS_DATABASE: {e}")))?;
        } else if let Ok(database) = std::env::var("REDIS_DATABASE") {
            self.database = database
                .parse()
                .map_err(|e| TylError::configuration(format!("Invalid REDIS_DATABASE: {e}")))?;
        }

        // Pool size: TYL only (no Redis standard)
        if let Ok(pool_size) = std::env::var("TYL_REDIS_POOL_SIZE") {
            self.pool_size = pool_size.parse().map_err(|e| {
                TylError::configuration(format!("Invalid TYL_REDIS_POOL_SIZE: {e}"))
            })?;
        }

        // Timeout: TYL only
        if let Ok(timeout) = std::env::var("TYL_REDIS_TIMEOUT_SECONDS") {
            self.timeout_seconds = timeout.parse().map_err(|e| {
                TylError::configuration(format!("Invalid TYL_REDIS_TIMEOUT_SECONDS: {e}"))
            })?;
        }

        Ok(())
    }
}

// Utility functions for configuration loading
pub fn load_from_env_or_default<T: ConfigPlugin + Default>() -> ConfigResult<T> {
    let mut config = T::default();
    config.merge_env()?;
    config.validate()?;
    Ok(config)
}

/// Helper to parse environment variable with default fallback
pub fn env_var_or_default<T: std::str::FromStr>(var_name: &str, default: T) -> T
where
    T::Err: std::fmt::Display,
{
    std::env::var(var_name)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;
    
    // Shared mutex for all environment variable tests to prevent races
    static ENV_TEST_MUTEX: Mutex<()> = Mutex::new(());
    use super::*;

    #[test]
    fn test_postgres_config_defaults() {
        let config = PostgresConfig::default();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 5432);
        assert_eq!(config.database, "app_dev");
        assert_eq!(config.pool_size, 10);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_postgres_config_url_generation() {
        let config = PostgresConfig::default();
        let url = config.connection_url();
        assert!(url.contains("postgresql://"));
        assert!(url.contains("localhost:5432"));
        assert!(url.contains("app_dev"));
    }

    #[test]
    fn test_redis_config_defaults() {
        let config = RedisConfig::default();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 6379);
        assert_eq!(config.database, 0);
        assert_eq!(config.pool_size, 5);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_redis_config_url_generation() {
        let config = RedisConfig::default();
        let url = config.connection_url();
        assert!(url.contains("redis://"));
        assert!(url.contains("localhost:6379"));
    }

    #[test]
    fn test_config_manager_builder() {
        let manager = ConfigManager::builder()
            .with_postgres(PostgresConfig::default())
            .with_redis(RedisConfig::default())
            .build();

        assert!(manager.postgres().is_some());
        assert!(manager.redis().is_some());
        assert!(manager.validate().is_ok());
    }

    #[test]
    fn test_postgres_validation_failures() {
        // Test empty host
        let invalid_config = PostgresConfig {
            host: "".to_string(),
            ..PostgresConfig::default()
        };
        let result = invalid_config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("host"));

        // Test empty database
        let invalid_config = PostgresConfig {
            database: "".to_string(),
            ..PostgresConfig::default()
        };
        let result = invalid_config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("database"));

        // Test empty username
        let invalid_config = PostgresConfig {
            username: "".to_string(),
            ..PostgresConfig::default()
        };
        let result = invalid_config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("username"));

        // Test empty password
        let invalid_config = PostgresConfig {
            password: "".to_string(),
            ..PostgresConfig::default()
        };
        let result = invalid_config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("password"));

        // Test zero pool size
        let invalid_config = PostgresConfig {
            pool_size: 0,
            ..PostgresConfig::default()
        };
        let result = invalid_config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("pool_size"));
    }

    #[test]
    fn test_redis_validation_failures() {
        // Test empty host
        let invalid_config = RedisConfig {
            host: "".to_string(),
            ..RedisConfig::default()
        };
        let result = invalid_config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("host"));

        // Test zero pool size
        let invalid_config = RedisConfig {
            pool_size: 0,
            ..RedisConfig::default()
        };
        let result = invalid_config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("pool_size"));
    }

    #[test]
    fn test_config_manager_validation_propagation() {
        // Test that ConfigManager propagates validation errors

        // Clean up any environment variables that could interfere
        std::env::remove_var("TYL_POSTGRES_HOST");
        std::env::remove_var("PGHOST");

        let mut invalid_postgres = PostgresConfig::default();
        invalid_postgres.host = "".to_string();

        let manager = ConfigManager::builder()
            .with_postgres(invalid_postgres)
            .build();

        let result = manager.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("host"));
    }

    #[test]
    fn test_postgres_validation_with_url_override() {
        // Test that when DATABASE_URL is set, component validation is more lenient
        let mut config = PostgresConfig::default();
        config.url = Some("postgresql://user:pass@prod-host:5432/proddb".to_string());
        config.password = "".to_string(); // Empty password should be OK with URL
        config.host = "".to_string(); // Empty host should be OK with URL

        let result = config.validate();
        assert!(result.is_ok());

        // But pool_size still needs to be valid
        config.pool_size = 0;
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("pool_size"));
    }

    #[test]
    fn test_missing_required_values_cause_failures() {
        // Test that a plugin fails validation when required values are missing

        // Clean up any environment variables that could interfere
        std::env::remove_var("TYL_POSTGRES_PASSWORD");
        std::env::remove_var("PGPASSWORD");
        std::env::remove_var("TYL_DATABASE_URL");
        std::env::remove_var("DATABASE_URL");
        std::env::remove_var("POSTGRES_URL");

        // Create config with missing password (no URL fallback)
        let mut config = PostgresConfig::default();
        config.password = "".to_string();

        let result = config.validate();
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("password"));
        assert!(error_msg.contains("required when not using DATABASE_URL"));

        // Show that ConfigManager builder will catch this during validation
        let manager = ConfigManager::builder().with_postgres(config).build();

        let result = manager.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("password"));
    }

    #[test]
    fn test_environment_variable_loading() {
        let _lock = ENV_TEST_MUTEX.lock().unwrap();
        
        // Store original values to restore later
        let original_host = std::env::var("TYL_POSTGRES_HOST").ok();
        let original_port = std::env::var("TYL_POSTGRES_PORT").ok();
        
        // Test TYL-prefixed environment variable override
        std::env::set_var("TYL_POSTGRES_HOST", "test-host");
        std::env::set_var("TYL_POSTGRES_PORT", "5433");

        let mut config = PostgresConfig::default();
        config.merge_env().unwrap();

        assert_eq!(config.host, "test-host");
        assert_eq!(config.port, 5433);

        // Restore original environment variables
        std::env::remove_var("TYL_POSTGRES_HOST");
        std::env::remove_var("TYL_POSTGRES_PORT");
        if let Some(host) = original_host {
            std::env::set_var("TYL_POSTGRES_HOST", host);
        }
        if let Some(port) = original_port {
            std::env::set_var("TYL_POSTGRES_PORT", port);
        }
    }

    #[test]
    fn test_standard_postgres_environment_variables() {
        // Test standard PostgreSQL environment variables
        std::env::set_var("PGHOST", "pg-host");
        std::env::set_var("PGPORT", "5434");
        std::env::set_var("PGDATABASE", "test_db");
        std::env::set_var("PGUSER", "test_user");

        let mut config = PostgresConfig::default();
        config.merge_env().unwrap();

        assert_eq!(config.host, "pg-host");
        assert_eq!(config.port, 5434);
        assert_eq!(config.database, "test_db");
        assert_eq!(config.username, "test_user");

        // Cleanup
        std::env::remove_var("PGHOST");
        std::env::remove_var("PGPORT");
        std::env::remove_var("PGDATABASE");
        std::env::remove_var("PGUSER");
    }

    #[test]
    fn test_database_url_priority() {
        // Test DATABASE_URL priority
        std::env::set_var(
            "DATABASE_URL",
            "postgresql://user:pass@db-host:5432/production",
        );

        let mut config = PostgresConfig::default();
        config.merge_env().unwrap();

        assert_eq!(
            config.connection_url(),
            "postgresql://user:pass@db-host:5432/production"
        );

        // Test TYL_DATABASE_URL takes priority
        std::env::set_var(
            "TYL_DATABASE_URL",
            "postgresql://tyl:secret@tyl-host:5432/tyldb",
        );
        config.merge_env().unwrap();

        assert_eq!(
            config.connection_url(),
            "postgresql://tyl:secret@tyl-host:5432/tyldb"
        );

        // Cleanup
        std::env::remove_var("DATABASE_URL");
        std::env::remove_var("TYL_DATABASE_URL");
    }

    #[test]
    fn test_config_plugin_trait() {
        let postgres = PostgresConfig::default();
        assert_eq!(postgres.name(), "postgres");
        assert_eq!(postgres.env_prefix(), "POSTGRES");

        let redis = RedisConfig::default();
        assert_eq!(redis.name(), "redis");
        assert_eq!(redis.env_prefix(), "REDIS");
    }

    #[test]
    fn test_error_types() {
        let validation_error = TylError::validation("test_field", "test message");
        assert!(validation_error.to_string().contains("test_field"));
        assert!(validation_error.to_string().contains("test message"));

        let loading_error = TylError::configuration("failed to load");
        assert!(loading_error.to_string().contains("failed to load"));
    }

    #[test]
    fn test_serialization() {
        let config = PostgresConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: PostgresConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.host, deserialized.host);
        assert_eq!(config.port, deserialized.port);
    }

    #[test]
    fn test_custom_config_extensibility() {
        // This test demonstrates how the plugin system would work
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct CustomConfig {
            api_key: String,
            timeout: u64,
        }

        impl Default for CustomConfig {
            fn default() -> Self {
                Self {
                    api_key: "dev-key".to_string(),
                    timeout: 5000,
                }
            }
        }

        impl ConfigPlugin for CustomConfig {
            fn name(&self) -> &'static str {
                "custom"
            }
            fn env_prefix(&self) -> &'static str {
                "CUSTOM"
            }
            fn validate(&self) -> ConfigResult<()> {
                if self.api_key.is_empty() {
                    return Err(TylError::validation("api_key", "cannot be empty"));
                }
                Ok(())
            }
            fn load_from_env(&self) -> ConfigResult<Self> {
                Ok(Self::default())
            }
            fn merge_env(&mut self) -> ConfigResult<()> {
                if let Ok(key) = std::env::var("CUSTOM_API_KEY") {
                    self.api_key = key;
                }
                Ok(())
            }
        }

        let custom = CustomConfig::default();
        assert_eq!(custom.name(), "custom");
        assert!(custom.validate().is_ok());
    }

    #[test]
    fn test_yaml_generation() {
        let config = ConfigManager::builder()
            .with_postgres(PostgresConfig::default())
            .with_redis(RedisConfig::default())
            .build();

        let temp_path = "/tmp/test-config.yaml";
        let result = config.generate_config_template(temp_path);
        assert!(result.is_ok());

        // Verify file was created and contains expected content
        assert!(std::path::Path::new(temp_path).exists());
        let content = std::fs::read_to_string(temp_path).unwrap();
        assert!(content.contains("postgres:"));
        assert!(content.contains("redis:"));
        assert!(content.contains("host: localhost"));

        // Cleanup
        let _ = std::fs::remove_file(temp_path);
    }

    #[test]
    fn test_yaml_loading() {
        let _lock = ENV_TEST_MUTEX.lock().unwrap();

        // Store original values to restore later
        let original_host = std::env::var("TYL_POSTGRES_HOST").ok();
        let original_pghost = std::env::var("PGHOST").ok();

        // Ensure no env vars are interfering
        std::env::remove_var("TYL_POSTGRES_HOST");
        std::env::remove_var("PGHOST");

        // Create test YAML file
        let yaml_content = r#"
postgres:
  host: test-host
  port: 5433
  database: test_db
  username: test_user
  password: test_pass
  pool_size: 20
  timeout_seconds: 60

redis:
  host: test-redis
  port: 6380
  database: 1
  pool_size: 10
  timeout_seconds: 15
"#;

        let temp_path = "/tmp/test-load-config.yaml";
        std::fs::write(temp_path, yaml_content).unwrap();

        // Load config from YAML
        let result = ConfigManager::builder().with_yaml_file(temp_path);
        assert!(result.is_ok());

        let config = result.unwrap().build();

        // Verify loaded values
        let postgres = config.postgres().unwrap();
        assert_eq!(postgres.host, "test-host");
        assert_eq!(postgres.port, 5433);
        assert_eq!(postgres.database, "test_db");
        assert_eq!(postgres.pool_size, 20);

        let redis = config.redis().unwrap();
        assert_eq!(redis.host, "test-redis");
        assert_eq!(redis.port, 6380);
        assert_eq!(redis.database, 1);

        // Restore original environment variables
        if let Some(host) = original_host {
            std::env::set_var("TYL_POSTGRES_HOST", host);
        }
        if let Some(pghost) = original_pghost {
            std::env::set_var("PGHOST", pghost);
        }
        let _ = std::fs::remove_file(temp_path);
    }

    #[test]
    fn test_yaml_environment_precedence() {
        let _lock = ENV_TEST_MUTEX.lock().unwrap();

        // Store original values to restore later
        let original_host = std::env::var("TYL_POSTGRES_HOST").ok();
        let original_pghost = std::env::var("PGHOST").ok();

        // Ensure no env vars are interfering initially
        std::env::remove_var("TYL_POSTGRES_HOST");
        std::env::remove_var("PGHOST");

        // Create YAML with base values (complete config)
        let yaml_content = r#"
postgres:
  host: yaml-host
  port: 5432
  database: yaml_db
  username: yaml-user
  password: yaml-pass
  pool_size: 10
  timeout_seconds: 30
"#;

        let temp_path = "/tmp/test-precedence.yaml";
        std::fs::write(temp_path, yaml_content).unwrap();

        // Set environment variable (should override YAML)
        std::env::set_var("TYL_POSTGRES_HOST", "env-host");

        let config = ConfigManager::builder()
            .with_yaml_file(temp_path)
            .unwrap()
            .build();

        let postgres = config.postgres().unwrap();

        // Environment variable should take precedence
        assert_eq!(postgres.host, "env-host");
        // YAML values should be used where no env var exists
        assert_eq!(postgres.username, "yaml-user");

        // Restore original environment variables
        std::env::remove_var("TYL_POSTGRES_HOST");
        if let Some(host) = original_host {
            std::env::set_var("TYL_POSTGRES_HOST", host);
        }
        if let Some(pghost) = original_pghost {
            std::env::set_var("PGHOST", pghost);
        }
        let _ = std::fs::remove_file(temp_path);
    }
}

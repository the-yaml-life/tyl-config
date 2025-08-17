use tyl_config::{ConfigManager, ConfigPlugin, PostgresConfig, RedisConfig};
use tyl_errors::TylError;

#[test]
fn test_end_to_end_configuration() {
    let config = ConfigManager::builder()
        .with_postgres(PostgresConfig::default())
        .with_redis(RedisConfig::default())
        .build();

    assert!(config.postgres().is_some());
    assert!(config.redis().is_some());
    assert!(config.validate().is_ok());
}

#[test]
fn test_environment_variable_precedence() {
    // Set both TYL and standard variables
    std::env::set_var("TYL_POSTGRES_HOST", "tyl-host");
    std::env::set_var("PGHOST", "pg-host");
    std::env::set_var("TYL_REDIS_HOST", "tyl-redis");
    std::env::set_var("REDIS_HOST", "standard-redis");

    let config = ConfigManager::builder()
        .with_postgres(PostgresConfig::default())
        .with_redis(RedisConfig::default())
        .build();

    // TYL variables should take precedence
    assert_eq!(config.postgres().unwrap().host, "tyl-host");
    assert_eq!(config.redis().unwrap().host, "tyl-redis");

    // Cleanup
    std::env::remove_var("TYL_POSTGRES_HOST");
    std::env::remove_var("PGHOST");
    std::env::remove_var("TYL_REDIS_HOST");
    std::env::remove_var("REDIS_HOST");
}

#[test]
fn test_database_url_integration() {
    let test_url = "postgresql://testuser:testpass@testhost:5432/testdb";
    std::env::set_var("DATABASE_URL", test_url);

    let mut config = PostgresConfig::default();
    config.merge_env().unwrap();

    assert_eq!(config.connection_url(), test_url);
    assert!(config.validate().is_ok());

    std::env::remove_var("DATABASE_URL");
}

#[test]
fn test_redis_url_integration() {
    let test_url = "redis://localhost:6380/1";
    std::env::set_var("REDIS_URL", test_url);

    let mut config = RedisConfig::default();
    config.merge_env().unwrap();

    assert_eq!(config.connection_url(), test_url);
    assert!(config.validate().is_ok());

    std::env::remove_var("REDIS_URL");
}

#[test]
fn test_validation_integration() {
    // Test that validation catches missing required values
    let mut invalid_postgres = PostgresConfig::default();
    invalid_postgres.host = "".to_string();
    invalid_postgres.password = "".to_string();

    let config = ConfigManager::builder()
        .with_postgres(invalid_postgres)
        .build();

    let result = config.validate();
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert!(matches!(error, TylError::Validation { .. }));
}

#[test]
fn test_plugin_trait_integration() {
    let postgres = PostgresConfig::default();
    assert_eq!(postgres.name(), "postgres");
    assert_eq!(postgres.env_prefix(), "POSTGRES");

    let redis = RedisConfig::default();
    assert_eq!(redis.name(), "redis");
    assert_eq!(redis.env_prefix(), "REDIS");
}

#[test]
fn test_configuration_serialization() {
    let postgres = PostgresConfig::default();
    let redis = RedisConfig::default();

    // Test that configs can be serialized and deserialized
    let postgres_json = serde_json::to_string(&postgres).unwrap();
    let postgres_restored: PostgresConfig = serde_json::from_str(&postgres_json).unwrap();
    assert_eq!(postgres.host, postgres_restored.host);

    let redis_json = serde_json::to_string(&redis).unwrap();
    let redis_restored: RedisConfig = serde_json::from_str(&redis_json).unwrap();
    assert_eq!(redis.host, redis_restored.host);
}

#[test]
fn test_missing_values_failure_integration() {
    // Integration test showing that plugins fail when required values are missing

    // Clean up any environment variables that could interfere
    std::env::remove_var("TYL_POSTGRES_PASSWORD");
    std::env::remove_var("PGPASSWORD");
    std::env::remove_var("TYL_DATABASE_URL");
    std::env::remove_var("DATABASE_URL");
    std::env::remove_var("POSTGRES_URL");
    std::env::remove_var("TYL_REDIS_HOST");
    std::env::remove_var("REDIS_HOST");

    // Create configs with missing values
    let mut postgres_no_password = PostgresConfig::default();
    postgres_no_password.password = "".to_string();

    let mut redis_no_host = RedisConfig::default();
    redis_no_host.host = "".to_string();

    // Both should fail validation
    assert!(postgres_no_password.validate().is_err());
    assert!(redis_no_host.validate().is_err());

    // ConfigManager should propagate these errors
    let postgres_manager = ConfigManager::builder()
        .with_postgres(postgres_no_password)
        .build();
    assert!(postgres_manager.validate().is_err());

    let redis_manager = ConfigManager::builder().with_redis(redis_no_host).build();
    assert!(redis_manager.validate().is_err());
}

#[test]
fn test_url_fallback_integration() {
    // Test that URL provides fallback when components are missing

    let mut config = PostgresConfig::default();
    config.url = Some("postgresql://fallback:pass@fallback-host:5432/fallbackdb".to_string());
    config.host = "".to_string(); // Would normally fail validation
    config.password = "".to_string(); // Would normally fail validation

    // Should pass validation because URL is set
    assert!(config.validate().is_ok());

    // Should use the URL for connection
    assert_eq!(
        config.connection_url(),
        "postgresql://fallback:pass@fallback-host:5432/fallbackdb"
    );
}

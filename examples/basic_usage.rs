use tyl_config::{ConfigManager, PostgresConfig, RedisConfig, ConfigPlugin};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== TYL Config Basic Usage ===\n");

    // Basic usage example
    basic_usage_example()?;
    
    // Environment variable example
    environment_config_example()?;
    
    // Validation example
    validation_example()?;
    
    // URL vs components example
    url_vs_components_example()?;
    
    Ok(())
}

fn basic_usage_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Basic Usage ---");
    
    // Create config with defaults
    let config = ConfigManager::builder()
        .with_postgres(PostgresConfig::default())
        .with_redis(RedisConfig::default())
        .build();
    
    // Validate all configurations
    config.validate()?;
    
    if let Some(postgres) = config.postgres() {
        println!("✅ PostgreSQL URL: {}", postgres.connection_url());
        println!("  Host: {}, Port: {}", postgres.host, postgres.port);
        println!("  Database: {}, Pool Size: {}", postgres.database, postgres.pool_size);
    }
    
    if let Some(redis) = config.redis() {
        println!("✅ Redis URL: {}", redis.connection_url());
        println!("  Host: {}, Port: {}", redis.host, redis.port);
        println!("  Database: {}, Pool Size: {}", redis.database, redis.pool_size);
    }
    
    println!();
    Ok(())
}

fn environment_config_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Environment Variables Example ---");
    
    // Set some environment variables for demo
    std::env::set_var("TYL_POSTGRES_HOST", "prod-postgres");
    std::env::set_var("TYL_REDIS_HOST", "prod-redis");
    std::env::set_var("TYL_POSTGRES_POOL_SIZE", "20");
    
    let config = ConfigManager::builder()
        .with_postgres(PostgresConfig::default())
        .with_redis(RedisConfig::default())
        .build();
        
    config.validate()?;
    
    if let Some(postgres) = config.postgres() {
        println!("✅ PostgreSQL with env overrides:");
        println!("  Host: {} (from TYL_POSTGRES_HOST)", postgres.host);
        println!("  Pool Size: {} (from TYL_POSTGRES_POOL_SIZE)", postgres.pool_size);
    }
    
    if let Some(redis) = config.redis() {
        println!("✅ Redis with env overrides:");
        println!("  Host: {} (from TYL_REDIS_HOST)", redis.host);
    }
    
    // Cleanup
    std::env::remove_var("TYL_POSTGRES_HOST");
    std::env::remove_var("TYL_REDIS_HOST");
    std::env::remove_var("TYL_POSTGRES_POOL_SIZE");
    
    println!();
    Ok(())
}

fn validation_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Validation Example ---");
    
    // Create invalid config
    let mut invalid_postgres = PostgresConfig::default();
    invalid_postgres.host = "".to_string(); // Invalid: empty host
    invalid_postgres.password = "".to_string(); // Invalid: empty password
    
    let config = ConfigManager::builder()
        .with_postgres(invalid_postgres)
        .build();
    
    // This should fail validation
    match config.validate() {
        Ok(_) => println!("❌ Unexpected: validation should have failed"),
        Err(e) => println!("✅ Validation correctly failed: {}", e),
    }
    
    println!();
    Ok(())
}

fn url_vs_components_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- URL vs Components Example ---");
    
    // Demo 1: Using DATABASE_URL (more lenient validation)
    std::env::set_var("DATABASE_URL", "postgresql://prod_user:secret@prod-host:5432/prod_db");
    
    let mut config_with_url = PostgresConfig::default();
    config_with_url.merge_env()?;
    config_with_url.password = "".to_string(); // This is OK because we have DATABASE_URL
    
    match config_with_url.validate() {
        Ok(_) => println!("✅ Config with DATABASE_URL: validation passed (empty password OK)"),
        Err(e) => println!("❌ Unexpected error: {}", e),
    }
    
    println!("  Connection URL: {}", config_with_url.connection_url());
    
    // Demo 2: Using individual components (strict validation)
    std::env::remove_var("DATABASE_URL");
    
    let mut config_components = PostgresConfig::default();
    config_components.password = "".to_string(); // This will fail without DATABASE_URL
    
    match config_components.validate() {
        Ok(_) => println!("❌ Unexpected: validation should have failed"),
        Err(e) => println!("✅ Config without DATABASE_URL: validation failed as expected: {}", e),
    }
    
    println!();
    Ok(())
}
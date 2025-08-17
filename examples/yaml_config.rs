use tyl_config::{ConfigManager, PostgresConfig, RedisConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== TYL Config YAML Generation ===\n");

    // Demo 1: Generate config template from current defaults
    generate_template_example()?;

    // Demo 2: Load from YAML file and show precedence
    load_from_yaml_example()?;

    // Demo 3: Show complete configuration hierarchy
    configuration_hierarchy_example()?;

    Ok(())
}

fn generate_template_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Generate Configuration Template ---");

    // Create config with defaults
    let config = ConfigManager::builder()
        .with_postgres(PostgresConfig::default())
        .with_redis(RedisConfig::default())
        .build();

    // Generate template file
    let template_path = "/tmp/tyl-config-template.yaml";
    config.generate_config_template(template_path)?;

    println!("✅ Generated configuration template at: {}", template_path);

    // Show the generated content
    let content = std::fs::read_to_string(template_path)?;
    println!("📄 Generated template content:");
    println!("{}", content);

    Ok(())
}

fn load_from_yaml_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Load Configuration from YAML ---");

    // Create a custom YAML config file
    let custom_yaml = r#"
postgres:
  host: yaml-postgres-host
  port: 5433
  database: yaml_db
  username: yaml_user
  password: yaml_password
  pool_size: 15
  timeout_seconds: 45

redis:
  host: yaml-redis-host
  port: 6380
  database: 2
  pool_size: 8
  timeout_seconds: 20
"#;

    let yaml_path = "/tmp/custom-config.yaml";
    std::fs::write(yaml_path, custom_yaml)?;

    // Load configuration from YAML file
    let config = ConfigManager::builder().with_yaml_file(yaml_path)?.build();

    println!("✅ Loaded configuration from YAML:");
    if let Some(postgres) = config.postgres() {
        println!("  PostgreSQL Host: {} (from YAML)", postgres.host);
        println!("  PostgreSQL Port: {} (from YAML)", postgres.port);
        println!("  PostgreSQL Pool: {} (from YAML)", postgres.pool_size);
    }

    if let Some(redis) = config.redis() {
        println!("  Redis Host: {} (from YAML)", redis.host);
        println!("  Redis Port: {} (from YAML)", redis.port);
        println!("  Redis Database: {} (from YAML)", redis.database);
    }

    println!();
    Ok(())
}

fn configuration_hierarchy_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Configuration Hierarchy Demo ---");

    // Create YAML with base values
    let yaml_content = r#"
postgres:
  host: yaml-host
  port: 5432
  database: yaml_db
  username: yaml_user
  password: yaml_pass
  pool_size: 10
  timeout_seconds: 30

redis:
  host: yaml-redis
  port: 6379
  database: 0
  pool_size: 5
  timeout_seconds: 10
"#;

    let yaml_path = "/tmp/hierarchy-test.yaml";
    std::fs::write(yaml_path, yaml_content)?;

    // Set some environment variables (higher priority)
    std::env::set_var("TYL_POSTGRES_HOST", "env-postgres");
    std::env::set_var("PGUSER", "env-user");
    std::env::set_var("TYL_REDIS_PORT", "6380");

    // Load with hierarchy: env vars > YAML > defaults
    let config = ConfigManager::builder().with_yaml_file(yaml_path)?.build();

    println!("📊 Configuration Hierarchy (priority: env vars > YAML > defaults):");

    if let Some(postgres) = config.postgres() {
        println!("\n🐘 PostgreSQL Configuration:");
        println!(
            "  host: {} (source: {})",
            postgres.host,
            if postgres.host == "env-postgres" {
                "TYL_POSTGRES_HOST env var"
            } else {
                "YAML/default"
            }
        );
        println!(
            "  username: {} (source: {})",
            postgres.username,
            if postgres.username == "env-user" {
                "PGUSER env var"
            } else {
                "YAML/default"
            }
        );
        println!("  database: {} (source: YAML)", postgres.database);
        println!("  port: {} (source: YAML)", postgres.port);
        println!("  pool_size: {} (source: YAML)", postgres.pool_size);
    }

    if let Some(redis) = config.redis() {
        println!("\n🔴 Redis Configuration:");
        println!("  host: {} (source: YAML)", redis.host);
        println!(
            "  port: {} (source: {})",
            redis.port,
            if redis.port == 6380 {
                "TYL_REDIS_PORT env var"
            } else {
                "YAML/default"
            }
        );
        println!("  database: {} (source: YAML)", redis.database);
        println!("  pool_size: {} (source: YAML)", redis.pool_size);
    }

    // Generate final resolved config
    let final_template_path = "/tmp/resolved-config.yaml";
    config.generate_config_template(final_template_path)?;

    println!(
        "\n✅ Generated resolved configuration at: {}",
        final_template_path
    );
    println!("💡 This file shows the final resolved values after applying hierarchy");

    // Cleanup env vars
    std::env::remove_var("TYL_POSTGRES_HOST");
    std::env::remove_var("PGUSER");
    std::env::remove_var("TYL_REDIS_PORT");

    println!();
    Ok(())
}

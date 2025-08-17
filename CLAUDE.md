# CLAUDE.md - tyl-config

## 📋 **Module Context**

**tyl-config** is the configuration management module for the TYL framework with hierarchical loading, environment variable precedence, and YAML generation.

## 🏗️ **Architecture**

### **Port (Interface)**
```rust
trait ConfigPlugin {
    fn name(&self) -> &'static str;
    fn env_prefix(&self) -> &'static str;
    fn validate(&self) -> ConfigResult<()>;
    fn merge_env(&mut self) -> ConfigResult<()>;
    fn to_yaml_value(&self) -> ConfigResult<serde_yaml::Value>;
    fn from_yaml(value: &serde_yaml::Value) -> ConfigResult<Self> where Self: Sized;
}
```

### **Adapters (Implementations)**
- `PostgresConfig` - PostgreSQL database configuration
- `RedisConfig` - Redis cache configuration
- `ConfigManager` - Main configuration coordinator

### **Core Types**
- `ConfigManager` - Main configuration manager
- `ConfigError` - Error types with thiserror
- `ConfigResult<T>` - Result type alias

## 🧪 **Testing**

```bash
cargo test -p tyl-config
cargo test --doc -p tyl-config
cargo run --example basic_usage -p tyl-config
cargo run --example yaml_config -p tyl-config
```

## 📂 **File Structure**

```
tyl-config/
├── src/lib.rs                 # Core implementation
├── examples/
│   ├── basic_usage.rs         # Basic usage example
│   └── yaml_config.rs         # YAML generation and loading
├── tests/
│   └── integration_tests.rs   # Integration tests
├── README.md                  # Main documentation
├── CLAUDE.md                  # This file
└── Cargo.toml                 # Package metadata
```

## 🔧 **How to Use**

### **Basic Usage**
```rust
use tyl_config::{ConfigManager, PostgresConfig, RedisConfig};

let config = ConfigManager::builder()
    .with_postgres(PostgresConfig::default())
    .with_redis(RedisConfig::default())
    .build();

// Access configuration
if let Some(pg) = config.postgres() {
    println!("Database URL: {}", pg.database_url());
}
```

### **Custom Configuration Plugin**
```rust
use tyl_config::{ConfigPlugin, ConfigResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MyServiceConfig {
    pub host: String,
    pub port: u16,
}

impl ConfigPlugin for MyServiceConfig {
    fn name(&self) -> &'static str { "myservice" }
    fn env_prefix(&self) -> &'static str { "TYL_MYSERVICE" }
    
    fn validate(&self) -> ConfigResult<()> {
        // Validation logic
        Ok(())
    }
    
    fn merge_env(&mut self) -> ConfigResult<()> {
        // Environment variable merging
        Ok(())
    }
    
    fn to_yaml_value(&self) -> ConfigResult<serde_yaml::Value> {
        Ok(serde_yaml::to_value(self)?)
    }
    
    fn from_yaml(value: &serde_yaml::Value) -> ConfigResult<Self> {
        Ok(serde_yaml::from_value(value.clone())?)
    }
}
```

## 🛠️ **Useful Commands**

```bash
cargo clippy -p tyl-config
cargo fmt -p tyl-config  
cargo doc --no-deps -p tyl-config --open
cargo test -p tyl-config --verbose
```

## 📦 **Dependencies**

### **Runtime**
- `serde` - Serialization support with derive features
- `serde_yaml` - YAML file handling and generation
- `thiserror` - Error handling and propagation
- `uuid` - Unique identifier generation

### **Development**
- Standard Rust testing framework
- Temporary file handling for tests

## 🎯 **Design Principles**

1. **Configuration Hierarchy** - Environment variables > YAML > defaults
2. **TYL Prefix Priority** - TYL_* variables take precedence over standard ones
3. **Plugin Architecture** - Extensible via ConfigPlugin trait
4. **YAML Generation** - Automatic template generation for documentation
5. **Validation** - Built-in validation with custom error messages

## ⚠️ **Known Limitations**

- Currently supports PostgreSQL and Redis configurations
- YAML generation includes all plugins, even unused ones
- Environment variable parsing is string-based

## 📝 **Notes for Contributors**

- Follow TDD approach
- Maintain hexagonal architecture
- Document all public APIs with examples
- Add integration tests for new features
- Keep dependencies minimal

## 🔗 **Related TYL Modules**

- [`tyl-errors`](https://github.com/the-yaml-life/tyl-errors) - Error handling
- [`tyl-logging`](https://github.com/the-yaml-life/tyl-logging) - Structured logging
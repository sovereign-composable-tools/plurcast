# Testing Quick Start Guide - Plurcast

Quick reference for implementing tests in the Plurcast codebase.

## Setup

### 1. Add Test Dependencies

Already in workspace `Cargo.toml`:
```toml
[workspace.dependencies]
tempfile = "3.10"
```

Add to `plur-post/Cargo.toml`:
```toml
[dev-dependencies]
assert_cmd = "2.0"
predicates = "3.0"
tempfile = { workspace = true }
```

### 2. Run Tests

```bash
# All tests
cargo test --workspace

# Specific module
cargo test --package libplurcast --lib config

# With output
cargo test -- --nocapture

# Integration tests only
cargo test --test '*'
```

## Test Patterns

### Unit Test Template

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_name() {
        // Arrange
        let input = "test";
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert_eq!(result, expected);
    }
}
```

### Async Test Template

```rust
#[tokio::test]
async fn test_async_function() {
    let result = async_function().await;
    assert!(result.is_ok());
}
```

### Database Test Template

```rust
#[tokio::test]
async fn test_database_operation() {
    // Use in-memory database
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    let db = Database { pool };
    
    // Enable foreign keys
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&db.pool)
        .await
        .unwrap();
    
    // Test operation
    let result = db.some_operation().await;
    assert!(result.is_ok());
}
```

### File-Based Test Template

```rust
#[test]
fn test_file_operation() {
    use tempfile::TempDir;
    
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    
    // Test file operations
    std::fs::write(&file_path, "content").unwrap();
    
    // Cleanup happens automatically when temp_dir drops
}
```

### Error Test Template

```rust
#[test]
fn test_error_handling() {
    let result = function_that_fails();
    
    assert!(result.is_err());
    
    match result {
        Err(PlurcastError::Platform(PlatformError::Authentication(msg))) => {
            assert!(msg.contains("expected text"));
        }
        _ => panic!("Expected authentication error"),
    }
}
```

### CLI Integration Test Template

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_cli_command() {
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    cmd.arg("--help");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("expected text"));
}
```

## Common Test Scenarios

### Testing Configuration Loading

```rust
#[test]
fn test_config_parsing() {
    let toml = r#"
        [database]
        path = "/tmp/test.db"
    "#;
    
    let config: Config = toml::from_str(toml).unwrap();
    assert_eq!(config.database.path, "/tmp/test.db");
}
```

### Testing Environment Variables

```rust
#[test]
fn test_env_override() {
    use std::env;
    
    env::set_var("PLURCAST_CONFIG", "/custom/path");
    let path = resolve_config_path().unwrap();
    env::remove_var("PLURCAST_CONFIG");
    
    assert_eq!(path.to_str().unwrap(), "/custom/path");
}
```

### Testing File Permissions (Unix)

```rust
#[test]
#[cfg(unix)]
fn test_file_permissions() {
    use std::os::unix::fs::PermissionsExt;
    
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    
    create_secure_file(&file_path).unwrap();
    
    let metadata = std::fs::metadata(&file_path).unwrap();
    assert_eq!(metadata.permissions().mode() & 0o777, 0o600);
}
```

### Testing Nostr Key Parsing

```rust
#[test]
fn test_hex_key_parsing() {
    use tempfile::NamedTempFile;
    use std::io::Write;
    
    let mut temp_file = NamedTempFile::new().unwrap();
    let hex_key = "a".repeat(64);
    writeln!(temp_file, "{}", hex_key).unwrap();
    
    let mut platform = NostrPlatform::new(&config);
    let result = platform.load_keys(temp_file.path().to_str().unwrap());
    
    // Check result based on actual nostr-sdk behavior
}
```

### Testing Database CRUD

```rust
#[tokio::test]
async fn test_create_and_retrieve() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    let db = Database { pool };
    
    let post = Post::new("test content".to_string());
    db.create_post(&post).await.unwrap();
    
    let retrieved = db.get_post(&post.id).await.unwrap().unwrap();
    assert_eq!(retrieved.content, "test content");
}
```

### Testing CLI Exit Codes

```rust
#[test]
fn test_exit_code_on_error() {
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    cmd.arg(""); // Empty content
    
    cmd.assert()
        .failure()
        .code(3); // InvalidInput exit code
}
```

### Testing JSON Output

```rust
#[test]
fn test_json_output() {
    let mut cmd = Command::cargo_bin("plur-post").unwrap();
    cmd.arg("--format").arg("json");
    cmd.arg("--draft");
    cmd.arg("Test");
    
    let output = cmd.output().unwrap();
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    
    assert_eq!(json["status"], "draft");
}
```

## Test Data Helpers

### Create Test Post

```rust
fn create_test_post() -> Post {
    Post {
        id: uuid::Uuid::new_v4().to_string(),
        content: "Test post content".to_string(),
        created_at: chrono::Utc::now().timestamp(),
        scheduled_at: None,
        status: PostStatus::Pending,
        metadata: None,
    }
}
```

### Create Test Config

```rust
fn create_test_config() -> Config {
    Config {
        database: DatabaseConfig {
            path: ":memory:".to_string(),
        },
        nostr: Some(NostrConfig {
            enabled: true,
            keys_file: "/tmp/test.keys".to_string(),
            relays: vec!["wss://relay.test".to_string()],
        }),
        defaults: DefaultsConfig {
            platforms: vec!["nostr".to_string()],
        },
    }
}
```

## Debugging Tests

### Print Test Output

```bash
# Show println! output
cargo test -- --nocapture

# Show specific test
cargo test test_name -- --nocapture
```

### Run Single Test

```bash
cargo test test_name
```

### Run Tests in Module

```bash
cargo test --package libplurcast --lib config::tests
```

## Common Assertions

```rust
// Equality
assert_eq!(actual, expected);
assert_ne!(actual, unexpected);

// Boolean
assert!(condition);
assert!(!condition);

// Result/Option
assert!(result.is_ok());
assert!(result.is_err());
assert!(option.is_some());
assert!(option.is_none());

// String contains
assert!(string.contains("substring"));

// Pattern matching
assert!(matches!(value, Pattern::Variant));

// Panic expected
#[should_panic(expected = "error message")]
```

## Next Steps

1. Start with `config.rs` tests (easiest)
2. Add `error.rs` tests (simple)
3. Add `types.rs` tests (straightforward)
4. Add `platforms/nostr.rs` tests (complex)
5. Add CLI integration tests (requires setup)

## Resources

- [Rust Book - Testing](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [assert_cmd docs](https://docs.rs/assert_cmd/)
- [tempfile docs](https://docs.rs/tempfile/)
- [tokio testing](https://tokio.rs/tokio/topics/testing)

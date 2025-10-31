//! Account management for multi-account credential support
//!
//! This module provides functionality for managing multiple named accounts per platform,
//! tracking active accounts, and persisting account state.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use crate::error::{AccountError, Result};

/// Account manager for handling multi-account state
///
/// Manages account registration, active account tracking, and state persistence.
/// Thread-safe via Arc<RwLock<AccountState>>.
#[derive(Clone)]
pub struct AccountManager {
    /// Path to the state file (accounts.toml)
    state_file: PathBuf,
    /// Account state with thread-safe access
    state: Arc<RwLock<AccountState>>,
}

/// Account state structure persisted to TOML
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccountState {
    /// Active account per platform (platform -> account_name)
    #[serde(default)]
    pub active: HashMap<String, String>,
    
    /// Registered accounts per platform
    #[serde(default)]
    pub accounts: HashMap<String, PlatformAccounts>,
}

/// Platform-specific account registry
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlatformAccounts {
    /// List of account names for this platform
    pub names: Vec<String>,
}

impl AccountManager {
    /// Create new AccountManager with default state file location
    ///
    /// Uses XDG Base Directory spec: ~/.config/plurcast/accounts.toml
    pub fn new() -> Result<Self> {
        let state_file = Self::resolve_state_file_path()?;
        Self::with_path(state_file)
    }
    
    /// Create AccountManager with custom state file path
    pub fn with_path(state_file: PathBuf) -> Result<Self> {
        let mut manager = Self {
            state_file,
            state: Arc::new(RwLock::new(AccountState::default())),
        };
        
        // Load existing state if file exists
        manager.load()?;
        
        Ok(manager)
    }
    
    /// Resolve the state file path using XDG Base Directory spec
    fn resolve_state_file_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| AccountError::StateFile("XDG config directory not found".to_string()))?;
        
        Ok(config_dir.join("plurcast").join("accounts.toml"))
    }
    
    /// Validate account name format
    ///
    /// Rules:
    /// - Alphanumeric characters, hyphens, and underscores only
    /// - Maximum 64 characters
    /// - Cannot be empty
    /// - Cannot be a reserved name
    pub fn validate_account_name(name: &str) -> Result<()> {
        // Check if empty
        if name.is_empty() {
            return Err(AccountError::InvalidName("Account name cannot be empty".to_string()).into());
        }
        
        // Check length
        if name.len() > 64 {
            return Err(AccountError::InvalidName(
                format!("Account name too long: {} characters (max 64)", name.len())
            ).into());
        }
        
        // Check for valid characters (alphanumeric, hyphens, underscores)
        if !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err(AccountError::InvalidName(
                format!("Invalid account name '{}'. Must be alphanumeric with hyphens/underscores only", name)
            ).into());
        }
        
        // Check for reserved names
        let reserved_names = vec!["all", "none", "list"];
        if reserved_names.contains(&name.to_lowercase().as_str()) {
            return Err(AccountError::ReservedName(name.to_string()).into());
        }
        
        Ok(())
    }

    /// Get active account for platform
    ///
    /// Returns "default" if no active account is set
    pub fn get_active_account(&self, platform: &str) -> String {
        let state = self.state.read().unwrap();
        state.active
            .get(platform)
            .cloned()
            .unwrap_or_else(|| "default".to_string())
    }
    
    /// Set active account for platform
    ///
    /// Validates that the account exists before setting it as active.
    /// Persists the state to disk.
    pub fn set_active_account(&self, platform: &str, account: &str) -> Result<()> {
        // Validate account name
        Self::validate_account_name(account)?;
        
        // Check if account exists
        if !self.account_exists(platform, account) {
            return Err(AccountError::NotFound(account.to_string(), platform.to_string()).into());
        }
        
        // Update state
        {
            let mut state = self.state.write().unwrap();
            state.active.insert(platform.to_string(), account.to_string());
        }
        
        // Persist to disk
        self.save()?;
        
        Ok(())
    }
    
    /// List all accounts for a platform
    pub fn list_accounts(&self, platform: &str) -> Vec<String> {
        let state = self.state.read().unwrap();
        state.accounts
            .get(platform)
            .map(|pa| pa.names.clone())
            .unwrap_or_default()
    }
    
    /// Register an account for a platform
    ///
    /// Called when credentials are stored for an account.
    /// Validates account name and adds to registry if not already present.
    pub fn register_account(&self, platform: &str, account: &str) -> Result<()> {
        // Validate account name
        Self::validate_account_name(account)?;
        
        // Update state
        {
            let mut state = self.state.write().unwrap();
            let platform_accounts = state.accounts
                .entry(platform.to_string())
                .or_insert_with(PlatformAccounts::default);
            
            // Add account if not already present
            if !platform_accounts.names.contains(&account.to_string()) {
                platform_accounts.names.push(account.to_string());
            }
        }
        
        // Persist to disk
        self.save()?;
        
        Ok(())
    }
    
    /// Unregister an account from a platform
    ///
    /// Called when credentials are deleted for an account.
    /// If the account is active, resets active account to "default".
    pub fn unregister_account(&self, platform: &str, account: &str) -> Result<()> {
        // Update state
        {
            let mut state = self.state.write().unwrap();
            
            // Remove from registry
            if let Some(platform_accounts) = state.accounts.get_mut(platform) {
                platform_accounts.names.retain(|a| a != account);
            }
            
            // Reset active account if this was the active one
            if let Some(active) = state.active.get(platform) {
                if active == account {
                    state.active.insert(platform.to_string(), "default".to_string());
                }
            }
        }
        
        // Persist to disk
        self.save()?;
        
        Ok(())
    }
    
    /// Check if account exists for platform
    pub fn account_exists(&self, platform: &str, account: &str) -> bool {
        let state = self.state.read().unwrap();
        state.accounts
            .get(platform)
            .map(|pa| pa.names.contains(&account.to_string()))
            .unwrap_or(false)
    }
    
    /// Save state to disk
    ///
    /// Serializes state to TOML and writes to state file.
    /// Creates parent directories if needed.
    /// Sets file permissions to 644 on Unix.
    fn save(&self) -> Result<()> {
        // Create parent directories if they don't exist
        if let Some(parent) = self.state_file.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| AccountError::StateFile(format!("Failed to create directory: {}", e)))?;
        }
        
        // Serialize state to TOML
        let state = self.state.read().unwrap();
        let toml_content = toml::to_string_pretty(&*state)
            .map_err(|e| AccountError::StateFile(format!("Failed to serialize state: {}", e)))?;
        
        // Write to file
        std::fs::write(&self.state_file, toml_content)
            .map_err(|e| AccountError::StateFile(format!("Failed to write state file: {}", e)))?;
        
        // Set file permissions to 644 on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = std::fs::Permissions::from_mode(0o644);
            std::fs::set_permissions(&self.state_file, permissions)
                .map_err(|e| AccountError::StateFile(format!("Failed to set permissions: {}", e)))?;
        }
        
        Ok(())
    }
    
    /// Load state from disk
    ///
    /// Handles missing file gracefully by using default state.
    /// Handles corrupted file gracefully by logging warning and using default state.
    fn load(&mut self) -> Result<()> {
        // If file doesn't exist, use default state
        if !self.state_file.exists() {
            return Ok(());
        }
        
        // Read file content
        let content = std::fs::read_to_string(&self.state_file)
            .map_err(|e| AccountError::StateFile(format!("Failed to read state file: {}", e)))?;
        
        // Parse TOML
        match toml::from_str::<AccountState>(&content) {
            Ok(loaded_state) => {
                let mut state = self.state.write().unwrap();
                *state = loaded_state;
                Ok(())
            }
            Err(e) => {
                // Log warning but don't fail - use default state
                tracing::warn!("Corrupted account state file, using defaults: {}", e);
                Ok(())
            }
        }
    }
}

impl Default for AccountManager {
    fn default() -> Self {
        Self::new().expect("Failed to create default AccountManager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::thread;
    use std::sync::Arc;

    #[test]
    fn test_validate_account_name_valid() {
        assert!(AccountManager::validate_account_name("default").is_ok());
        assert!(AccountManager::validate_account_name("test-account").is_ok());
        assert!(AccountManager::validate_account_name("prod_123").is_ok());
        assert!(AccountManager::validate_account_name("work").is_ok());
        assert!(AccountManager::validate_account_name("a1-b2_c3").is_ok());
    }

    #[test]
    fn test_validate_account_name_invalid() {
        // Empty name
        assert!(AccountManager::validate_account_name("").is_err());
        
        // Too long (>64 chars)
        let long_name = "a".repeat(65);
        assert!(AccountManager::validate_account_name(&long_name).is_err());
        
        // Invalid characters
        assert!(AccountManager::validate_account_name("test account").is_err()); // space
        assert!(AccountManager::validate_account_name("test@account").is_err()); // @
        assert!(AccountManager::validate_account_name("test.account").is_err()); // .
        assert!(AccountManager::validate_account_name("test/account").is_err()); // /
        
        // Reserved names
        assert!(AccountManager::validate_account_name("all").is_err());
        assert!(AccountManager::validate_account_name("none").is_err());
        assert!(AccountManager::validate_account_name("list").is_err());
        assert!(AccountManager::validate_account_name("ALL").is_err()); // case insensitive
    }

    #[test]
    fn test_account_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let state_file = temp_dir.path().join("accounts.toml");
        
        let manager = AccountManager::with_path(state_file.clone()).unwrap();
        
        // Should have default state
        assert_eq!(manager.get_active_account("nostr"), "default");
        assert!(manager.list_accounts("nostr").is_empty());
    }

    #[test]
    fn test_register_account() {
        let temp_dir = TempDir::new().unwrap();
        let state_file = temp_dir.path().join("accounts.toml");
        
        let manager = AccountManager::with_path(state_file.clone()).unwrap();
        
        // Register account
        manager.register_account("nostr", "test-account").unwrap();
        
        // Verify account exists
        assert!(manager.account_exists("nostr", "test-account"));
        assert_eq!(manager.list_accounts("nostr"), vec!["test-account"]);
        
        // Verify state file was created
        assert!(state_file.exists());
    }

    #[test]
    fn test_register_account_duplicate() {
        let temp_dir = TempDir::new().unwrap();
        let state_file = temp_dir.path().join("accounts.toml");
        
        let manager = AccountManager::with_path(state_file).unwrap();
        
        // Register same account twice
        manager.register_account("nostr", "test-account").unwrap();
        manager.register_account("nostr", "test-account").unwrap();
        
        // Should only appear once
        assert_eq!(manager.list_accounts("nostr"), vec!["test-account"]);
    }

    #[test]
    fn test_unregister_account() {
        let temp_dir = TempDir::new().unwrap();
        let state_file = temp_dir.path().join("accounts.toml");
        
        let manager = AccountManager::with_path(state_file).unwrap();
        
        // Register and then unregister
        manager.register_account("nostr", "test-account").unwrap();
        assert!(manager.account_exists("nostr", "test-account"));
        
        manager.unregister_account("nostr", "test-account").unwrap();
        assert!(!manager.account_exists("nostr", "test-account"));
        assert!(manager.list_accounts("nostr").is_empty());
    }

    #[test]
    fn test_get_active_account_default() {
        let temp_dir = TempDir::new().unwrap();
        let state_file = temp_dir.path().join("accounts.toml");
        
        let manager = AccountManager::with_path(state_file).unwrap();
        
        // Should return "default" when no active account set
        assert_eq!(manager.get_active_account("nostr"), "default");
        assert_eq!(manager.get_active_account("mastodon"), "default");
    }

    #[test]
    fn test_set_active_account() {
        let temp_dir = TempDir::new().unwrap();
        let state_file = temp_dir.path().join("accounts.toml");
        
        let manager = AccountManager::with_path(state_file).unwrap();
        
        // Register account first
        manager.register_account("nostr", "test-account").unwrap();
        
        // Set as active
        manager.set_active_account("nostr", "test-account").unwrap();
        
        // Verify active account
        assert_eq!(manager.get_active_account("nostr"), "test-account");
    }

    #[test]
    fn test_set_active_account_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let state_file = temp_dir.path().join("accounts.toml");
        
        let manager = AccountManager::with_path(state_file).unwrap();
        
        // Try to set non-existent account as active
        let result = manager.set_active_account("nostr", "nonexistent");
        
        assert!(result.is_err());
        match result {
            Err(crate::error::PlurcastError::Account(AccountError::NotFound(account, platform))) => {
                assert_eq!(account, "nonexistent");
                assert_eq!(platform, "nostr");
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_unregister_active_account_resets_to_default() {
        let temp_dir = TempDir::new().unwrap();
        let state_file = temp_dir.path().join("accounts.toml");
        
        let manager = AccountManager::with_path(state_file).unwrap();
        
        // Register and set as active
        manager.register_account("nostr", "test-account").unwrap();
        manager.set_active_account("nostr", "test-account").unwrap();
        assert_eq!(manager.get_active_account("nostr"), "test-account");
        
        // Unregister the active account
        manager.unregister_account("nostr", "test-account").unwrap();
        
        // Should reset to "default"
        assert_eq!(manager.get_active_account("nostr"), "default");
    }

    #[test]
    fn test_state_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let state_file = temp_dir.path().join("accounts.toml");
        
        // Create manager and register accounts
        {
            let manager = AccountManager::with_path(state_file.clone()).unwrap();
            manager.register_account("nostr", "test-account").unwrap();
            manager.register_account("nostr", "prod-account").unwrap();
            manager.set_active_account("nostr", "test-account").unwrap();
        }
        
        // Create new manager with same state file
        {
            let manager = AccountManager::with_path(state_file).unwrap();
            
            // Verify state was loaded
            assert!(manager.account_exists("nostr", "test-account"));
            assert!(manager.account_exists("nostr", "prod-account"));
            assert_eq!(manager.get_active_account("nostr"), "test-account");
            
            let accounts = manager.list_accounts("nostr");
            assert_eq!(accounts.len(), 2);
            assert!(accounts.contains(&"test-account".to_string()));
            assert!(accounts.contains(&"prod-account".to_string()));
        }
    }

    #[test]
    fn test_missing_state_file_graceful() {
        let temp_dir = TempDir::new().unwrap();
        let state_file = temp_dir.path().join("nonexistent.toml");
        
        // Should not fail when file doesn't exist
        let manager = AccountManager::with_path(state_file).unwrap();
        
        // Should have default state
        assert_eq!(manager.get_active_account("nostr"), "default");
        assert!(manager.list_accounts("nostr").is_empty());
    }

    #[test]
    fn test_corrupted_state_file_graceful() {
        let temp_dir = TempDir::new().unwrap();
        let state_file = temp_dir.path().join("corrupted.toml");
        
        // Write invalid TOML
        std::fs::write(&state_file, "invalid toml {{{").unwrap();
        
        // Should not fail, should use default state
        let manager = AccountManager::with_path(state_file).unwrap();
        
        // Should have default state
        assert_eq!(manager.get_active_account("nostr"), "default");
        assert!(manager.list_accounts("nostr").is_empty());
    }

    #[test]
    #[cfg(unix)]
    fn test_state_file_permissions() {
        use std::os::unix::fs::PermissionsExt;
        
        let temp_dir = TempDir::new().unwrap();
        let state_file = temp_dir.path().join("accounts.toml");
        
        let manager = AccountManager::with_path(state_file.clone()).unwrap();
        manager.register_account("nostr", "test-account").unwrap();
        
        // Check file permissions
        let metadata = std::fs::metadata(&state_file).unwrap();
        let permissions = metadata.permissions();
        
        // Should be 644 (owner read/write, group/others read)
        assert_eq!(permissions.mode() & 0o777, 0o644);
    }

    #[test]
    fn test_multiple_platforms() {
        let temp_dir = TempDir::new().unwrap();
        let state_file = temp_dir.path().join("accounts.toml");
        
        let manager = AccountManager::with_path(state_file).unwrap();
        
        // Register accounts for different platforms
        manager.register_account("nostr", "test-nostr").unwrap();
        manager.register_account("mastodon", "test-mastodon").unwrap();
        manager.register_account("bluesky", "test-bluesky").unwrap();
        
        // Set active accounts
        manager.set_active_account("nostr", "test-nostr").unwrap();
        manager.set_active_account("mastodon", "test-mastodon").unwrap();
        
        // Verify each platform has correct state
        assert_eq!(manager.get_active_account("nostr"), "test-nostr");
        assert_eq!(manager.get_active_account("mastodon"), "test-mastodon");
        assert_eq!(manager.get_active_account("bluesky"), "default");
        
        assert_eq!(manager.list_accounts("nostr"), vec!["test-nostr"]);
        assert_eq!(manager.list_accounts("mastodon"), vec!["test-mastodon"]);
        assert_eq!(manager.list_accounts("bluesky"), vec!["test-bluesky"]);
    }

    #[test]
    fn test_thread_safety() {
        let temp_dir = TempDir::new().unwrap();
        let state_file = temp_dir.path().join("accounts.toml");
        
        let manager = Arc::new(AccountManager::with_path(state_file).unwrap());
        
        // Spawn multiple threads that register accounts
        let mut handles = vec![];
        
        for i in 0..10 {
            let manager_clone = Arc::clone(&manager);
            let handle = thread::spawn(move || {
                let account_name = format!("account-{}", i);
                manager_clone.register_account("nostr", &account_name).unwrap();
            });
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Verify all accounts were registered
        let accounts = manager.list_accounts("nostr");
        assert_eq!(accounts.len(), 10);
        
        for i in 0..10 {
            let account_name = format!("account-{}", i);
            assert!(accounts.contains(&account_name));
        }
    }

    #[test]
    fn test_account_exists() {
        let temp_dir = TempDir::new().unwrap();
        let state_file = temp_dir.path().join("accounts.toml");
        
        let manager = AccountManager::with_path(state_file).unwrap();
        
        // Initially doesn't exist
        assert!(!manager.account_exists("nostr", "test-account"));
        
        // Register account
        manager.register_account("nostr", "test-account").unwrap();
        
        // Now exists
        assert!(manager.account_exists("nostr", "test-account"));
        
        // Different platform doesn't exist
        assert!(!manager.account_exists("mastodon", "test-account"));
    }
}

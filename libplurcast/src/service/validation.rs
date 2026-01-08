//! Content validation service
//!
//! Provides real-time validation of content against platform requirements,
//! including character limits, content size, and empty content checks.

use crate::Config;
use std::collections::HashMap;
use std::sync::Arc;

/// Maximum content size in bytes (100KB)
const MAX_CONTENT_LENGTH: usize = 100 * 1024;

/// Character limits for platforms
const NOSTR_CHAR_LIMIT: Option<usize> = None; // No hard limit, warn at 280
const NOSTR_WARN_LIMIT: usize = 280;
const MASTODON_DEFAULT_CHAR_LIMIT: usize = 500;

/// Service for validating content against platform requirements
///
/// Validates content in real-time before posting, checking:
/// - Empty or whitespace-only content
/// - Content size (MAX_CONTENT_LENGTH = 100KB)
/// - Platform-specific character limits
///
/// # Example
///
/// ```no_run
/// use libplurcast::service::validation::{ValidationService, ValidationRequest};
/// use libplurcast::Config;
/// use std::sync::Arc;
///
/// # fn example() -> libplurcast::Result<()> {
/// let config = Config::load()?;
/// let service = ValidationService::new(Arc::new(config));
///
/// let request = ValidationRequest {
///     content: "Hello decentralized world!".to_string(),
///     platforms: vec!["nostr".to_string(), "mastodon".to_string()],
///     auto_thread: false,
/// };
///
/// let response = service.validate(request);
/// if response.valid {
///     println!("Content is valid for all platforms");
/// } else {
///     for result in response.results {
///         if !result.valid {
///             println!("{}: {:?}", result.platform, result.errors);
///         }
///     }
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct ValidationService {
    #[allow(dead_code)] // Reserved for future use (instance-specific limits)
    config: Arc<Config>,
}

/// Request to validate content for specific platforms
///
/// TODO: Consider refactoring `auto_thread` out of ValidationRequest.
/// Auto-threading is a posting concern, not a validation concern. The validation
/// layer could instead expose a `validate_with_options(skip_char_limits: bool)`
/// method, keeping the request struct focused on what to validate rather than
/// how it will be posted.
#[derive(Debug, Clone)]
pub struct ValidationRequest {
    /// Content to validate
    pub content: String,
    /// Platforms to validate against
    pub platforms: Vec<String>,
    /// If true, content will be auto-threaded (skip character limit checks)
    pub auto_thread: bool,
}

/// Response containing validation results
#[derive(Debug, Clone)]
pub struct ValidationResponse {
    /// Whether content is valid for all requested platforms
    pub valid: bool,
    /// Per-platform validation results
    pub results: Vec<PlatformValidation>,
}

/// Validation result for a single platform
#[derive(Debug, Clone)]
pub struct PlatformValidation {
    /// Platform name
    pub platform: String,
    /// Whether content is valid for this platform
    pub valid: bool,
    /// Validation errors (if any)
    pub errors: Vec<String>,
    /// Validation warnings (non-blocking)
    pub warnings: Vec<String>,
}

impl ValidationService {
    /// Create a new validation service
    ///
    /// # Arguments
    ///
    /// * `config` - Shared configuration containing platform settings
    pub fn new(config: Arc<Config>) -> Self {
        Self { config }
    }

    /// Validate content for specified platforms
    ///
    /// Checks content against all validation rules:
    /// - Empty/whitespace-only content
    /// - Content size (MAX_CONTENT_LENGTH)
    /// - Platform-specific character limits (unless auto_thread is enabled)
    ///
    /// # Arguments
    ///
    /// * `request` - Validation request with content and platforms
    ///
    /// # Returns
    ///
    /// Validation response with per-platform results
    pub fn validate(&self, request: ValidationRequest) -> ValidationResponse {
        let mut results = Vec::new();
        let mut all_valid = true;

        for platform in &request.platforms {
            let validation =
                self.validate_for_platform(&request.content, platform, request.auto_thread);
            if !validation.valid {
                all_valid = false;
            }
            results.push(validation);
        }

        ValidationResponse {
            valid: all_valid,
            results,
        }
    }

    /// Check if content is valid for all specified platforms
    ///
    /// Convenience method that returns a simple boolean.
    ///
    /// # Arguments
    ///
    /// * `content` - Content to validate
    /// * `platforms` - Platforms to validate against
    ///
    /// # Returns
    ///
    /// `true` if content is valid for all platforms, `false` otherwise
    pub fn is_valid(&self, content: &str, platforms: &[String]) -> bool {
        let request = ValidationRequest {
            content: content.to_string(),
            platforms: platforms.to_vec(),
            auto_thread: false,
        };
        self.validate(request).valid
    }

    /// Get character limits for specified platforms
    ///
    /// Returns a map of platform names to their character limits.
    /// `None` indicates no hard limit.
    ///
    /// # Arguments
    ///
    /// * `platforms` - Platforms to get limits for
    ///
    /// # Returns
    ///
    /// Map of platform names to character limits (None = no limit)
    pub fn get_limits(&self, platforms: &[String]) -> HashMap<String, Option<usize>> {
        let mut limits = HashMap::new();

        for platform in platforms {
            let limit = match platform.as_str() {
                "nostr" => NOSTR_CHAR_LIMIT,
                "mastodon" => Some(self.get_mastodon_char_limit()),
                "ssb" => None, // SSB has no hard limit
                _ => None,
            };
            limits.insert(platform.clone(), limit);
        }

        limits
    }

    /// Validate content for a single platform
    ///
    /// # Arguments
    ///
    /// * `content` - Content to validate
    /// * `platform` - Platform to validate for
    /// * `auto_thread` - If true, skip character limit checks (content will be split into threads)
    fn validate_for_platform(
        &self,
        content: &str,
        platform: &str,
        auto_thread: bool,
    ) -> PlatformValidation {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Check for empty or whitespace-only content
        if content.trim().is_empty() {
            errors.push("Content cannot be empty or whitespace-only".to_string());
        }

        // Check content size (100KB limit)
        if content.len() > MAX_CONTENT_LENGTH {
            errors.push(format!(
                "Content size ({} bytes) exceeds maximum allowed size ({} bytes)",
                content.len(),
                MAX_CONTENT_LENGTH
            ));
        }

        // Platform-specific validation (skip character limits if auto-threading)
        match platform {
            "nostr" => {
                self.validate_nostr(content, &mut errors, &mut warnings, auto_thread);
            }
            "mastodon" => {
                self.validate_mastodon(content, &mut errors, &mut warnings, auto_thread);
            }
            "ssb" => {
                // SSB has no hard character limit, just warn if very large
                if content.len() > 8192 {
                    warnings.push(
                        "Content is very large (>8KB). SSB messages are typically smaller."
                            .to_string(),
                    );
                }
            }
            _ => {
                warnings.push(format!(
                    "Unknown platform '{}', skipping platform-specific validation",
                    platform
                ));
            }
        }

        PlatformValidation {
            platform: platform.to_string(),
            valid: errors.is_empty(),
            errors,
            warnings,
        }
    }

    /// Validate content for Nostr
    ///
    /// # Arguments
    ///
    /// * `auto_thread` - If true, skip character limit warnings (content will be threaded)
    fn validate_nostr(
        &self,
        content: &str,
        _errors: &mut Vec<String>,
        warnings: &mut Vec<String>,
        auto_thread: bool,
    ) {
        if auto_thread {
            // Skip limit checks when auto-threading - content will be split
            return;
        }

        let char_count = content.chars().count();

        // Nostr has no hard limit, but warn if exceeding typical limit
        if char_count > NOSTR_WARN_LIMIT {
            warnings.push(format!(
                "Content length ({} characters) exceeds recommended limit of {} characters for Nostr",
                char_count,
                NOSTR_WARN_LIMIT
            ));
        }
    }

    /// Validate content for Mastodon
    ///
    /// # Arguments
    ///
    /// * `auto_thread` - If true, skip character limit checks (content will be threaded)
    fn validate_mastodon(
        &self,
        content: &str,
        errors: &mut Vec<String>,
        _warnings: &mut Vec<String>,
        auto_thread: bool,
    ) {
        if auto_thread {
            // Skip limit checks when auto-threading - content will be split
            return;
        }

        let char_count = content.chars().count();
        let limit = self.get_mastodon_char_limit();

        if char_count > limit {
            errors.push(format!(
                "Content length ({} characters) exceeds Mastodon limit of {} characters",
                char_count, limit
            ));
        }
    }

    /// Get Mastodon character limit from config or use default
    fn get_mastodon_char_limit(&self) -> usize {
        // For now, use default. In the future, this could query the instance
        // for its actual limit via the API.
        MASTODON_DEFAULT_CHAR_LIMIT
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{DatabaseConfig, DefaultsConfig};

    fn create_test_config() -> Config {
        Config {
            database: DatabaseConfig {
                path: ":memory:".to_string(),
            },
            credentials: None,
            nostr: None,
            mastodon: None,
            ssb: None,
            defaults: DefaultsConfig::default(),
            scheduling: None,
        }
    }

    #[test]
    fn test_validate_valid_content_single_platform() {
        let config = Arc::new(create_test_config());
        let service = ValidationService::new(config);

        let request = ValidationRequest {
            content: "Hello world!".to_string(),
            platforms: vec!["nostr".to_string()],
            auto_thread: false,
        };

        let response = service.validate(request);
        assert!(response.valid);
        assert_eq!(response.results.len(), 1);
        assert!(response.results[0].valid);
        assert!(response.results[0].errors.is_empty());
    }

    #[test]
    fn test_validate_valid_content_multiple_platforms() {
        let config = Arc::new(create_test_config());
        let service = ValidationService::new(config);

        let request = ValidationRequest {
            content: "Hello decentralized world!".to_string(),
            platforms: vec![
                "nostr".to_string(),
                "mastodon".to_string(),
                "ssb".to_string(),
            ],
            auto_thread: false,
        };

        let response = service.validate(request);
        assert!(response.valid);
        assert_eq!(response.results.len(), 3);

        for result in &response.results {
            assert!(result.valid);
            assert!(result.errors.is_empty());
        }
    }

    #[test]
    fn test_validate_empty_content() {
        let config = Arc::new(create_test_config());
        let service = ValidationService::new(config);

        let request = ValidationRequest {
            content: "".to_string(),
            platforms: vec!["nostr".to_string()],
            auto_thread: false,
        };

        let response = service.validate(request);
        assert!(!response.valid);
        assert_eq!(response.results.len(), 1);
        assert!(!response.results[0].valid);
        assert!(!response.results[0].errors.is_empty());
        assert!(response.results[0].errors[0].contains("empty"));
    }

    #[test]
    fn test_validate_whitespace_only_content() {
        let config = Arc::new(create_test_config());
        let service = ValidationService::new(config);

        let request = ValidationRequest {
            content: "   \n\t  ".to_string(),
            platforms: vec!["nostr".to_string()],
            auto_thread: false,
        };

        let response = service.validate(request);
        assert!(!response.valid);
        assert!(!response.results[0].valid);
        assert!(response.results[0].errors[0].contains("whitespace"));
    }

    #[test]
    fn test_validate_max_content_length() {
        let config = Arc::new(create_test_config());
        let service = ValidationService::new(config);

        // Create content exceeding 100KB
        let large_content = "a".repeat(MAX_CONTENT_LENGTH + 1);

        let request = ValidationRequest {
            content: large_content,
            platforms: vec!["nostr".to_string()],
            auto_thread: false,
        };

        let response = service.validate(request);
        assert!(!response.valid);
        assert!(!response.results[0].valid);
        assert!(response.results[0]
            .errors
            .iter()
            .any(|e| e.contains("exceeds maximum")));
    }

    #[test]
    fn test_validate_nostr_no_hard_limit() {
        let config = Arc::new(create_test_config());
        let service = ValidationService::new(config);

        // Create content exceeding typical Twitter-like limit but valid
        let long_content = "a".repeat(500);

        let request = ValidationRequest {
            content: long_content,
            platforms: vec!["nostr".to_string()],
            auto_thread: false,
        };

        let response = service.validate(request);
        assert!(response.valid); // Should be valid (no hard limit)
        assert!(response.results[0].valid);
        assert!(response.results[0].errors.is_empty());
        // Should have warning about exceeding recommended limit
        assert!(!response.results[0].warnings.is_empty());
    }

    #[test]
    fn test_validate_mastodon_char_limit() {
        let config = Arc::new(create_test_config());
        let service = ValidationService::new(config);

        // Create content exceeding Mastodon's default 500 char limit
        let long_content = "a".repeat(501);

        let request = ValidationRequest {
            content: long_content,
            platforms: vec!["mastodon".to_string()],
            auto_thread: false,
        };

        let response = service.validate(request);
        assert!(!response.valid);
        assert!(!response.results[0].valid);
        assert!(response.results[0]
            .errors
            .iter()
            .any(|e| e.contains("Mastodon limit")));
    }

    #[test]
    fn test_validate_mastodon_char_limit_skipped_with_auto_thread() {
        let config = Arc::new(create_test_config());
        let service = ValidationService::new(config);

        // Create content exceeding Mastodon's 500 char limit
        let long_content = "a".repeat(501);

        let request = ValidationRequest {
            content: long_content,
            platforms: vec!["mastodon".to_string()],
            auto_thread: true, // With auto-thread enabled
        };

        let response = service.validate(request);
        // Should be valid because auto_thread skips char limit checks
        assert!(response.valid);
        assert!(response.results[0].valid);
        assert!(response.results[0].errors.is_empty());
    }

    #[test]
    fn test_is_valid_convenience_method() {
        let config = Arc::new(create_test_config());
        let service = ValidationService::new(config);

        assert!(service.is_valid("Hello world!", &vec!["nostr".to_string()]));
        assert!(!service.is_valid("", &vec!["nostr".to_string()]));

        let long_content = "a".repeat(501);
        assert!(!service.is_valid(&long_content, &vec!["mastodon".to_string()]));
    }

    #[test]
    fn test_get_limits() {
        let config = Arc::new(create_test_config());
        let service = ValidationService::new(config);

        let platforms = vec![
            "nostr".to_string(),
            "mastodon".to_string(),
            "ssb".to_string(),
        ];

        let limits = service.get_limits(&platforms);

        assert_eq!(limits.get("nostr"), Some(&None)); // No hard limit
        assert_eq!(
            limits.get("mastodon"),
            Some(&Some(MASTODON_DEFAULT_CHAR_LIMIT))
        );
        assert_eq!(limits.get("ssb"), Some(&None)); // SSB has no hard limit
    }

    #[test]
    fn test_get_limits_unknown_platform() {
        let config = Arc::new(create_test_config());
        let service = ValidationService::new(config);

        let platforms = vec!["unknown".to_string()];
        let limits = service.get_limits(&platforms);

        assert_eq!(limits.get("unknown"), Some(&None));
    }

    #[test]
    fn test_validate_unknown_platform() {
        let config = Arc::new(create_test_config());
        let service = ValidationService::new(config);

        let request = ValidationRequest {
            content: "Hello world!".to_string(),
            platforms: vec!["unknown_platform".to_string()],
            auto_thread: false,
        };

        let response = service.validate(request);
        assert!(response.valid); // Should be valid (only basic checks)
        assert!(!response.results[0].warnings.is_empty()); // Should have warning
        assert!(response.results[0].warnings[0].contains("Unknown platform"));
    }

    #[test]
    fn test_char_count_vs_byte_count() {
        let config = Arc::new(create_test_config());
        let service = ValidationService::new(config);

        // Unicode characters (emoji) count as 1 character but multiple bytes
        let content = "🚀".repeat(500); // 500 characters, but more bytes

        let request = ValidationRequest {
            content: content.clone(),
            platforms: vec!["mastodon".to_string()],
            auto_thread: false,
        };

        let response = service.validate(request);
        // Should be valid (exactly at limit)
        assert!(response.valid);

        // Add one more character to exceed limit
        let content_over = format!("{}🚀", content);
        let request_over = ValidationRequest {
            content: content_over,
            platforms: vec!["mastodon".to_string()],
            auto_thread: false,
        };

        let response_over = service.validate(request_over);
        assert!(!response_over.valid);
    }

    #[test]
    fn test_validation_service_clone() {
        let config = Arc::new(create_test_config());
        let service = ValidationService::new(config);
        let cloned = service.clone();

        // Both should work identically
        let request = ValidationRequest {
            content: "Test".to_string(),
            platforms: vec!["nostr".to_string()],
            auto_thread: false,
        };

        let response1 = service.validate(request.clone());
        let response2 = cloned.validate(request);

        assert_eq!(response1.valid, response2.valid);
    }
}

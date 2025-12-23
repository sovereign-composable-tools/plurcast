//! Platform ID Detection
//!
//! This module provides utilities for detecting which platform a post ID belongs to
//! based on its format. This is used to intelligently route reply-to IDs to the
//! correct platform when cross-posting.
//!
//! # Supported Formats
//!
//! - **Nostr**: `note1...` (bech32) or 64-character hex event ID
//! - **Mastodon**: Numeric string (Snowflake ID, e.g., "123456789012345678")
//! - **SSB**: `%...=.sha256` (cypherlink format)
//!
//! # Example
//!
//! ```
//! use libplurcast::platforms::id_detection::{detect_platform_from_id, DetectedPlatform};
//!
//! let nostr_id = "note1xvwqmxy5t2dhujkme857rfdhul424wkpthzqfwfkxcdlzgkyu2fsra5prs";
//! assert_eq!(detect_platform_from_id(nostr_id), DetectedPlatform::Nostr);
//!
//! let mastodon_id = "123456789012345678";
//! assert_eq!(detect_platform_from_id(mastodon_id), DetectedPlatform::Mastodon);
//! ```

/// Platform that a given ID belongs to
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectedPlatform {
    /// Nostr event ID (note1... bech32 or 64-char hex)
    Nostr,
    /// Mastodon status ID (numeric Snowflake ID)
    Mastodon,
    /// SSB message ID (%...=.sha256 cypherlink)
    Ssb,
    /// Unknown format - could not determine platform
    Unknown,
}

impl DetectedPlatform {
    /// Returns the platform name as a string, matching the names used in plurcast
    ///
    /// Returns `None` for `Unknown` variant.
    #[must_use]
    pub fn as_platform_name(&self) -> Option<&'static str> {
        match self {
            Self::Nostr => Some("nostr"),
            Self::Mastodon => Some("mastodon"),
            Self::Ssb => Some("ssb"),
            Self::Unknown => None,
        }
    }
}

/// Detect which platform a post ID belongs to based on its format
///
/// # Format Detection Rules
///
/// - **Nostr bech32**: Starts with `note1` and is at least 59 characters
/// - **Nostr hex**: Exactly 64 hexadecimal characters (event ID in hex format)
/// - **Mastodon**: Numeric-only string (Snowflake IDs are typically 18-19 digits)
/// - **SSB**: Starts with `%` and ends with `=.sha256` (cypherlink format)
///
/// # Arguments
///
/// * `id` - The post ID to detect
///
/// # Returns
///
/// The detected platform, or `DetectedPlatform::Unknown` if the format doesn't match
/// any known platform.
///
/// # Examples
///
/// ```
/// use libplurcast::platforms::id_detection::{detect_platform_from_id, DetectedPlatform};
///
/// // Nostr bech32 format
/// let nostr_bech32 = "note1xvwqmxy5t2dhujkme857rfdhul424wkpthzqfwfkxcdlzgkyu2fsra5prs";
/// assert_eq!(detect_platform_from_id(nostr_bech32), DetectedPlatform::Nostr);
///
/// // Nostr hex format
/// let nostr_hex = "4a5d5f14bfbcbd646dc231648e80ee21e65e0779509bece2aebcc54dcd85b2a1";
/// assert_eq!(detect_platform_from_id(nostr_hex), DetectedPlatform::Nostr);
///
/// // Mastodon numeric ID
/// let mastodon_id = "123456789012345678";
/// assert_eq!(detect_platform_from_id(mastodon_id), DetectedPlatform::Mastodon);
///
/// // SSB cypherlink
/// let ssb_id = "%abc123def456=.sha256";
/// assert_eq!(detect_platform_from_id(ssb_id), DetectedPlatform::Ssb);
///
/// // Unknown format
/// let unknown = "some-random-string";
/// assert_eq!(detect_platform_from_id(unknown), DetectedPlatform::Unknown);
/// ```
#[must_use]
pub fn detect_platform_from_id(id: &str) -> DetectedPlatform {
    // Nostr: bech32 note ID (note1... format)
    // Standard bech32 note IDs are 63 characters, but we allow some flexibility
    if id.starts_with("note1") && id.len() >= 59 {
        return DetectedPlatform::Nostr;
    }

    // Nostr: 64-character hex event ID
    if id.len() == 64 && id.chars().all(|c| c.is_ascii_hexdigit()) {
        return DetectedPlatform::Nostr;
    }

    // SSB: cypherlink format %...=.sha256
    // Check this before Mastodon since SSB IDs contain non-numeric chars
    if id.starts_with('%') && id.ends_with("=.sha256") {
        return DetectedPlatform::Ssb;
    }

    // Mastodon: numeric string (Snowflake IDs)
    // Mastodon uses Snowflake IDs which are typically 18-19 digits
    // We accept any non-empty numeric string for flexibility
    if !id.is_empty() && id.chars().all(|c| c.is_ascii_digit()) {
        return DetectedPlatform::Mastodon;
    }

    DetectedPlatform::Unknown
}

/// Check if a post ID matches a specific platform
///
/// # Arguments
///
/// * `id` - The post ID to check
/// * `platform` - The platform name to check against (e.g., "nostr", "mastodon", "ssb")
///
/// # Returns
///
/// `true` if the ID format matches the specified platform, `false` otherwise.
///
/// # Examples
///
/// ```
/// use libplurcast::platforms::id_detection::id_matches_platform;
///
/// let nostr_id = "note1xvwqmxy5t2dhujkme857rfdhul424wkpthzqfwfkxcdlzgkyu2fsra5prs";
/// assert!(id_matches_platform(nostr_id, "nostr"));
/// assert!(!id_matches_platform(nostr_id, "mastodon"));
///
/// let mastodon_id = "123456789012345678";
/// assert!(id_matches_platform(mastodon_id, "mastodon"));
/// assert!(!id_matches_platform(mastodon_id, "nostr"));
/// ```
#[must_use]
pub fn id_matches_platform(id: &str, platform: &str) -> bool {
    let detected = detect_platform_from_id(id);
    match platform {
        "nostr" => detected == DetectedPlatform::Nostr,
        "mastodon" => detected == DetectedPlatform::Mastodon,
        "ssb" => detected == DetectedPlatform::Ssb,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Nostr ID Detection Tests
    // =========================================================================

    #[test]
    fn test_detect_nostr_bech32_id() {
        // Standard note1... format (63 chars)
        let id = "note1xvwqmxy5t2dhujkme857rfdhul424wkpthzqfwfkxcdlzgkyu2fsra5prs";
        assert_eq!(detect_platform_from_id(id), DetectedPlatform::Nostr);
    }

    #[test]
    fn test_detect_nostr_bech32_id_minimum_length() {
        // Test minimum valid length (59 chars)
        let id = "note1" .to_string() + &"a".repeat(54);
        assert_eq!(detect_platform_from_id(&id), DetectedPlatform::Nostr);
    }

    #[test]
    fn test_detect_nostr_bech32_id_too_short() {
        // Too short to be a valid note ID
        let id = "note1abc";
        assert_eq!(detect_platform_from_id(id), DetectedPlatform::Unknown);
    }

    #[test]
    fn test_detect_nostr_hex_id() {
        // 64-character hex event ID
        let id = "4a5d5f14bfbcbd646dc231648e80ee21e65e0779509bece2aebcc54dcd85b2a1";
        assert_eq!(detect_platform_from_id(id), DetectedPlatform::Nostr);
    }

    #[test]
    fn test_detect_nostr_hex_id_uppercase() {
        // Hex IDs can be uppercase too
        let id = "4A5D5F14BFBCBD646DC231648E80EE21E65E0779509BECE2AEBCC54DCD85B2A1";
        assert_eq!(detect_platform_from_id(id), DetectedPlatform::Nostr);
    }

    #[test]
    fn test_detect_nostr_hex_id_mixed_case() {
        // Mixed case hex
        let id = "4a5D5f14BfBcBd646dC231648E80eE21e65e0779509bEcE2AeBcC54dCd85B2a1";
        assert_eq!(detect_platform_from_id(id), DetectedPlatform::Nostr);
    }

    #[test]
    fn test_detect_nostr_hex_id_wrong_length() {
        // 63 chars - one short
        let id = "4a5d5f14bfbcbd646dc231648e80ee21e65e0779509bece2aebcc54dcd85b2a";
        assert_eq!(detect_platform_from_id(id), DetectedPlatform::Unknown);
    }

    #[test]
    fn test_detect_nostr_hex_id_with_invalid_char() {
        // Contains 'g' which is not a hex digit
        let id = "4a5d5f14bfbcbd646dc231648e80ee21e65e0779509bece2aebcc54dcd85b2g1";
        assert_eq!(detect_platform_from_id(id), DetectedPlatform::Unknown);
    }

    // =========================================================================
    // Mastodon ID Detection Tests
    // =========================================================================

    #[test]
    fn test_detect_mastodon_id_typical() {
        // Typical Snowflake ID (18-19 digits)
        let id = "123456789012345678";
        assert_eq!(detect_platform_from_id(id), DetectedPlatform::Mastodon);
    }

    #[test]
    fn test_detect_mastodon_id_long() {
        // Longer numeric ID
        let id = "1234567890123456789012345";
        assert_eq!(detect_platform_from_id(id), DetectedPlatform::Mastodon);
    }

    #[test]
    fn test_detect_mastodon_id_short() {
        // Short numeric ID (older instances might have shorter IDs)
        let id = "12345";
        assert_eq!(detect_platform_from_id(id), DetectedPlatform::Mastodon);
    }

    #[test]
    fn test_detect_mastodon_id_single_digit() {
        // Edge case: single digit
        let id = "1";
        assert_eq!(detect_platform_from_id(id), DetectedPlatform::Mastodon);
    }

    #[test]
    fn test_detect_mastodon_id_with_letter() {
        // Not a valid Mastodon ID - contains a letter
        let id = "12345a6789";
        assert_eq!(detect_platform_from_id(id), DetectedPlatform::Unknown);
    }

    // =========================================================================
    // SSB ID Detection Tests
    // =========================================================================

    #[test]
    fn test_detect_ssb_id_typical() {
        // Typical SSB cypherlink
        let id = "%abc123def456ghi789=.sha256";
        assert_eq!(detect_platform_from_id(id), DetectedPlatform::Ssb);
    }

    #[test]
    fn test_detect_ssb_id_base64() {
        // Real SSB message ID (base64 with special chars)
        let id = "%HZVnEzm0NgoSVfG0Hx4gMFbMMHhFvhJsG2zK/pijYII=.sha256";
        assert_eq!(detect_platform_from_id(id), DetectedPlatform::Ssb);
    }

    #[test]
    fn test_detect_ssb_id_missing_percent() {
        // Missing % prefix
        let id = "abc123=.sha256";
        assert_eq!(detect_platform_from_id(id), DetectedPlatform::Unknown);
    }

    #[test]
    fn test_detect_ssb_id_wrong_suffix() {
        // Wrong suffix
        let id = "%abc123=.sha512";
        assert_eq!(detect_platform_from_id(id), DetectedPlatform::Unknown);
    }

    // =========================================================================
    // Unknown ID Detection Tests
    // =========================================================================

    #[test]
    fn test_detect_unknown_empty() {
        let id = "";
        assert_eq!(detect_platform_from_id(id), DetectedPlatform::Unknown);
    }

    #[test]
    fn test_detect_unknown_random_string() {
        let id = "some-random-string-123";
        assert_eq!(detect_platform_from_id(id), DetectedPlatform::Unknown);
    }

    #[test]
    fn test_detect_unknown_uuid() {
        // UUIDs should be Unknown (handled separately in main.rs)
        let id = "550e8400-e29b-41d4-a716-446655440000";
        assert_eq!(detect_platform_from_id(id), DetectedPlatform::Unknown);
    }

    #[test]
    fn test_detect_unknown_url() {
        let id = "https://example.com/status/123";
        assert_eq!(detect_platform_from_id(id), DetectedPlatform::Unknown);
    }

    // =========================================================================
    // Platform Matching Tests
    // =========================================================================

    #[test]
    fn test_id_matches_platform_nostr() {
        let nostr_id = "note1xvwqmxy5t2dhujkme857rfdhul424wkpthzqfwfkxcdlzgkyu2fsra5prs";
        assert!(id_matches_platform(nostr_id, "nostr"));
        assert!(!id_matches_platform(nostr_id, "mastodon"));
        assert!(!id_matches_platform(nostr_id, "ssb"));
    }

    #[test]
    fn test_id_matches_platform_mastodon() {
        let mastodon_id = "123456789012345678";
        assert!(id_matches_platform(mastodon_id, "mastodon"));
        assert!(!id_matches_platform(mastodon_id, "nostr"));
        assert!(!id_matches_platform(mastodon_id, "ssb"));
    }

    #[test]
    fn test_id_matches_platform_ssb() {
        let ssb_id = "%abc123=.sha256";
        assert!(id_matches_platform(ssb_id, "ssb"));
        assert!(!id_matches_platform(ssb_id, "nostr"));
        assert!(!id_matches_platform(ssb_id, "mastodon"));
    }

    #[test]
    fn test_id_matches_platform_unknown() {
        let unknown_id = "some-random-string";
        assert!(!id_matches_platform(unknown_id, "nostr"));
        assert!(!id_matches_platform(unknown_id, "mastodon"));
        assert!(!id_matches_platform(unknown_id, "ssb"));
        assert!(!id_matches_platform(unknown_id, "unknown"));
    }

    #[test]
    fn test_id_matches_platform_invalid_platform() {
        let nostr_id = "note1xvwqmxy5t2dhujkme857rfdhul424wkpthzqfwfkxcdlzgkyu2fsra5prs";
        assert!(!id_matches_platform(nostr_id, "bluesky"));
        assert!(!id_matches_platform(nostr_id, ""));
        assert!(!id_matches_platform(nostr_id, "NOSTR")); // Case sensitive
    }

    // =========================================================================
    // as_platform_name Tests
    // =========================================================================

    #[test]
    fn test_as_platform_name() {
        assert_eq!(DetectedPlatform::Nostr.as_platform_name(), Some("nostr"));
        assert_eq!(DetectedPlatform::Mastodon.as_platform_name(), Some("mastodon"));
        assert_eq!(DetectedPlatform::Ssb.as_platform_name(), Some("ssb"));
        assert_eq!(DetectedPlatform::Unknown.as_platform_name(), None);
    }
}

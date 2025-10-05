# Input Validation Security - Design

**Spec ID**: input-validation-security  
**Created**: 2025-10-05  
**Status**: Draft  
**Related Requirements**: requirements.md

---

## Design Overview

This design implements content length validation to prevent memory exhaustion and DoS attacks in the `plur-post` binary. The solution adds a hard limit on input size with early termination for oversized content, minimal performance overhead, and clear error messaging.

**Security Context**: Addresses Issue H2 from security audit - Missing Input Validation on Content Length. This is a CRITICAL vulnerability that allows unbounded input to exhaust system memory.

**Key Design Principles**:
1. **Fail Fast**: Detect oversized content as early as possible
2. **Memory Safe**: Never allocate more than MAX_CONTENT_LENGTH bytes
3. **User Friendly**: Provide clear, actionable error messages
4. **Zero Dependencies**: Use only Rust standard library
5. **Backward Compatible**: Don't break existing functionality

---

## Architecture

### Component Diagram

```
┌─────────────────────────────────────────────────────────┐
│                    plur-post Binary                      │
├─────────────────────────────────────────────────────────┤
│                                                           │
│  ┌─────────────────────────────────────────────────┐   │
│  │           get_content(cli: &Cli)                 │   │
│  │                                                   │   │
│  │  ┌──────────────────────────────────────────┐  │   │
│  │  │  Argument Input Path                      │  │   │
│  │  │  - Check cli.content.len()                │  │   │
│  │  │  - Validate against MAX_CONTENT_LENGTH    │  │   │
│  │  │  - Return error if too large              │  │   │
│  │  └──────────────────────────────────────────┘  │   │
│  │                                                   │   │
│  │  ┌──────────────────────────────────────────┐  │   │
│  │  │  Stdin Input Path                         │  │   │
│  │  │  - Use stdin.lock().take(limit + 1)       │  │   │
│  │  │  - Read to string with size limit         │  │   │
│  │  │  - Check if limit was exceeded            │  │   │
│  │  │  - Return error if at/over limit          │  │   │
│  │  └──────────────────────────────────────────┘  │   │
│  │                                                   │   │
│  │  ┌──────────────────────────────────────────┐  │   │
│  │  │  Validation                               │  │   │
│  │  │  - Empty content check                    │  │   │
│  │  │  - Size limit check                       │  │   │
│  │  │  - Return PlurcastError::InvalidInput     │  │   │
│  │  └──────────────────────────────────────────┘  │   │
│  └─────────────────────────────────────────────────┘   │
│                                                           │
│  ┌─────────────────────────────────────────────────┐   │
│  │         Error Handling (main.rs)                 │   │
│  │  - Catch PlurcastError::InvalidInput             │   │
│  │  - Print to stderr                               │   │
│  │  - Exit with code 3                              │   │
│  └─────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

### Data Flow

```
User Input (stdin or arg)
    ↓
┌───────────────────────┐
│  Input Source Check   │
│  - Argument?          │
│  - Stdin?             │
│  - TTY?               │
└───────────────────────┘
    ↓
┌───────────────────────┐
│  Size Validation      │
│  - Check length       │
│  - Compare to limit   │
│  - Fail fast if over  │
└───────────────────────┘
    ↓
┌───────────────────────┐
│  Content Validation   │
│  - Empty check        │
│  - Trim whitespace    │
└───────────────────────┘
    ↓
┌───────────────────────┐
│  Return Result        │
│  - Ok(content)        │
│  - Err(InvalidInput)  │
└───────────────────────┘
```

---

## Components and Interfaces

### Constants

**MAX_CONTENT_LENGTH**
- Type: `const usize`
- Value: `100_000` (100KB)
- Location: `plur-post/src/main.rs` (module level, after imports)
- Visibility: Private to module
- Purpose: Single source of truth for content size limit

### Modified Functions

**get_content(cli: &Cli) -> Result<String>**
- Location: `plur-post/src/main.rs`
- Modifications: Add size validation for both stdin and argument paths
- Returns: `Result<String>` with `PlurcastError::InvalidInput` on validation failure
- Side effects: None (pure validation logic)

### Error Types

**PlurcastError::InvalidInput**
- Existing error variant (no changes needed)
- Used for all validation failures including size limits
- Maps to exit code 3 in main error handler

---

## Implementation Details

### 1. Constants Definition

**Location**: `plur-post/src/main.rs` (top of file, after imports)

```rust
/// Maximum content length in bytes (100KB)
/// 
/// This limit prevents memory exhaustion and DoS attacks while allowing
/// for long-form posts. Most social platforms have much lower limits:
/// - Nostr: ~32KB practical limit
/// - Mastodon: 500 characters default (configurable)
/// - Bluesky: 300 characters
/// 
/// 100KB provides headroom for future features while protecting against abuse.
const MAX_CONTENT_LENGTH: usize = 100_000;
```

**Design Rationale**:
- 100KB is sufficient for very long posts (≈50,000 words)
- Well above any platform's actual limits
- Small enough to prevent memory issues
- Easy to remember and document
- Aligns with requirements FR-1

### 2. Modified get_content() Function

**Location**: `plur-post/src/main.rs`

**Design Approach**: Add validation at the earliest possible point in each input path to fail fast and prevent unnecessary memory allocation.

```rust
/// Get content from CLI argument or stdin with size validation
/// 
/// This function implements security controls to prevent memory exhaustion:
/// - Limits stdin reads to MAX_CONTENT_LENGTH bytes
/// - Validates argument length before processing
/// - Fails fast on oversized content
/// 
/// # Security
/// 
/// Addresses Issue H2: Missing Input Validation on Content Length
/// Prevents DoS attacks via unbounded input streams
/// 
/// # Errors
/// 
/// Returns `PlurcastError::InvalidInput` if:
/// - Content is empty or whitespace-only
/// - Content exceeds MAX_CONTENT_LENGTH bytes
/// - Stdin is a TTY and no argument provided
/// - Failed to read from stdin
fn get_content(cli: &Cli) -> Result<String> {
    // Path 1: Content from command-line argument
    if let Some(content) = &cli.content {
        // Validate length BEFORE any processing
        if content.len() > MAX_CONTENT_LENGTH {
            return Err(PlurcastError::InvalidInput(format!(
                "Content too large: {} bytes (maximum: {} bytes)",
                content.len(),
                MAX_CONTENT_LENGTH
            
       )));
        }
        
        let trimmed = content.trim();
        if trimmed.is_empty() {
            return Err(PlurcastError::InvalidInput(
                "Content cannot be empty".to_string()
            ));
        }
        
        return Ok(trimmed.to_string());
    }

    // Path 2: Content from stdin
    let stdin = io::stdin();
    
    // Check if stdin is a TTY (interactive terminal)
    if stdin.is_terminal() {
        return Err(PlurcastError::InvalidInput(
            "No content provided. Use: plur-post \"content\" or echo \"content\" | plur-post".to_string()
        ));
    }

    // Use take() to limit bytes read - prevents reading infinite streams
    // Read MAX_CONTENT_LENGTH + 1 to detect if limit was exceeded
    let mut buffer = String::new();
    stdin.lock()
        .take((MAX_CONTENT_LENGTH + 1) as u64)
        .read_to_string(&mut buffer)
        .map_err(|e| PlurcastError::InvalidInput(format!("Failed to read from stdin: {}", e)))?;

    // Check if we hit the limit
    if buffer.len() > MAX_CONTENT_LENGTH {
        return Err(PlurcastError::InvalidInput(format!(
            "Content too large: exceeds {} bytes (maximum: {} bytes)",
            MAX_CONTENT_LENGTH,
            MAX_CONTENT_LENGTH
        )));
    }

    let trimmed = buffer.trim();
    if trimmed.is_empty() {
        return Err(PlurcastError::InvalidInput(
            "Content cannot be empty".to_string()
        ));
    }

    Ok(trimmed.to_string())
}
```

**Key Design Decisions**:

1. **take(MAX_CONTENT_LENGTH + 1)**: Read one extra byte to detect if limit was exceeded
   - Rationale: Allows us to distinguish between "exactly at limit" and "over limit"
   - Alternative considered: Read exactly MAX_CONTENT_LENGTH and check stream state (more complex)

2. **Validate before trim**: Check size on raw input, then trim
   - Rationale: Prevents attacker from bypassing limit with whitespace padding
   - Aligns with requirement FR-2 and FR-3

3. **Consistent error format**: Both paths use same error message structure
   - Rationale: User-friendly, provides actionable information
   - Aligns with requirement FR-4

4. **Early return pattern**: Validate and return immediately on error
   - Rationale: Fail fast, no unnecessary processing
   - Aligns with NFR-1 (performance)

### 3. Error Message Format

**Design**: Error messages follow a consistent, informative pattern

```rust
// For oversized content
format!(
    "Content too large: {} bytes (maximum: {} bytes)",
    actual_size,
    MAX_CONTENT_LENGTH
)
```

**Rationale**:
- Clear indication of problem ("Content too large")
- Shows actual size for user awareness
- Shows maximum allowed size
- No content samples (security requirement SR-3)
- Human-readable format
- Aligns with requirement FR-4

### 4. Exit Code Handling

**Design**: No changes needed to existing error handling

The existing main() function already maps `PlurcastError::InvalidInput` to exit code 3:

```rust
// Existing code in main.rs
Err(e) => {
    match e {
        PlurcastError::InvalidInput(_) => std::process::exit(3),
        // ... other error types
    }
}
```

**Rationale**:
- Maintains backward compatibility
- Consistent with existing error handling
- Aligns with requirement FR-5

---

## Data Models

No new data models required. This feature uses existing types:

- `String` for content storage
- `PlurcastError::InvalidInput` for error reporting
- `usize` for size tracking

---

## Error Handling

### Error Scenarios

| Scenario | Detection Point | Error Message | Exit Code |
|----------|----------------|---------------|-----------|
| Argument > 100KB | Argument validation | "Content too large: X bytes (maximum: 100000 bytes)" | 3 |
| Stdin > 100KB | Stdin read | "Content too large: exceeds 100000 bytes (maximum: 100000 bytes)" | 3 |
| Empty content | After trim | "Content cannot be empty" | 3 |
| TTY stdin | Source check | "No content provided. Use: plur-post \"content\" or echo \"content\" \| plur-post" | 3 |
| Stdin read failure | I/O operation | "Failed to read from stdin: {error}" | 3 |

### Error Handling Strategy

1. **Validation First**: Check size before any processing
2. **Clear Messages**: Include actual and maximum sizes
3. **Consistent Exit Codes**: All validation errors use exit code 3
4. **Stderr Output**: All errors go to stderr (existing behavior)
5. **No Sensitive Data**: Error messages never include content samples

**Rationale**: Aligns with requirements FR-4, FR-5, and SR-3

---

## Testing Strategy

### Unit Tests

**Location**: `plur-post/tests/cli_integration.rs` (or new test file)

**Test Cases**:

1. **Valid Content Tests**
   - Content under limit (should pass)
   - Content exactly at limit (should pass)
   - Empty content after trim (should fail)

2. **Oversized Content Tests**
   - Argument exactly at limit + 1 byte (should fail)
   - Argument significantly over limit (should fail)
   - Stdin exactly at limit + 1 byte (should fail)
   - Stdin significantly over limit (should fail)

3. **Attack Scenario Tests**
   - Simulated infinite stream (should fail fast)
   - Very large argument (should fail immediately)
   - Whitespace padding attack (should fail)

4. **Error Message Tests**
   - Verify error format includes sizes
   - Verify no content samples in errors
   - Verify exit code 3 for all validation failures

5. **Performance Tests**
   - Validation overhead < 1ms for normal content
   - Immediate termination on oversized stdin

### Integration Tests

**Test Approach**: Use actual CLI invocations

```bash
# Test oversized argument
plur-post "$(python -c 'print("x"*100001)')"
# Expected: Exit code 3, error message with sizes

# Test oversized stdin
head -c 100001 /dev/zero | plur-post
# Expected: Exit code 3, immediate termination

# Test valid content
echo "Valid post" | plur-post
# Expected: Exit code 0 (or appropriate success code)
```

**Coverage Target**: >= 95% for validation logic (requirement NFR-4)

---

## Performance Considerations

### Memory Usage

**Design Goal**: Never allocate more than MAX_CONTENT_LENGTH bytes

**Implementation**:
- Argument path: No additional allocation (uses existing String)
- Stdin path: `take()` limits read to MAX_CONTENT_LENGTH + 1 bytes
- Early termination prevents reading entire oversized stream

**Rationale**: Aligns with requirement NFR-2 (memory safety)

### Validation Overhead

**Design Goal**: < 1ms for normal-sized content

**Implementation**:
- Single length check: O(1) operation
- No regex or complex parsing
- No additional allocations
- Trim operation: O(n) but only on valid content

**Expected Performance**:
- Argument validation: < 0.1ms (single comparison)
- Stdin validation: < 1ms for content under 10KB
- Oversized content: Immediate rejection (< 100ms)

**Rationale**: Aligns with requirement NFR-1 (performance)

### Backward Compatibility

**Design Goal**: No breaking changes for valid use cases

**Verification**:
- All existing tests continue to pass
- No changes to CLI interface
- No changes to output format for valid content
- Only new behavior: reject previously-unbounded large inputs

**Rationale**: Aligns with requirement NFR-3 (backward compatibility)

---

## Security Analysis

### Threat Mitigation

| Threat | Mitigation | Requirement |
|--------|-----------|-------------|
| Memory exhaustion via infinite stdin | `take()` limits read to 100KB | SR-1 |
| Memory exhaustion via huge argument | Length check before processing | SR-1 |
| Database bloat | Validation before DB insertion | SR-2 |
| Information leakage | No content in error messages | SR-3 |
| DoS via repeated large inputs | Fast rejection (< 100ms) | SR-1 |

### Attack Vector Coverage

**From Requirements Document**:

1. ✅ `cat /dev/zero | plur-post` - Blocked by `take()` limit
2. ✅ `plur-post "$(python -c 'print("x"*10000000)')"` - Blocked by argument validation
3. ✅ `cat huge_file.txt | plur-post` - Blocked by `take()` limit

### Security Properties

- **Fail-Safe**: Rejects invalid input, never processes it
- **No Bypass**: Both input paths validated
- **Fast Rejection**: < 100ms for oversized content
- **Memory Bounded**: Never exceeds MAX_CONTENT_LENGTH allocation
- **Audit Trail**: Validation failures logged at INFO level (future enhancement)

**Rationale**: Addresses all security requirements (SR-1, SR-2, SR-3)

---

## Dependencies

### Code Dependencies

- `std::io::Read` - For `take()` method
- `std::io::IsTerminal` - For TTY detection
- `PlurcastError::InvalidInput` - Existing error type

### No External Dependencies

This implementation uses only Rust standard library, maintaining the project's zero-dependency principle for core functionality.

**Rationale**: Aligns with design principle "Zero Dependencies"

---

## Implementation Checklist

- [ ] Add MAX_CONTENT_LENGTH constant to `plur-post/src/main.rs`
- [ ] Modify get_content() to validate argument length
- [ ] Modify get_content() to use take() for stdin
- [ ] Add size check after stdin read
- [ ] Update error messages to include sizes
- [ ] Verify exit code 3 for validation failures
- [ ] Add unit tests for validation logic
- [ ] Add integration tests for attack scenarios
- [ ] Update README.md with size limits
- [ ] Update CLAUDE.md with validation notes
- [ ] Mark Issue H2 as RESOLVED in security tracker

---

## Future Enhancements

### Out of Scope for This Spec

The following are explicitly NOT included in this implementation:

1. **Configurable Limits**: Making MAX_CONTENT_LENGTH user-configurable
   - Rationale: Not needed for alpha, adds complexity
   - Future: Could add to config.toml

2. **Platform-Specific Validation**: Character limits for Nostr/Mastodon/Bluesky
   - Rationale: Separate feature, different requirements
   - Future: Add platform-aware validation layer

3. **Logging**: Security audit trail for validation failures
   - Rationale: No logging infrastructure yet
   - Future: Add when logging system is implemented

4. **Rate Limiting**: Limiting number of posts per time period
   - Rationale: Separate security issue (C2)
   - Future: Separate implementation

### Potential Improvements

1. **Human-Readable Sizes**: Display "100 KB" instead of "100000 bytes"
   - Low priority: Current format is clear enough
   - Easy addition if users request it

2. **Suggestion Messages**: Provide guidance on how to reduce content
   - Low priority: Error message is already actionable
   - Could add: "Consider splitting into multiple posts"

3. **Progress Indicator**: Show progress for large stdin reads
   - Low priority: Validation is fast enough
   - Only useful for content near the limit

---

## References

- **Requirements**: `.kiro/specs/input-validation-security/requirements.md`
- **Security Audit**: `.kiro/reports/security-review-2025-10-05.md`
- **Issue Tracker**: `.kiro/reports/security-issues-tracker.md` (Issue H2)
- **Action Plan**: `.kiro/reports/security-action-plan.md` (Phase 1.2)
- **OWASP Input Validation**: https://cheatsheetseries.owasp.org/cheatsheets/Input_Validation_Cheat_Sheet.html
- **CWE-20**: https://cwe.mitre.org/data/definitions/20.html (Improper Input Validation)
- **CWE-400**: https://cwe.mitre.org/data/definitions/400.html (Uncontrolled Resource Consumption)

---

## Approval

**Design Status**: Ready for Review

This design addresses all requirements from the requirements document:
- ✅ All functional requirements (FR-1 through FR-5)
- ✅ All non-functional requirements (NFR-1 through NFR-4)
- ✅ All security requirements (SR-1 through SR-3)
- ✅ All constraints (technical, business, regulatory)
- ✅ All success criteria

**Next Step**: Create implementation plan (tasks.md)

# Security Issues Tracker

**Last Updated**: 2025-10-05  
**Project**: Plurcast v0.1.0-alpha

This document tracks the status of security issues identified in security audits. Issues are marked as OPEN, IN_PROGRESS, or RESOLVED.

**Latest Review**: 2025-10-05 - No new vulnerabilities found, previous issues remain valid

---

## Critical Severity

| ID | Issue | Status | Assigned | Target | Notes |
|----|-------|--------|----------|--------|-------|
| C1 | Nostr Private Keys Stored in Plaintext | OPEN | - | Beta | Documentation added, keyring integration planned |
| C2 | No Rate Limiting on Database Operations | OPEN | - | Beta | Requires SqlitePoolOptions configuration |

---

## High Severity

| ID | Issue | Status | Assigned | Target | Notes |
|----|-------|--------|----------|--------|-------|
| H1 | SQL Injection via Platform Names (Theoretical) | OPEN | - | 1.0 | Mitigated by validation, needs DB constraints |
| H2 | Missing Input Validation on Content Length | OPEN | - | Alpha | Quick fix, should be done immediately |
| H3 | Insufficient Error Message Sanitization | OPEN | - | Beta | Requires error.rs refactoring |
| H4 | No Integrity Verification for Configuration Files | OPEN | - | 1.0 | Consider for stable release |

---

## Medium Severity

| ID | Issue | Status | Assigned | Target | Notes |
|----|-------|--------|----------|--------|-------|
| M1 | Weak File Permission Handling on Windows | OPEN | - | 1.0 | Requires windows-acl crate |
| M2 | No Timeout on Network Operations | OPEN | - | Beta | Add tokio::time::timeout |
| M3 | Potential Path Traversal in Configuration Paths | OPEN | - | 1.0 | Add path validation function |

---

## Low Severity

| ID | Issue | Status | Assigned | Target | Notes |
|----|-------|--------|----------|--------|-------|
| L1 | Missing Dependency Vulnerability Scanning | OPEN | - | Alpha | Install cargo-audit |
| L2 | No Logging of Security Events | OPEN | - | Beta | Add structured security logging |

---

## Resolution Template

When marking an issue as RESOLVED, add details below:

### [Issue ID] - [Issue Title]

**Resolved Date**: YYYY-MM-DD  
**Resolved By**: [Name/Team]  
**Resolution**: [Description of fix]  
**Verification**: [How it was tested]  
**Commit**: [Git commit hash]

---

## Resolved Issues

(None yet - this is the initial audit)

---

## Notes

- Issues are prioritized by severity and target release
- Alpha release should address documentation and quick wins
- Beta release should address most HIGH and MEDIUM issues
- 1.0 release should address all CRITICAL and HIGH issues
- Some LOW issues may be deferred post-1.0


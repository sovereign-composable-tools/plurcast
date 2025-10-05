# Test Coverage Tracker - Plurcast

**Last Updated**: 2025-10-04  
**Overall Coverage**: ~15%  
**Target for Alpha**: 60%

## Module Coverage Status

### libplurcast Library

| Module | Current | Target | Status | Priority |
|--------|---------|--------|--------|----------|
| `config.rs` | 0% | 80% | ❌ Not Started | P0 |
| `db.rs` | 70% | 85% | 🟡 In Progress | P0 |
| `error.rs` | 0% | 90% | ❌ Not Started | P0 |
| `types.rs` | 0% | 85% | ❌ Not Started | P1 |
| `platforms/mod.rs` | 0% | 60% | ❌ Not Started | P2 |
| `platforms/nostr.rs` | 0% | 75% | ❌ Not Started | P0 |

### plur-post Binary

| Module | Current | Target | Status | Priority |
|--------|---------|--------|--------|----------|
| `main.rs` (unit) | 0% | 70% | ❌ Not Started | P1 |
| Integration tests | 0% | 80% | ❌ Not Started | P1 |

## Test Categories

### Unit Tests
- [x] Database error handling (db.rs)
- [ ] Configuration parsing (config.rs)
- [ ] Nostr key loading (platforms/nostr.rs)
- [ ] Error exit codes (error.rs)
- [ ] Post creation (types.rs)

### Integration Tests
- [ ] CLI argument parsing
- [ ] Stdin input handling
- [ ] Exit code verification
- [ ] Output formats (text/json)
- [ ] Draft mode workflow
- [ ] Platform posting workflow

### End-to-End Tests
- [ ] Full posting workflow
- [ ] Multi-platform posting
- [ ] Error recovery
- [ ] Database persistence

## Progress by Phase

### Phase 1: Critical Path (P0) - Target: 6 hours
- [ ] config.rs tests (2h)
- [ ] platforms/nostr.rs tests (2h)
- [ ] Expand db.rs tests (1h)
- [ ] error.rs tests (1h)

**Progress**: 0/4 complete (0%)

### Phase 2: Integration (P1) - Target: 6 hours
- [ ] CLI integration tests (3h)
- [ ] types.rs tests (1h)
- [ ] E2E workflow test (2h)

**Progress**: 0/3 complete (0%)

### Phase 3: Advanced (P2/P3) - Target: 15 hours
- [ ] Property-based tests (4h)
- [ ] Doc tests (2h)
- [ ] CI setup (2h)
- [ ] Benchmarks (3h)
- [ ] Fuzzing (4h)

**Progress**: 0/5 complete (0%)

## Test Quality Metrics

### Code Coverage
- **Lines**: ~15%
- **Branches**: ~10%
- **Functions**: ~20%

### Test Characteristics
- ✅ Async tests use `tokio::test`
- ✅ Database tests use in-memory SQLite
- ✅ Proper error type checking
- ❌ No integration tests yet
- ❌ No property-based tests yet
- ❌ No doc tests yet

## Blockers & Issues

### Critical Blockers (Alpha Release)
1. **No configuration tests** - Can't verify config parsing works
2. **No Nostr key tests** - Can't verify authentication works
3. **No CLI tests** - Can't verify exit codes and output

### Known Issues
- Database tests don't verify all CRUD operations
- No tests for concurrent operations
- No tests for rate limiting (future)
- No tests for retry logic (future)

## Next Actions

### This Week
1. Implement config.rs tests
2. Implement platforms/nostr.rs tests
3. Expand db.rs test coverage

### Next Week
1. Add CLI integration tests
2. Add types.rs tests
3. Create first E2E test

### Future
1. Set up CI with coverage reporting
2. Add property-based tests
3. Add benchmarks for critical paths

## Coverage Commands

```bash
# Run all tests
cargo test --workspace

# Run with coverage (requires tarpaulin)
cargo tarpaulin --out Html --output-dir coverage

# Run specific module
cargo test --package libplurcast --lib config

# Run integration tests only
cargo test --test '*'
```

## Notes

- Database module has good test patterns to follow
- Use `tempfile` for file-based tests
- Use in-memory SQLite for database tests
- Use `assert_cmd` for CLI integration tests
- Mock external services (Nostr relays) when possible

---

**Maintainer**: Update this file as tests are added  
**Review Frequency**: Weekly during active development

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.3]

### Added

#### Test Suite
- **Comprehensive Unit Tests** (72 tests total, 100% pass rate)
  - `errors.rs`: Tests for all error variants, message generation, and serialization
  - `tools/structure.rs`: Tests for `Limit`, `Pointer`, and `Timestamp` structs
  - `engine/structure.rs`: Tests for `Engine` constructor, URI parsing, TLS handling, and `ValidPermissions` enum
  - `response/structure.rs`: Tests for `MontycatResponse` and `MontycatStreamResponse` parsing, including nested JSON

#### CI/CD Pipeline
- **GitHub Actions Workflows**
  - `ci.yml`: Multi-platform testing (Ubuntu, macOS, Windows) across Rust stable and beta
  - `publish.yml`: Automated publishing to crates.io on version tags
  - `security.yml`: Weekly security audits with `cargo-audit` and dependency checks
- **CI/CD Documentation**
  - `.github/CICD_SETUP.md`: Comprehensive setup guide with configuration instructions
  - CI badge integration in README.md

#### Documentation
- **Artifacts**
  - `walkthrough.md`: Complete documentation of test suite implementation and verification
  - `cicd_walkthrough.md`: Detailed CI/CD pipeline documentation
  - `task.md`: Project task tracking and progress

### Changed

#### Code Quality Improvements
- **Clippy Warning Fixes**
  - Added `StreamCallback` type alias to reduce type complexity in `engine/utils.rs`
  - Boxed large `Req::Store` enum variant in `request/structure.rs`
  - Changed `parse_response` parameter from `&Vec<u8>` to `&[u8]` in `response/structure.rs`
  - Removed unused `Arc` import in `keyspace/structures/persistent.rs`
  - Fixed empty line after doc comment in `persistent.rs`
  - Updated `subscribe` function documentation to reflect new return type

#### Dependencies
- Added `tokio-test` as dev-dependency for async testing support

### Fixed
- All clippy warnings resolved (5 total)
- Type mismatch in `test_montycat_stream_response_parse_invalid_json`
- URL encoding issue in `test_engine_from_uri_with_special_characters`
- Various clippy warnings including empty lines after doc comments, type complexity, large enum variants, and `&Vec` vs `&[_]` usage

### Testing
- **Code Coverage**: Comprehensive unit test coverage across core modules
- **CI Integration**: Automated testing on push and pull requests
- **Multi-Platform**: Tests run on Ubuntu, macOS, and Windows
- **Rust Versions**: Tests run on stable and beta Rust channels

### Security
- Weekly automated security audits via GitHub Actions
- Dependency vulnerability scanning with `cargo-audit`
- Outdated dependency checks with `cargo-outdated`

## [0.1.2] - Previous Release

### Initial Features
- Core Montycat client implementation
- Engine for connection management
- Keyspace abstractions (persistent and in-memory)
- Request/response handling
- Error handling system
- Tool structures for data manipulation

---

## Notes

### CI/CD Setup Requirements
To enable the full CI/CD pipeline:
1. Add `CARGO_REGISTRY_TOKEN` to GitHub repository secrets
2. Push changes to trigger CI workflows
3. Create version tags (e.g., `v0.1.3`) to trigger publish workflow

### Test Execution
```bash
# Run all tests
cargo test --all-features

# Run with coverage
cargo test --all-features --no-fail-fast

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings

# Run formatting check
cargo fmt --all -- --check
```

### Future Improvements
- Integration tests with running Montycat server
- Doc tests for public API examples
- Performance benchmarks
- Additional platform support

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.2] - 2026-07-29

Fixes a hang that affects any request whose payload contains the word
`subscribe`. **Upgrade from 0.3.1 is recommended.**

### Fixed

- **A request whose value contained the substring `subscribe` never returned.**
  Subscription mode was detected by scanning the serialized request for
  `b"subscribe"`, so a call like
  `insert_value(None, json!({"note": "please subscribe"}))` was routed into the
  streaming branch — which has no read timeout and loops forever. Any record
  mentioning the word was affected, `unsubscribe` included.

  A request is now a subscription because the caller supplied a callback. That
  was always the real distinction: exactly one call site in the crate passes
  one. Intent is no longer inferred from user data.

  Present in every release before this one. The Python and Dart clients carry
  the same defect and are fixed in their matching releases; the Node client was
  already correct.

## [0.3.1] - 2026-07-29

Documentation and tests only — no library code changed, so upgrading from 0.3.0
is optional.

### Added

- README sections for behavior that was previously undocumented: response shape
  and `MontycatResponse::parse_response`, real-time subscriptions returning a
  `tokio::sync::watch::Sender<bool>` on the `port + 1` subscription port, TLS via
  the `tls` feature and `enable_tls`, and owner/access management with
  `create_owner`, `grant_to`, `revoke_from`, and `ValidPermissions`.
- Tests covering u128 key preservation through `parse_response`, both directly
  and inside double-encoded JSON payloads, and payload/type mismatches returning
  `ClientValueParsingError` rather than panicking.
- Changelog link in the README.

### Fixed

- The README pinned `montycat = "0.2"` in both dependency snippets, a version
  that predates the governance APIs documented further down the same file.
- The README semantic-search example used `SemanticModel::BgeBase` without
  importing `SemanticModel`.
- Removed a leftover installation heading that duplicated the "Get the Engine"
  section.

## [0.3.0] - 2026-07-28

### Changed
- Governance qualifiers are validated client-side before sending a command:
  - semantic models apply to `ProvisionKeyspace` and `ManageSemantic`
  - storage types apply to `ProvisionKeyspace`, `RemoveKeyspace`, `ManageSchema`,
    `ManageAccess`, and `ManageSemantic`; `ManageSnapshots` is always in-memory
- `ProvisionKeyspace` is treated as a store-level capability, so its policy commands
  omit `keyspace`.

### Added
- Data-mesh governance policy APIs on `Engine`:
  - inspection: `policy_view`, `policy_history`, `policy_explain`, `policy_export`
  - mutation: `policy_grant`, `policy_revoke`, `policy_deny`, `policy_remove_denial`
  - dry runs: `policy_preview_grant`, `policy_preview_revoke`
  - manifests: `policy_validate`, `policy_plan`, `policy_apply`
- `PolicyCapability`, `PolicyKeyspaceType`, `SemanticModel`, and `PolicyFormat`,
  exported from the crate root.
- Keyspace-scoped semantic enrollment and removal through
  `enable_semantic_search_for_keyspace` and `disable_semantic_search_for_keyspace`.
- Hybrid semantic search through `semantic_search_get_keys_where` and
  `semantic_search_get_values_where`.
- Metadata criteria use the same shape as `lookup_keys_where` and act as a
  hard AND pre-filter for the vector search; ranking remains cosine similarity.
- Optional `min_score` filtering for hybrid queries.

## [0.2.1] - 2026-07-19

### ⚠️ Breaking
- **Semantic search response shape**: each hit is now `{__key__, __score__, __value__}`
  (was `{key, score, value}`); `semantic_search_get_keys` returns `{__key__, __score__}`.
  Aligns with the dunder envelope `lookup_values` returns with `key_included`. Wire-breaking
  for code that read the old `key`/`score`/`value` field names off the parsed payload.

## [0.2.0] - 2026-07-10

Semantic search, per-request/DB-wide index-wait control, and a batch of operator
commands — plus a correctness fix to `insert_bulk`. Contains breaking changes to
every write/delete method signature (see below), hence the minor bump.

### ⚠️ Breaking
- **`wait_for_index` parameter on all write/delete methods**: every insert/update/delete
  now takes a trailing `wait_for_index: Option<bool>` — `insert_value`, `insert_custom_key`,
  `insert_bulk`, `insert_bulk_no_schema`, `update_value` (on `InMemoryKeyspace` and
  `PersistentKeyspace`), and `delete_key`, `delete_bulk`, `update_bulk` (on the `Keyspace`
  trait). `Some(true)` makes a persistent write return only after its index is updated
  (read-your-writes); `Some(false)` is fire-and-forget; `None` uses the DB-wide default.
  Existing calls must add a trailing `None`.

### Fixed
- **`insert_bulk` / `insert_bulk_no_schema` created a single record instead of many**:
  the whole `Vec` was serialized into one `value` under `command = "insert_value"`, so a
  bulk insert stored one record whose value was the array. They now send
  `command = "insert_bulk"` with per-element `bulk_values`, creating one record per element
  (matching the server and the other SDKs). `process_bulk_values` now returns
  `Vec<String>` (one JSON string per value) rather than one concatenated string.

### Added
- **Semantic (vector) search** on the `Keyspace` trait:
  - `semantic_search_get_keys(query, limit, min_score)` → ranked `{key, score}` hits.
  - `semantic_search_get_values(query, limit, min_score, with_pointers, pointers_metadata)`
    → ranked hits with values inline (key always included). Empty query → `ClientNoValidInputProvided`.
- **Semantic search control on `Engine`**:
  - `enable_semantic_search(model, field, store)` / `disable_semantic_search(drop_vectors, store)`
    — DB-wide, or scoped to a single store when `store` is `Some`.
- **`wait_for_index` DB-wide default on `Engine`**: `enable_wait_for_index()` / `disable_wait_for_index()` (superowner).
- **Operator/admin commands on `Engine`** (superowner): `enable_reports()` / `disable_reports()`,
  `allow_subscriptions()` / `restrict_subscriptions()`, `queue_depths()` (returns per-queue
  depth maps), `set_snapshot_rate(rate)`, `set_expiration_check_rate(rate)` (value ×900s server-side).
- **`StoreRequestClient` fields** `min_score: Option<f32>` and `wait_for_index: Option<bool>`,
  both `#[serde(skip_serializing_if = "Option::is_none")]` so the wire is unchanged for callers
  that don't set them.

### Changed
- **`#[derive(RuntimeSchema)]` ergonomics** (via `montycat_serialization_derive` 0.1.7): the
  derive now emits fully-qualified paths (`::montycat::…`, `::std::collections::HashMap`), so you
  no longer need `use montycat::{Pointer, Timestamp};` or `use std::collections::HashMap;` next to
  the derive — just `use montycat::RuntimeSchema;`. (Structs with `Pointer`/`Timestamp` *fields*
  still import those to name the field types.)

## [0.1.7]
### Fixed
- **get_bulk API**: Corrected `StoreRequestClient` initialization where `volumes` and `latest_volume` fields were being ignored.
- **get_bulk Validation**: Refactored strict validation logic to allow any valid combination of `volumes`, `latest_volume`, and `limit` as a group, while maintaining mutual exclusivity with direct key retrieval.
- **subscribe API**: Added support for subscription port if specified.
### Fixed
- **get_bulk API**: Prevented a panic when both `bulk_keys` and `bulk_custom_keys` are `None` — `merge_keys` is now skipped and an empty key list is returned instead.
- **Key/Custom Key Validation**: Corrected inverted validation logic in the `Keyspace` trait — `ClientSelectedBothKeyAndCustomKey` now correctly triggers when *both* `key` and `custom_key` are provided, not when neither is.

### Changed
- **get_bulk Validation (InMemoryKeyspace)**: Volume-scope guard now requires that at least one of `volumes` (non-empty) or `latest_volume=true` is set, rather than only erroring when `latest_volume` is false without volumes.
- **`ClientSelectedBothPointersValueAndMetadata` error removed**: The restriction preventing simultaneous use of `with_pointers` and `with_pointers_metadata` has been lifted. Both can now be requested together.

### Added
- **`get_keys` — `limit.stop=0` sentinel**: `stop=0` now means "return all records" when the query is volume-scoped (`volumes` or `latest_volume` provided). A nonzero `stop` must still be ≥ `start`.
- **`get_keys` query parameter enforcement**: At least one of `volumes`, `latest_volume`, or a nonzero `limit` must be provided; omitting all three now returns a `ClientGenericError`.

## [0.1.6]
### Changed
- **get_bulk Parameter Handling**: Improved `limit_map` generation to use inclusive reference borrowing and more robust error checking for start/stop bounds.

## [0.0.5]
### Improved
- **Code Robustness**: Enhanced `get_bulk` signature and internal key merging logic to better handle complex query scenarios involving both custom and internal keyspace formats.

## [0.1.4]
## Added
- Volume-Based Bulk Retrieval
- Volume Filtering in get_bulk()
- Added support for retrieving values by specific volumes within the get_bulk() function.
- Enables targeted data access across selected storage volumes.
- Improves flexibility for multi-volume environments and sharded datasets.
- Latest Volume Retrieval
- Introduced latest_volume flag in get_bulk() to fetch values only from the most recent volume.
- Ensures efficient access to the newest version of data without scanning all volumes.
- Optimized for versioned and time-based storage architectures.

### Improved
- Performance & Query Precision
- Enhanced internal filtering logic to reduce unnecessary volume scans.
- Improved bulk query execution flow when volumes and latest_volume are specified.
- More deterministic behavior when both filtering options are used together.

### Technical Notes
`get_bulk()` now supports:
volumes: `Option<Vec<String>>`
latest_volume: `Option<bool>`

- Fully backward compatible — existing calls without volume parameters behave as before.

### Testing
- Extended unit tests to cover:
- Volume-specific retrieval scenarios
- Latest volume resolution logic
- Combined filtering edge cases
- All existing tests pass successfully.

## [0.1.3]

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

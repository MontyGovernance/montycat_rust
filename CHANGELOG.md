# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0]

Opt-in connection pooling, two semantic-configuration commands, and fixes to
response framing and subscription frame delivery.

Upgrading from 0.3.2 needs no code changes **unless** you construct `Engine`
with a struct literal — see Breaking below. Runtime behavior is unchanged until
you opt into pooling, with one exception: a subscription callback now fires once
per frame rather than once per socket chunk.

### ⚠️ Breaking

- **`Engine` gained a private field, so struct-literal construction no longer
  compiles.** Use the constructors:

  ```rust
  let engine = Engine { host: "…".into(), port: 21210, /* … */ };  // no longer compiles
  let engine = Engine::new("…".into(), 21210, /* … */);            // use this
  let engine = Engine::from_uri("montycat://user:pass@host:21210/store")?;
  ```

  The field holds an optional connection pool and is `#[serde(skip)]`, so
  `Engine` stays serializable and a deserialized engine simply has no pool.

- **Write and semantic-search methods gained a vector parameter, so every call
  site needs one more argument.** Rust has no default arguments, so unlike the
  Dart, Python, and Node clients — where the equivalent parameter is optional
  and named — this does not compile until updated. Pass `None` to keep today's
  behavior:

  ```rust
  keyspace.insert_value(None, value, None, None).await?;          // no longer compiles
  keyspace.insert_value(None, value, None, None, None).await?;    // vector: None
  ```

  Affected: `insert_value`, `insert_value_no_schema`, `update_value`,
  `insert_bulk`, and `insert_bulk_no_schema` on both `KeyspaceInMemory` and
  `KeyspacePersistent`, gaining `vector: Option<Vec<f32>>` or
  `vectors: Option<Vec<Vec<f32>>>` after `value` / `bulk_values`;
  `update_bulk`, gaining `vectors` and `custom_vectors`; and the four
  `semantic_search_*` methods, gaining `vector: Option<Vec<f32>>` after `query`.

  The mechanical fix is one `None` per call site — the compiler lists every one.

### Added

- **Precomputed vectors.** Vectors produced elsewhere — another model, a batch
  pipeline, an existing embedding store — can now be supplied directly, and the
  server skips embedding entirely. Requires a Montycat Semantic server 1.3.0 or
  newer. See Breaking above for the call-site impact.

  ```rust
  // Writing: the vector is applied after the write succeeds.
  keyspace.insert_value(None, doc, Some(my_embedding), None, None).await?;

  // Bulk: paired with bulk_values by position.
  keyspace.insert_bulk(docs, Some(vec![emb1, emb2]), None, None).await?;

  // Searching: a query vector bypasses text embedding; query may be empty.
  keyspace.semantic_search_get_values("", Some(my_query_embedding), /* … */).await?;
  ```

  Dimensions must match the keyspace's enrolled model; the server validates
  before anything reaches the index. A supplied vector is not overwritten by
  background embedding — a later ordinary write to the same item clears that
  protection and re-embeds from text.

- **`MontycatClientError` now implements `Display` and `std::error::Error`.**
  It previously derived only `Debug`, `Clone`, `Serialize`, and `Deserialize`,
  so it did not compose with `?`, `Box<dyn Error>`, `anyhow`, or `thiserror`'s
  `#[from]`. The usual shape of an `axum` or `tokio` `main` failed to compile:

  ```rust
  #[tokio::main]
  async fn main() -> Result<(), Box<dyn std::error::Error>> {
      let engine = Engine::from_uri("montycat://user:pass@host:21210/store")?;   // now compiles
      // previously required: .map_err(|e| e.message())?  at every call site
  }
  ```

  `Display` renders exactly what `message()` returns, so existing output is
  unchanged. `source()` returns `None`: the variants carry rendered strings
  rather than an underlying error value, so there is no cause to chain to.

- **Opt-in connection pooling.** Every request previously opened a TCP
  connection, sent one request, read one response, and closed. Reusing a
  connection measures **2.56x faster** on loopback against a debug engine
  (161 µs → 63 µs per `list-owners`); the gap widens over a network, where the
  handshake costs a full round trip before the query is sent, and widens again
  with TLS.

  ```rust
  let engine = Engine::from_uri("montycat://user:pass@127.0.0.1:21210/mystore")?
      .with_pool(PoolConfig::default());   // the only new line

  persistent.insert_value(None, employee, None).await;   // unchanged
  ```

  Disabled by default, so no existing deployment changes behavior on upgrade —
  an idle pooled connection still holds one of the engine's connection permits,
  so the bound is deliberately conservative (`max_idle: 8`, 30s idle timeout).
  Subscriptions are never pooled. Reuse one `Engine` for the process lifetime:
  cloning shares the pool, but constructing a fresh one per request builds a
  fresh empty pool and amortises nothing. Call `Engine::close_pool()` before
  exit so TLS connections close with `close_notify`.

  Exported `PoolConfig` and `ConnectionPool` from the crate root.

- `Engine::get_semantic_status(store, keyspace)` returns the server's actual
  semantic settings rather than what the caller assumed: the DB-wide switch and
  default model, plus each enrolled keyspace's model, dimensions, field,
  storage type, and whether a backfill is still pending.
- `Engine::reembed_semantic_search(store, keyspace, model, field)` atomically
  drops one keyspace's vectors, records the new configuration, and starts a
  complete backfill. It reports the previous model alongside the new one, so a
  caller can confirm what it replaced.

### Fixed

- **A subscription frame could be delivered concatenated with the next one, or
  dropped entirely.** The subscription reader appended raw socket chunks and
  invoked the callback with whatever had accumulated, then cleared the buffer.
  When two frames arrived in a single chunk the callback received both as one
  event; when a chunk ended mid-frame after a complete one, the partial frame
  was discarded by the clear. Frames are now read one at a time and the callback
  fires once per frame.

- **Reading a response rescanned its whole buffer on every chunk.** The reader
  tested `buf.contains(&b'\n')` after each 256 KiB read, making a large response
  O(n²) in the number of chunks. Responses are now read with
  `BufReader::read_until`, which is O(n) and stops at the frame boundary instead
  of retaining bytes belonging to whatever comes next — a prerequisite for the
  connection pooling in `CONNECTION_POOLING_PLAN.md`.

### Changed

- Documented that `enable_semantic_search` leaves an already-enrolled keyspace
  alone. It was never a way to switch models; `reembed_semantic_search` is.
  Behavior is unchanged — only the documentation was misleading.
- Corrected the `disable_semantic_search` docs: `drop_vectors` is not "required
  before switching to a different embedding model". Use
  `reembed_semantic_search`, which does not leave the keyspace unsearchable in
  between.

## [0.3.2]

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

## [0.3.1]

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

## [0.3.0]

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

## [0.2.1]

### ⚠️ Breaking
- **Semantic search response shape**: each hit is now `{__key__, __score__, __value__}`
  (was `{key, score, value}`); `semantic_search_get_keys` returns `{__key__, __score__}`.
  Aligns with the dunder envelope `lookup_values` returns with `key_included`. Wire-breaking
  for code that read the old `key`/`score`/`value` field names off the parsed payload.

## [0.2.0]

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

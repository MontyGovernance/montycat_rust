# 🦀 Montycat for Rust — The AI-Native NoSQL Database with Semantic Search for RAG & Agents

### Abolish the two-database stack.

The official async Rust client for [Montycat](https://montygovernance.com) — a self-hosted **NoSQL + vector database** with AI **semantic search** forged into the core, built for **RAG and AI-agent memory**. One Rust engine, not a sprawl of services. **Your hardware. Your data. Your meaning.**

[![Crates.io](https://img.shields.io/crates/v/montycat.svg)](https://crates.io/crates/montycat)
[![Downloads](https://img.shields.io/crates/d/montycat.svg)](https://crates.io/crates/montycat)
[![Docs.rs](https://docs.rs/montycat/badge.svg)](https://docs.rs/montycat)
[![Docker Pulls](https://img.shields.io/docker/pulls/montygovernance/montycat)](https://hub.docker.com/r/montygovernance/montycat)
[![CI](https://github.com/MontyGovernance/montycat_rust/workflows/CI/badge.svg)](https://github.com/MontyGovernance/montycat_rust/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/MontyGovernance/montycat_rust/blob/master/LICENSE)

```rust
// Search your data by MEANING — no external APIs, no separate vector database.
// (already ON by default in the montycat-semantic server edition)
let hits = keyspace
    .semantic_search_get_values("Show all Bluetooth devices", None, None, false, false)
    .await;
// → [{ __key__: 123..., __score__: 0.82, __value__: { "name": "Wireless Headphones" }}]
```

> ### 🧩 All-in-one. AI-native. **Zero external dependencies.**
> The vector-embedding engine runs **inside** the database — **no** separate vector DB, **no** embedding API, **no** API keys, **no** sidecar service. One engine, one binary, your hardware.

## 🦀 What Is Montycat?

For a generation we were told the price of intelligence was two systems: a database for your records, and a separate vector store — with its per-query bill — for their meaning. Montycat rejects that tax. It is a **self-hosted NoSQL + vector database**: one Rust-powered engine with semantic search built in, so **RAG, AI-agent memory, and vector search** live where your data already lives. No cloud lock-in. No ops headache.

Think of it as an **open-source, self-hosted alternative to Pinecone, Weaviate, Chroma, Qdrant, and Redis** — a **vector database _and_ a NoSQL store in a single engine**. Built entirely in Rust and exposed through a client that *is* Rust, not a wrapper around legacy C: no bloated SQL, no fragile ORMs, no half-baked drivers. Just pure async power, memory safety, and a structured API that works exactly the way a Rust developer expects. Montycat isn't a database inspired by Rust. It is a break with the databases you know.

## 🦾 Built Different — The Montycat Philosophy

- Rust-native, not Rust-compatible. Every API, trait, and type is designed for idiomatic Rust, 100% safe code, not ported from a C library.
- No Query Languages. No SQL, no CQL, no “whateverQL”. Just structured, safe function calls.
- No Glue Code. Forget about ORM mappers or DSLs. Montycat works directly with your Rust structs.
- No Nonsense. One protocol, one codepath, maximum performance.
- Montycat isn’t a database “inspired by Rust.”
- Montycat is Rust — in database form.

## ⚡ Montycat Rust Client

- The Montycat Rust Client is the official, fully asynchronous interface to the Montycat engine. It’s built for developers who value performance and beauty in equal measure — offering the cleanest API, lowest latency, and strongest safety guarantees in the industry. If you’ve ever struggled with clunky, unsafe, or inconsistent database clients, welcome home. Montycat is the only database client that looks and feels like Rust — not like a wrapper around legacy code.

- Whether you’re building analytics dashboards, real-time messaging, or structured data storage, Montycat Client brings speed, reliability, and simplicity right into your Rust app.

- Unlike ugly SQL/NoSQL systems that force rigid schemas, inconsistent APIs, or costly drivers, it is designed from the ground up for Rust — blending speed, safety, and simplicity into a unified experience.

## Feature	Description

- 🧩 `Async-First Design`	Built on Tokio for fully asynchronous networking and I/O — no blocking, no lag. Compatible with all major crates - Tokio, Actix, Warp, Axum, etc.
- 💾 `Persistent + In-Memory Keyspaces` Combine ultra-fast in-memory stores with durable persistence — dynamically, within the same engine.
- 🧬 `Runtime Schemas` Enforce and evolve schemas at runtime using #[derive(RuntimeSchema)]. Change data structures on the fly. Natively use Rust Structs as data schemas for your database!
- 🔍 `Dynamic Querying` Effortlessly and organomically retrieve structured data without complex ORM overhead.
- 🔄 `Real-Time Subscriptions` Subscribe to live keyspace or key updates with callback-based reactive streams. Ideal for dashboards and event-driven apps.
- 🔐 `Secure by Default` No SQL, CQL, WhateverQL - no injection possible. Only structred tiny API. Native TLS support ensures encrypted and authenticated communication across distributed nodes.
- 🕒 `Timestamped Data` Built-in timestamp support via Montycat::Timestamp for precise event tracking and data lineage.
- 🧭 `Native Foreign Keys Supports` Pointer-based integrity, just like SQL foreign keys — without the performance overhead or complexity.
- 🧠 `Schema-Aware Serialization` Fully compatible with serde and serde_json::Value for seamless encoding/decoding.
- 🧱 `Client Memory-Safe and Zero-Copy` Written entirely in Rust — leveraging ownership and zero-cost abstractions for maximum efficiency and no GC overhead.
- 🕹️ `Developer-Centric Ergonomics` Clean, composable APIs that make even complex data interactions intuitive. The easiest database client for Rust!

## 🔍 Example Use Cases

- **RAG pipelines & semantic retrieval** for LLM-powered Rust services
- **AI agent long-term memory** that survives restarts
- **Semantic search & recommendations** — match intent, not keywords
- Real-time dashboards and event-driven systems (Tokio, Axum, Actix, Warp)
- High-throughput microservice data stores
- Data products in a decentralized Mesh architecture

## 🚀 Get the Engine (30 seconds)

The client talks to a Montycat server. Fastest way — Docker, with AI semantic search built in:

```bash
docker run -d --name montycat \
  -p 21210:21210 -p 21211:21211 \
  -e MONTYCAT_SUPEROWNER="admin" \
  -e MONTYCAT_PASSWORD="change-me" \
  -v montycat_data:/var/lib/.montycat \
  montygovernance/montycat:semantic
```

Prefer the lean edition without the embedding engine? Use the `latest` tag. Prebuilt packages (apt, macOS, Windows) at **https://montygovernance.com**.

## Installation

Add the client to your `Cargo.toml`:

```toml
[dependencies]
montycat = "0.3"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

With TLS:

```toml
montycat = { version = "0.3", features = ["tls"] }
```

## Quick Start

```rust
use montycat::{Engine, InMemoryKeyspace, PersistentKeyspace, RuntimeSchema, MontycatResponse, Keyspace};
use serde::{Serialize, Deserialize};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // Connect to Montycat engine
    let engine = Engine::from_uri("montycat://USER:PASS@127.0.0.1:21210/mystore").unwrap();

    // Persistent and in-memory keyspaces
    let persistent = Arc::new(PersistentKeyspace::new("employees", &engine));
    let in_mem = Arc::new(InMemoryKeyspace::new("employeesInMem", &engine));

    // Create keyspaces
    let (res_persist, res_mem) = tokio::join!(
        persistent.create_keyspace(None, None),
        in_mem.create_keyspace()
    );

    println!("Persistent keyspace: {:?}", res_persist);
    println!("In-memory keyspace: {:?}", res_mem);

    // Define a schema
    #[derive(Serialize, Deserialize, RuntimeSchema, Clone, Debug)]
    struct Employee {
        id: u32,
        name: String,
    }

    // Insert a value
    let employee = Employee {
        id: 1,
        name: "Monty".to_string(),
    };

    // insert_value(custom_key, value, [expire (in-memory only),] wait_for_index)
    let insert_res_in_mem = in_mem.insert_value(None, employee.clone(), None, None).await;
    println!("Insert response: {:?}", insert_res_in_mem);

    let insert_res_pers = persistent.insert_value(None, employee, None).await;
    println!("Insert response: {:?}", insert_res_pers);

    let search_criteria = serde_json::json!({
        "name": "Monty"
    });

    // Lookup values where name is Monty
    let lookup_res_in_mem = in_mem.lookup_values_where(search_criteria.clone(), None, false, true, false, None).await;
    // Parse into desired type
    let parsed = MontycatResponse::<Option<Employee>>::parse_response(lookup_res_in_mem);
    println!("Lookup response: {:?}", parsed);

    // Lookup values where name is Monty and Schema is Employee
    let lookup_res_pers = persistent.lookup_values_where(
        search_criteria,
        None,
        false, true, false,
        Some(Employee::schema_params())
    ).await;

    // Parse into desired type
    let parsed = MontycatResponse::<Option<Employee>>::parse_response(lookup_res_pers);
    println!("Lookup response: {:?}", parsed);

}
```

## 🧠 AI-Native Semantic Search — Vector Search Built Into Your Database

**Stop bolting a separate vector database onto your stack.** Montycat ranks your data by
*meaning*, not keywords — an embedded, on-device vector-embedding engine turns every write
into a searchable vector automatically. It's the retrieval layer for **RAG pipelines, AI
agents, semantic search, recommendation engines, and LLM-powered apps** — with **zero
external APIs, zero API keys, and zero extra infrastructure.**

- 🔎 **Semantic / vector search** — kNN similarity over on-device embeddings, not brittle keyword matches.
- 🤖 **Built for AI** — RAG, semantic retrieval, AI agents, recommendations, dedup, clustering.
- 🔒 **Private & free** — embeddings never leave your machine. No OpenAI/Cohere bill, no data egress.
- ⚡ **One system, not two** — your data *and* its vectors live in the same database. No sync jobs, no drift, no second service to run.
- 🚀 **Zero setup** — no index tuning, no pipeline: `enable_semantic_search()` and you're ranking by meaning.

> **⚠️ Requires the semantic edition of the server — nothing to compile.** Semantic
> search runs an embedded ONNX vector-embedding engine that ships only in the
> **`montycat-semantic`** edition; the default lean `montycat` server does not include it.
> Get it the way that suits you — pull the **Docker image**
> (`montygovernance/montycat:semantic`), download the prebuilt **package**, or install
> `montycat-semantic` from the **apt repository**. The Rust client API is identical either
> way; just point it at a semantic-edition server (semantic search is enabled by default
> there, using the `bge-small` model).

Beyond exact-match `lookup_keys_where` / `lookup_values_where`, Montycat ranks stored
items by *meaning* using on-device vector embeddings — no external API, no extra service,
no separate vector database. It's ON by default in the semantic edition, so just search.

```rust
use montycat::{Keyspace, Limit, MontycatResponse, SemanticModel};

// (reuses the `engine` and `persistent` keyspace from the Quick Start above)

// Rank stored items by meaning — two flavors:
//   get_values -> each hit is { __key__, __score__, __value__ }
//   get_keys   -> each hit is { __key__, __score__ } (lighter; fetch a page later with get_bulk)
let values = persistent
    .semantic_search_get_values("Show all Bluetooth devices", Some(Limit { start: 0, stop: 5 }), None, false, false)
    .await;

// Keys only, with a cosine-similarity floor (range [-1, 1]):
let _keys = persistent
    .semantic_search_get_keys("Show all Bluetooth devices", Some(Limit { start: 0, stop: 5 }), Some(0.35))
    .await;

let parsed = MontycatResponse::<Vec<serde_json::Value>>::parse_response(values);
println!("{:?}", parsed);

// Control the DB-wide switch (optional — it's already on):
// Read back the model and backfill state actually assigned to a keyspace.
engine
    .get_semantic_status(Some("catalog"), Some("products"))
    .await?;

// Enable an unenrolled keyspace with an explicit model.
engine
    .enable_semantic_search_for_keyspace(
        "catalog",
        "products",
        Some(SemanticModel::BgeBase),
        None,
    )
    .await?;

// Changing an enrolled keyspace is destructive and starts a full backfill.
engine
    .reembed_semantic_search(
        "catalog",
        "products",
        SemanticModel::BgeBase,
        None,
    )
    .await?;
```

### Hybrid semantic search

Restrict semantic ranking to records matching structured metadata. The filter
is a hard AND pre-filter with the same criteria shape as `lookup_keys_where`;
it does not boost or otherwise alter cosine scores.

```rust
let filtered = persistent
    .semantic_search_get_values_where(
        "astronomy and outer space",
        serde_json::json!({"category": "space"}),
        Some(Limit { start: 0, stop: 5 }),
        Some(0.35),
        false,
        false,
    )
    .await;

let parsed = MontycatResponse::<Vec<serde_json::Value>>::parse_response(filtered);
// each hit: {"__key__": ..., "__score__": ..., "__value__": ...}
println!("{:?}", parsed);
```

### Bring your own vectors

If you already have embeddings from a batch pipeline or vector store, first
enroll the keyspace for externally generated vectors. External profiles support
1–4,096 dimensions for OpenAI-style 1,536d pipelines, Pinecone/Qdrant/Milvus
migrations, and image or multimodal vectors:

```rust
items.create_keyspace_without_semantic(None, None).await?;
engine.enable_precomputed_vector_search("app", "items", 1536, "text-embedding-3-small:v1").await?;
```

`embedding_space` is a descriptive name for the vectors' model/configuration;
it does not invoke or validate that model. Then supply vectors directly and the
server skips embedding.
Needs a Montycat Semantic server 1.3.0 or newer.

```rust
// Writing: the vector is applied after the write succeeds.
keyspace
    .insert_value(None, doc, Some(my_embedding), None, None)
    .await?;

// Bulk: paired with bulk_values by position.
keyspace
    .insert_bulk(docs, Some(vec![embedding1, embedding2]), None, None)
    .await?;

// Searching: a query vector bypasses text embedding, so the query may be empty.
let hits = keyspace
    .semantic_search_get_values("", Some(my_query_embedding), None, None, false, false)
    .await?;
```

`vector` is also accepted by `insert_custom_key_value` and `update_value`, and
`update_bulk` takes `vectors` for numeric keys plus `custom_vectors` for custom
keys. All four `semantic_search_*` methods accept a query vector. Pass `None`
anywhere you want the server to embed.

**Embedding-space compatibility is required.** Every supplied record vector and
query vector must be produced by the model enrolled for that keyspace, including
the same model revision, preprocessing, pooling, and normalization. Matching the
dimension alone is not enough: an auto-enrolled BGE-small keyspace accepts only
BGE-small-compatible 384d vectors. To use vectors from another model, create the
keyspace with semantic auto-enrollment disabled and enroll a matching external
profile first. The server validates dimensions before anything reaches the
index, but it cannot prove that two equal-length vectors came from the same
embedding space. A vector you supplied will not be
overwritten by background embedding; a later ordinary write to that item clears
the protection and re-embeds from its text, which is when re-embedding is what
you want.

Mixing is fine: items with supplied vectors and items the server embeds can
live in one keyspace as long as every vector comes from the same model.

## 📨 Response Shape

Commands return `Result<Option<Vec<u8>>, MontycatClientError>` — raw bytes off the wire.
`MontycatResponse::parse_response` turns them into a typed envelope, so you pick the
payload type at the call site instead of hand-rolling `serde_json`:

```rust,ignore
use montycat::MontycatResponse;

// { "status": true,  "payload": <result>, "error": null }
// { "status": false, "payload": null,     "error": "Governance permission denied: ..." }

let raw = persistent.insert_value(None, employee, None).await;
let parsed: MontycatResponse<Option<String>> = MontycatResponse::parse_response(raw)?;

if parsed.status {
    println!("new key: {:?}", parsed.payload);
} else {
    eprintln!("server rejected it: {:?}", parsed.error);
}
```

`parse_response` also unwraps payloads the server double-encodes as JSON strings, so a
nested document arrives as a real structure. **Keys are u128 and travel as strings** —
deserialize them into `String`, not an integer type. Client-side mistakes surface as
`MontycatClientError` before anything is sent; server-side failures arrive with
`status: false` and a populated `error`.

`MontycatClientError` implements `Display` and `std::error::Error`, so it composes with
`?`, `Box<dyn Error>`, `anyhow`, and `thiserror`'s `#[from]`:

```rust,ignore
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = Engine::from_uri("montycat://user:pass@127.0.0.1:21210/mystore")?;
    Ok(())
}
```

## 🔄 Connection Pooling

By default every request opens a TCP connection, sends, reads one response, and closes.
Reuse the connection instead and the handshake disappears — **2.56x faster** on loopback
against a debug engine (161 µs → 63 µs per `list-owners`). That is the conservative
figure: over a network the handshake costs a full round trip before the query is even
sent, and TLS adds one or two more.

Pooling is opt-in. Enabling it is one line, and no call site changes:

```rust,ignore
use montycat::{Engine, PoolConfig};

let engine = Engine::from_uri("montycat://user:pass@127.0.0.1:21210/mystore")?
    .with_pool(PoolConfig::default());          // ← the only new line

persistent.insert_value(None, employee, None).await;   // unchanged

// Before exit, in a long-lived process:
engine.close_pool().await;
```

Tune it if you need to:

```rust,ignore
use std::time::Duration;

let engine = engine.with_pool(PoolConfig {
    max_idle: 4,                              // default 8
    idle_timeout: Duration::from_secs(15),    // default 30s
});
```

**Reuse one `Engine` for the process lifetime.** Cloning is cheap and shares the pool, so
keyspaces built from it reuse connections. Building a fresh
`Engine::from_uri(..).with_pool(..)` per request creates a fresh *empty* pool every time —
you still pay every handshake and nothing is ever amortised.

**Pooling is per-`Engine`, not global.** Two engines pointing at the same server keep
separate pools.

**Call `close_pool()` before exit.** `Drop` cannot be async, so without it TLS connections
close without `close_notify` and the server logs an error for each one.

**Keep `max_idle` modest.** An idle pooled connection still holds one of the engine's
connection permits. The defaults are deliberately small; raise them only after measuring
with the `queue-depths` command under realistic load.

Subscriptions are never pooled — they are long-lived, stream many responses to one
request, and live on their own port.

## 📡 Real-Time Subscriptions

Subscribe to one key or to a whole keyspace and get pushed every change — the reactive
core behind live dashboards and event-driven Tokio services.

```rust,ignore
use montycat::{Keyspace, MontycatStreamResponse};
use std::sync::Arc;

// Whole keyspace: pass None for both key and custom_key.
let stop = persistent
    .subscribe(
        None, // subscription port; None => engine port + 1
        None,
        None,
        Arc::new(|bytes: &mut [u8]| {
            let event = MontycatStreamResponse::<serde_json::Value>::parse_response(bytes);
            println!("changed: {:?}", event);
        }),
    )
    .await?;

// Stop the stream. Passing key and custom_key together returns
// MontycatClientError::ClientSelectedBothKeyAndCustomKey instead of subscribing.
let _ = stop.send(true);
```

`subscribe` spawns the listener on Tokio and hands back a `tokio::sync::watch::Sender<bool>`;
sending `true` ends it. The callback is an `Arc<dyn Fn(&mut [u8]) + Send + Sync>` receiving
each frame as raw bytes — parse it with `MontycatStreamResponse::parse_response`, which
carries an extra `message` field alongside the usual `status` / `payload` / `error`.
Subscriptions use the **subscription port**, `engine.port + 1` by default — that is the
second port (`21211`) published in the Docker command above.

## 🔐 TLS

TLS is behind a feature flag, so the dependency tree stays lean when you do not need it:

```toml
montycat = { version = "0.3", features = ["tls"] }
```

```rust,ignore
let mut engine = Engine::from_uri("montycat://USER:PASS@127.0.0.1:21210/mystore")?;
engine.enable_tls();
```

`Engine::new(..)` takes the same switch as its final `use_tls` argument. It applies to
commands and subscriptions alike.

For a server with a publicly trusted certificate, enabling TLS is sufficient. For a
private CA or a self-issued MontyCat certificate, explicitly add the PEM trust anchor:

```rust,ignore
let engine = Engine::from_uri("montycat://USER:PASS@montycat.internal:21210/mystore")?
    .with_tls_ca_file("/etc/montycat/tls/ca.pem")?;
```

`with_tls_ca_file` is optional: it adds its certificates alongside the normal public
WebPKI roots and enables TLS for that engine. Rustls still validates the certificate
chain and the server hostname. The certificate must therefore contain the name or IP
address that the client connects to as a Subject Alternative Name.

## 👥 Owners & Access

Governance policies below are written against *owners*, so create them first. A
superowner provisions an owner, then grants data access — optionally narrowed to
specific keyspaces:

```rust,ignore
use montycat::ValidPermissions;

engine.create_owner("alice", "alice-password").await?;

// store: None falls back to the engine's store.
engine.grant_to("alice", ValidPermissions::Read, None, None).await?;
engine
    .grant_to("alice", ValidPermissions::Write, None, Some(vec!["employees"]))
    .await?;

engine.list_owners().await?;

engine
    .revoke_from("alice", ValidPermissions::Write, None, Some(vec!["employees"]))
    .await?;
engine.remove_owner("alice").await?;
```

`ValidPermissions` is `Read`, `Write`, or `All`. This governs **data access**; to
delegate *administrative* capabilities such as provisioning keyspaces or managing
schemas, see
[Data-mesh governance](#data-mesh-governance-for-shared-and-multi-tenant-deployments) at
the end of this document.

## Want more?

### 🧩 The Montycat Architecture
- Hybrid Engine Design: Seamlessly switch between persistent and in-memory data.
- Data Mesh by Design: Each keyspace is independently owned and domain-oriented.
- Reactive Core: Native subscription support makes Montycat perfect for live apps and real-time analytics.

### 🔐 Security & Reliability
- TLS-enabled client-server communication
- Encrypted authentication
- Strong data isolation between keyspaces
- Safe concurrency with Tokio + Rust guarantees

### 🏁 Lastly
- There are databases written in C, C++, Java, even Python. And then there’s Montycat — the only database that feels like Rust.
- Every other client library tries to hide its ugliness behind ORMs and drivers. Montycat doesn’t need to — it’s beautiful by design, safe by default, and fast beyond reason.

### 🏆 The Only Rust Database That Deserves Rust.
- 100% Async
- 100% Memory-Safe
- 100% Rust
- 0% Nonsense

## 🔗 Links

- 🌐 **Website & Docs** — https://montygovernance.com
- 📦 **crates.io** — https://crates.io/crates/montycat
- 📚 **docs.rs** — https://docs.rs/montycat
- 🐳 **Docker Hub** — https://hub.docker.com/r/montygovernance/montycat
- 💻 **Source** — https://github.com/MontyGovernance/montycat_rust
- 📝 **Changelog** — [CHANGELOG.md](CHANGELOG.md)

## ❓ FAQ

- **Is Montycat a vector database or a NoSQL database?** Both — one engine. Store records and query them by *meaning* (vector / semantic search) or by key/schema, without running two systems.
- **Do I need OpenAI or an embedding API?** No. Embeddings run on-device in the `montycat-semantic` server. No API keys, no per-query bill, no data egress.
- **Is it a Pinecone / Weaviate / Chroma / Qdrant alternative?** Yes — self-hosted and open-source, with a NoSQL store built in.
- **Which async runtime?** Tokio. Works with Axum, Actix, Warp, and any Tokio-based stack.

## Data-mesh governance for shared and multi-tenant deployments

Delegate administration without giving every team full server control. Policies scope
authority to an owner and store, with optional keyspace, storage-type, and semantic-model
constraints. Platform teams can govern shared infrastructure while domain teams operate
the data products they own.

- Grant, revoke, or explicitly deny keyspace provisioning/removal, schema, semantic,
  snapshot, and access-management capabilities.
- Inspect effective permissions and policy history, or preview a grant/revoke before
  applying it.
- Validate, plan, apply, and export JSON or YAML policy manifests for repeatable
  infrastructure-as-code workflows.
- Constrain storage types for provisioning, removal, schema, access, and semantic
  management. Snapshot management is always in-memory, so it takes no storage-type
  qualifier.
- Constrain semantic models during keyspace provisioning and semantic management.

For example, a superowner can restrict what Alice may provision and separately delegate
semantic management for one keyspace:

```rust,ignore
use montycat::{PolicyCapability, PolicyKeyspaceType, SemanticModel};

engine.policy_grant(
    "alice", PolicyCapability::ProvisionKeyspace, "catalog", None,
    &[PolicyKeyspaceType::InMemory, PolicyKeyspaceType::Persistent], &[SemanticModel::BgeSmall],
).await?;
engine.policy_grant(
    "alice", PolicyCapability::ManageSemantic, "catalog", Some("products"),
    &[PolicyKeyspaceType::InMemory], &[SemanticModel::BgeSmall],
).await?;
engine.policy_view(Some("alice"), Some("catalog")).await?;
```

Use `policy_explain` to inspect an authorization decision and `policy_history` to audit
changes. Superowners can manage policies directly with `policy_grant`, `policy_revoke`,
`policy_deny`, and `policy_remove_denial`, or use `policy_validate`, `policy_plan`,
`policy_apply`, and `policy_export` with JSON or YAML documents.

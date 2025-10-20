# 🚀 Rust Client for Montycat - High-Performance NoSQL Database. The Fastest, Safest, and Most Elegant Database Client Ever Built in Rust.

## 🦀 What Is Montycat?

Montycat isn’t just another database — it’s the future of data systems. Built entirely in Rust, Montycat redefines what performance, safety, and developer ergonomics mean in the NoSQL world. It fuses the best ideas from distributed data meshes, real-time streaming, and memory-safe system design — giving developers the only database engine that feels truly native to Rust. Montycat eliminates everything ugly about existing databases — no bloated SQL syntax, no fragile ORMs, no half-baked drivers. Just pure async power, memory safety, and a clean, structured API that works exactly the way a Rust developer expects.

## 🦾 Built Different — The Montycat Philosophy

- Rust-native, not Rust-compatible. Every API, trait, and type is designed for idiomatic Rust, 100% safe code, not ported from a C library.
- No Query Languages. No SQL, no CQL, no “whateverQL”. Just structured, safe function calls.
- No Glue Code. Forget about ORM mappers or DSLs. Montycat works directly with your Rust structs.
- No Nonsense. One protocol, one codepath, maximum performance.
- Montycat isn’t a database “inspired by Rust.”
- Montycat is Rust — in database form.

## `For installation of the Montycat Engine, see 👉 https://montygovernance.com`

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

## Installation

```bash
[dependencies]
montycat = { version = "1.0.1", features = ["tls"] }
tokio = { version = "1", features = ["full"] }
serde_json = "1"
serde = {version = "1", features = ["derive"]}
```

## `For installation of the Montycat Engine, see 👉 https://montygovernance.com`

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

    let insert_res_in_mem = in_mem.insert_value(employee, None).await;
    println!("Insert response: {:?}", insert_res_in_mem);

    let insert_res_pers = persistent.insert_value(employee, None).await;
    println!("Insert response: {:?}", insert_res_pers);

    let search_criteria = serde_json::json!({
        "name": "Monty"
    });

    // Lookup values where name is Monty
    let lookup_res_in_mem = in_mem.lookup_values_where(search_criteria, None, false, true, false, None).await;
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

## `For installation of the Montycat Engine, see 👉 https://montygovernance.com`

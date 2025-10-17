# 🚀 Rust Client for Montycat - High-Performance NoSQL Database

## MontyCat is a blazing-fast, fully asynchronous, and real-time Rust client for the MontyCat NoSQL database. Designed for modern applications that demand ultra-low latency, memory-safe operations, and full control over both persistent and in-memory keyspaces.

## Whether you’re building analytics dashboards, real-time messaging, or structured data storage, MontyCat brings speed, reliability, and simplicity right into your Rust app.

### Ultra-Fast Async Operations
Fully asynchronous Rust API powered by tokio. Insert, update, query, and subscribe to keyspaces in real-time.

### Persistent & In-Memory Keyspaces
Manage both ephemeral in-memory collections and durable persistent storage effortlessly.

### Dynamic Runtime Schemas
Enforce, remove, and validate complex schemas on the fly using RuntimeSchema derive macros.

### Flexible Querying
Lookup keys with filters, bulk fetch values, and track key dependencies.

### Real-Time Subscriptions
Subscribe to keyspace changes with callback-based streams for live updates.

### Secure Connections
Supports TLS for secure client-server communication.

### JSON + Timestamp Support
Seamless integration with serde_json::Value and MontyCat Timestamp.

### Lightweight, Developer-Friendly API
Clear, Rust-native ergonomics without boilerplate.

```bash
[dependencies]
montycat = "0.1.0"
montycat_serialization_derive = "0.1.6"
tokio = { version = "1.44.1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

Install engine ---->>

```rust
use montycat::{Engine, InMemoryKeyspace, PersistentKeyspace, RuntimeSchema, Timestamp};
use serde::{Serialize, Deserialize};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // Connect to Montycat engine
    let engine = Engine::from_uri("montycat://USER:PASS@127.0.0.1:21210/mystore").unwrap();

    // Persistent and in-memory keyspaces
    let persistent = Arc::new(PersistentKeyspace::new("employees", &engine));
    let in_mem = Arc::new(InMemoryKeyspace::new("employeesInMem", &engine));

    persistent.connect_engine(&engine);
    in_mem.connect_engine(&engine);

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
        created_at: Timestamp,
    }

    // Insert a value
    let employee = Employee {
        id: 1,
        name: "Eugene".into(),
        created_at: Timestamp::new("2023-10-10T10:10:10.000Z"),
    };

    let insert_res_in_mem = in_mem.insert_value(employee, None).await;
    println!("Insert response: {:?}", insert_res_in_mem);

    let insert_res_pers = persistent.insert_value(employee, None).await;
    println!("Insert response: {:?}", insert_res_pers);

}
```
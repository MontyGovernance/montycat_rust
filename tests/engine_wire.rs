use montycat::{
    Engine, PolicyCapability, PolicyFormat, PolicyKeyspaceType, SemanticModel, ValidPermissions,
};
use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

async fn capture_engine(expected_requests: usize) -> (Engine, tokio::task::JoinHandle<Vec<Value>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let server = tokio::spawn(async move {
        let mut requests = Vec::with_capacity(expected_requests);
        for _ in 0..expected_requests {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buffer = vec![0; 16 * 1024];
            let size = socket.read(&mut buffer).await.unwrap();
            requests.push(serde_json::from_slice(&buffer[..size]).unwrap());
            socket.write_all(b"{\"status\":true}\n").await.unwrap();
            socket.shutdown().await.unwrap();
        }
        requests
    });
    (
        Engine::new(
            "127.0.0.1".into(),
            port,
            "owner".into(),
            "secret".into(),
            Some("orders".into()),
            false,
        ),
        server,
    )
}

fn raw(request: &Value) -> &Vec<Value> {
    request["raw"].as_array().unwrap()
}

#[tokio::test]
async fn store_owner_access_and_semantic_commands_match_wire_contract() {
    let (engine, server) = capture_engine(12).await;

    engine.create_store().await.unwrap();
    engine.remove_store().await.unwrap();
    engine.create_owner("alice", "pw").await.unwrap();
    engine.remove_owner("alice").await.unwrap();
    engine.list_owners().await.unwrap();
    engine
        .grant_to(
            "alice",
            ValidPermissions::Read,
            None,
            Some(vec!["events", "users"]),
        )
        .await
        .unwrap();
    engine
        .revoke_from("alice", ValidPermissions::Write, None, Some(vec!["events"]))
        .await
        .unwrap();
    engine
        .enable_semantic_search(Some(SemanticModel::BgeSmall), Some("body"), Some("catalog"))
        .await
        .unwrap();
    engine
        .disable_semantic_search(true, Some("catalog"))
        .await
        .unwrap();
    engine
        .get_semantic_status(Some("catalog"), Some("products"))
        .await
        .unwrap();
    engine
        .reembed_semantic_search(
            "catalog",
            "products",
            SemanticModel::BgeBase,
            Some("description"),
        )
        .await
        .unwrap();
    engine
        .enable_precomputed_vector_search(
            "catalog",
            "embeddings",
            1536,
            "text-embedding-3-small:v1",
        )
        .await
        .unwrap();

    let requests = server.await.unwrap();
    let expected = [
        vec!["create-store", "store", "orders"],
        vec!["remove-store", "store", "orders"],
        vec!["create-owner", "username", "alice", "password", "pw"],
        vec!["remove-owner", "username", "alice"],
        vec!["list-owners"],
        vec![
            "grant-to",
            "owner",
            "alice",
            "permission",
            "read",
            "store",
            "orders",
            "keyspaces",
            "events,users",
        ],
        vec![
            "revoke-from",
            "owner",
            "alice",
            "permission",
            "write",
            "store",
            "orders",
            "keyspaces",
            "events",
        ],
        vec![
            "enable-semantic-search",
            "model",
            "bge-small",
            "field",
            "body",
            "store",
            "catalog",
        ],
        vec![
            "disable-semantic-search",
            "drop-vectors",
            "store",
            "catalog",
        ],
        vec![
            "get-semantic-status",
            "store",
            "catalog",
            "keyspace",
            "products",
        ],
        vec![
            "reembed-semantic-search",
            "model",
            "bge-base",
            "field",
            "description",
            "store",
            "catalog",
            "keyspace",
            "products",
        ],
        vec![
            "enable-semantic-search",
            "source",
            "external",
            "dimensions",
            "1536",
            "embedding-space",
            "text-embedding-3-small:v1",
            "store",
            "catalog",
            "keyspace",
            "embeddings",
        ],
    ];
    for (request, expected_raw) in requests.iter().zip(expected) {
        assert_eq!(raw(request), &expected_raw);
        assert_eq!(
            request["credentials"],
            serde_json::json!(["owner", "secret"])
        );
    }
}

#[tokio::test]
async fn governance_commands_match_wire_contract() {
    let (engine, server) = capture_engine(13).await;

    engine
        .policy_view(Some("alice"), Some("catalog"))
        .await
        .unwrap();
    engine
        .policy_history(Some("alice"), Some("catalog"), Some("products"))
        .await
        .unwrap();
    engine
        .policy_explain(
            PolicyCapability::ManageSemantic,
            "catalog",
            Some("alice"),
            Some("products"),
            Some(PolicyKeyspaceType::Persistent),
            Some(SemanticModel::BgeSmall),
        )
        .await
        .unwrap();

    engine
        .policy_grant(
            "alice",
            PolicyCapability::ProvisionKeyspace,
            "catalog",
            Some("ignored"),
            &[PolicyKeyspaceType::Persistent],
            &[SemanticModel::BgeSmall],
        )
        .await
        .unwrap();
    engine
        .policy_revoke(
            "alice",
            PolicyCapability::ProvisionKeyspace,
            "catalog",
            Some("ignored"),
            &[PolicyKeyspaceType::Persistent],
            &[SemanticModel::BgeSmall],
        )
        .await
        .unwrap();
    engine
        .policy_deny(
            "alice",
            PolicyCapability::ProvisionKeyspace,
            "catalog",
            Some("ignored"),
            &[PolicyKeyspaceType::Persistent],
            &[SemanticModel::BgeSmall],
        )
        .await
        .unwrap();
    engine
        .policy_remove_denial(
            "alice",
            PolicyCapability::ProvisionKeyspace,
            "catalog",
            Some("ignored"),
            &[PolicyKeyspaceType::Persistent],
            &[SemanticModel::BgeSmall],
        )
        .await
        .unwrap();
    engine
        .policy_preview_grant(
            "alice",
            PolicyCapability::ProvisionKeyspace,
            "catalog",
            Some("ignored"),
            &[PolicyKeyspaceType::Persistent],
            &[SemanticModel::BgeSmall],
        )
        .await
        .unwrap();
    engine
        .policy_preview_revoke(
            "alice",
            PolicyCapability::ProvisionKeyspace,
            "catalog",
            Some("ignored"),
            &[PolicyKeyspaceType::Persistent],
            &[SemanticModel::BgeSmall],
        )
        .await
        .unwrap();
    engine
        .policy_validate("rules: []", PolicyFormat::Yaml)
        .await
        .unwrap();
    engine
        .policy_plan("rules: []", PolicyFormat::Yaml)
        .await
        .unwrap();
    engine
        .policy_apply("rules: []", PolicyFormat::Yaml)
        .await
        .unwrap();
    engine.policy_export(PolicyFormat::Yml).await.unwrap();

    let requests = server.await.unwrap();
    assert_eq!(
        raw(&requests[0]),
        &vec!["policy-view", "owner", "alice", "store", "catalog"]
    );
    assert_eq!(
        raw(&requests[1]),
        &vec![
            "policy-history",
            "owner",
            "alice",
            "store",
            "catalog",
            "keyspace",
            "products",
        ]
    );
    assert_eq!(
        raw(&requests[2]),
        &vec![
            "policy-explain",
            "capability",
            "manage-semantic",
            "store",
            "catalog",
            "owner",
            "alice",
            "keyspace",
            "products",
            "type",
            "persistent",
            "model",
            "bge-small",
        ]
    );
    let operations = [
        "policy-grant",
        "policy-revoke",
        "policy-deny",
        "policy-remove-denial",
        "policy-preview-grant",
        "policy-preview-revoke",
    ];
    for (index, operation) in operations.iter().enumerate() {
        assert_eq!(
            requests[index + 3]["raw"],
            serde_json::json!([
                operation,
                "owner",
                "alice",
                "capability",
                "provision-keyspace",
                "store",
                "catalog",
                "types",
                "persistent",
                "models",
                "bge-small",
            ])
        );
    }
    assert_eq!(
        raw(&requests[9]),
        &vec!["policy-validate", "format", "yaml", "document", "rules: []"]
    );
    assert_eq!(
        raw(&requests[10]),
        &vec!["policy-plan", "format", "yaml", "document", "rules: []"]
    );
    assert_eq!(
        raw(&requests[11]),
        &vec!["policy-apply", "format", "yaml", "document", "rules: []"]
    );
    assert_eq!(raw(&requests[12]), &vec!["policy-export", "format", "yml"]);
}

#[tokio::test]
async fn operator_commands_match_wire_contract() {
    let (engine, server) = capture_engine(10).await;
    engine.get_structure_available().await.unwrap();
    engine.enable_wait_for_index().await.unwrap();
    engine.disable_wait_for_index().await.unwrap();
    engine.enable_reports().await.unwrap();
    engine.disable_reports().await.unwrap();
    engine.allow_subscriptions().await.unwrap();
    engine.restrict_subscriptions().await.unwrap();
    engine.queue_depths().await.unwrap();
    engine.set_snapshot_rate(5).await.unwrap();
    engine.set_expiration_check_rate(10).await.unwrap();

    let requests = server.await.unwrap();
    let expected = [
        vec!["get-structure-available", "store", "orders"],
        vec!["enable-wait-for-index"],
        vec!["disable-wait-for-index"],
        vec!["enable-reports"],
        vec!["disable-reports"],
        vec!["allow-subscriptions"],
        vec!["restrict-subscriptions"],
        vec!["queue-depths"],
        vec!["snapshot-rate", "5"],
        vec!["expiration-check", "10"],
    ];
    for (request, expected_raw) in requests.iter().zip(expected) {
        assert_eq!(raw(request), &expected_raw);
    }
}

#[tokio::test]
async fn qualifier_errors_are_returned_without_networking() {
    let engine = Engine::new(
        "127.0.0.1".into(),
        1,
        "owner".into(),
        "secret".into(),
        None,
        false,
    );
    let model_error = engine
        .policy_grant(
            "alice",
            PolicyCapability::ManageSchema,
            "catalog",
            None,
            &[],
            &[SemanticModel::BgeSmall],
        )
        .await
        .unwrap_err();
    assert!(
        model_error
            .message()
            .contains("models is only valid for provision-keyspace or manage-semantic")
    );

    let type_error = engine
        .policy_grant(
            "alice",
            PolicyCapability::ManageSnapshots,
            "catalog",
            None,
            &[PolicyKeyspaceType::Persistent],
            &[],
        )
        .await
        .unwrap_err();
    assert!(
        type_error
            .message()
            .contains("types is not valid for manage-snapshots")
    );
}

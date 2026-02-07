use super::utils::send_data;
use crate::{errors::MontycatClientError, request::structure::Req};
use serde::{Deserialize, Serialize};
use url::Url;

/// Valid permissions for granting or revoking access.
/// Read: Read-only access.
/// Write: Write-only access.
/// All: Full access (read and write).
///
/// Examples
///
/// ```rust
/// use montycat::engine::structure::ValidPermissions;
/// let read_permission = ValidPermissions::Read;
/// let write_permission = ValidPermissions::Write;
/// let all_permission = ValidPermissions::All;
/// ```
///
pub enum ValidPermissions {
    Read,
    Write,
    All,
}

impl ValidPermissions {
    pub fn as_str(&self) -> &str {
        match self {
            ValidPermissions::Read => "read",
            ValidPermissions::Write => "write",
            ValidPermissions::All => "all",
        }
    }
}

/// Represents the Montycat engine configuration and connection details.
///
/// # Fields
///
/// - `host`: The hostname or IP address of the Montycat server.
/// - `port`: The port number of the Montycat server.
/// - `username`: The username for authentication.
/// - `password`: The password for authentication.
/// - `store`: An optional store name to connect to.
/// - `use_tls`: A boolean indicating whether to use TLS for the connection.
///
/// # Examples
/// ```rust
/// use montycat::engine::structure::Engine;
/// let engine = Engine::new("localhost".into(), 21210, "user".into(), "pass".into(), Some("mystore".into()), false);
/// ```
///
/// # Errors
/// This struct does not return errors. However, ensure that the provided parameters are valid.
///
/// # Notes
///
/// If `use_tls` is set to true, the connection will be established using TLS encryption.
/// You have to enable the `tls` feature in Cargo.toml for TLS support.
///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Engine {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub store: Option<String>,
    pub use_tls: bool,
}

impl Engine {
    /// Creates a new Engine instance.
    ///
    /// # Arguments
    ///
    /// * `host` - The hostname or IP address of the Montycat server.
    /// * `port` - The port number of the Montycat server.
    /// * `username` - The username for authentication.
    /// * `password` - The password for authentication.
    /// * `store` - An optional store name to connect to.
    /// * `use_tls` - A boolean indicating whether to use TLS for the connection.
    ///
    /// # Returns
    ///
    /// * `Arc<Engine>` - An Arc-wrapped Engine instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let engine = Engine::new("localhost".into(), 21210, "user".into(), "pass".into(), Some("mystore".into()), false);
    /// ```
    ///
    /// # Errors
    ///
    /// This function does not return errors. However, ensure that the provided parameters are valid.
    ///
    pub fn new(
        host: String,
        port: u16,
        username: String,
        password: String,
        store: Option<String>,
        use_tls: bool,
    ) -> Self {
        Engine {
            host,
            port,
            username,
            password,
            store,
            use_tls,
        }
    }

    pub(crate) fn get_credentials(&self) -> Vec<String> {
        vec![self.username.clone(), self.password.clone()]
    }

    /// Enables TLS for the engine connection.
    ///
    /// This method sets the `use_tls` field to true, indicating that the connection
    /// should be established using TLS.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut engine = Engine::new("localhost".into(), 21210, "user".into(), "pass".into(), None, false);
    /// engine.enable_tls();
    /// ```
    ///
    /// # Notes
    ///
    /// If `use_tls` is set to true, the connection will be established using TLS encryption.
    /// You have to enable the `tls` feature in Cargo.toml for TLS support.
    /// If you are inside multithreaded context, ensure to use synchronization primitives
    /// as this method mutably borrows the Engine instance. Arc<Mutex<Engine>> or Arc<RwLock<Engine>> is recommended.
    ///
    pub fn enable_tls(&mut self) {
        self.use_tls = true;
    }

    /// Creates a new Engine instance from a Montycat URI.
    ///
    /// # Arguments
    /// * `uri` - A string slice that holds the Montycat URI in the format:
    ///   `montycat://username:password@host:port/store`
    ///
    /// # Returns
    ///
    /// * `Result<Arc<Engine>, MontycatClientError>` - An Arc-wrapped Engine instance or an error
    ///
    /// # Examples
    ///
    /// ```rust
    /// let engine = Engine::from_uri("montycat://username:password@localhost:21210/mystore").unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns MontycatClientError if the URI is invalid or missing required components
    ///
    pub fn from_uri(uri: &str) -> Result<Self, MontycatClientError> {
        if !uri.starts_with("montycat://") {
            return Err(MontycatClientError::ClientGenericError(
                "URI must start with montycat://".into(),
            ));
        }

        let parsed: Url =
            Url::parse(uri).map_err(|e| MontycatClientError::ClientEngineError(e.to_string()))?;

        let username: &str = parsed.username();
        if username.is_empty() {
            return Err(MontycatClientError::ClientGenericError(
                "Username must be provided".into(),
            ));
        }

        let password: &str = parsed.password().ok_or_else(|| {
            MontycatClientError::ClientGenericError("Password must be provided".into())
        })?;

        let host: &str = parsed.host_str().ok_or_else(|| {
            MontycatClientError::ClientGenericError("Host must be provided".into())
        })?;

        let port: u16 = parsed.port().ok_or_else(|| {
            MontycatClientError::ClientGenericError("Port must be provided".into())
        })?;

        let store: Option<String> = parsed.path().strip_prefix('/').and_then(|p| {
            if p.is_empty() {
                None
            } else {
                Some(p.to_string())
            }
        });

        let connection: Engine = Self::new(
            host.to_string(),
            port,
            username.to_string(),
            password.to_string(),
            store,
            false,
        );

        Ok(connection)
    }

    /// Creates a new store in the Montycat database.
    ///
    /// # Examples
    /// ```
    /// let engine = Engine::from_uri("montycat://username:password@localhost:21210/mystore").unwrap();
    /// let response = engine.create_store().await;
    /// ```
    /// # Errors
    /// Returns MontycatClientError if the store is not set or if there is a communication error.
    ///
    pub async fn create_store(&self) -> Result<Option<Vec<u8>>, MontycatClientError> {
        if let Some(store) = &self.store {
            let request: Req = Req::new_raw_command(
                vec!["create-store".into(), "store".into(), store.clone()],
                vec![self.username.clone(), self.password.clone()],
            );

            let response: Option<Vec<u8>> = send_data(
                &self.host,
                self.port,
                request.byte_down()?.as_slice(),
                None,
                None,
                self.use_tls,
            )
            .await?;

            Ok(response)
        } else {
            Err(MontycatClientError::ClientStoreNotSet)
        }
    }

    /// Removes the store from the Montycat database.
    ///
    /// # Examples
    /// ```
    /// let engine = Engine::from_uri("montycat://username:password@localhost:21210/mystore").unwrap();
    /// let response = engine.remove_store().await;
    ///
    /// ```
    /// # Returns
    /// * `Result<Option<Vec<u8>>, MontycatClientError>` - The response from the server or an error
    ///
    /// # Errors
    /// Returns MontycatClientError if the store is not set or if there is a communication error.
    ///
    pub async fn remove_store(&self) -> Result<Option<Vec<u8>>, MontycatClientError> {
        if let Some(store) = &self.store {
            let request: Req = Req::new_raw_command(
                vec!["remove-store".into(), "store".into(), store.clone()],
                vec![self.username.clone(), self.password.clone()],
            );

            let response: Option<Vec<u8>> = send_data(
                &self.host,
                self.port,
                request.byte_down()?.as_slice(),
                None,
                None,
                self.use_tls,
            )
            .await?;

            Ok(response)
        } else {
            Err(MontycatClientError::ClientStoreNotSet)
        }
    }

    /// Retrieves the available structures from the Montycat database.
    ///
    /// # Examples
    /// ```
    /// let engine = Engine::from_uri("montycat://username:password@localhost:21210/mystore").unwrap();
    /// let response = engine.get_structure_available().await;
    /// ```
    /// # Errors
    /// Returns MontycatClientError if the store is not set or if there is a communication error.
    ///
    pub async fn get_structure_available(&self) -> Result<Option<Vec<u8>>, MontycatClientError> {
        let command: Vec<String> = {
            if let Some(part) = self.store.as_ref().map(|s| vec!["store".into(), s.clone()]) {
                let mut cmd = vec!["get-structure-available".into()];
                cmd.extend(part);
                cmd
            } else {
                vec!["get-structure-available".into()]
            }
        };

        let request: Req =
            Req::new_raw_command(command, vec![self.username.clone(), self.password.clone()]);

        let response: Option<Vec<u8>> = send_data(
            &self.host,
            self.port,
            request.byte_down()?.as_slice(),
            None,
            None,
            self.use_tls,
        )
        .await?;

        Ok(response)
    }

    /// Lists all owners in the Montycat database.
    ///
    /// # Examples
    /// ```
    /// let engine = Engine::from_uri("montycat://username:password@localhost:21210/mystore").unwrap();
    /// let response = engine.list_owners().await;
    /// ```
    ///
    /// # Returns
    /// * `Result<Option<Vec<u8>>, MontycatClientError>` - The response
    ///
    /// # Errors
    /// Returns MontycatClientError if there is a communication error.
    ///
    pub async fn list_owners(&self) -> Result<Option<Vec<u8>>, MontycatClientError> {
        let request: Req = Req::new_raw_command(
            vec!["list-owners".into()],
            vec![self.username.clone(), self.password.clone()],
        );

        let response: Option<Vec<u8>> = send_data(
            &self.host,
            self.port,
            request.byte_down()?.as_slice(),
            None,
            None,
            self.use_tls,
        )
        .await?;

        Ok(response)
    }

    /// Creates a new owner in the Montycat database.
    ///
    /// # Arguments
    /// * `username` - The username of the new owner
    /// * `password` - The password of the new owner
    ///
    /// # Returns
    /// * `Result<Option<Vec<u8>>, MontycatClientError>` - The response
    ///
    /// # Examples
    /// ```
    /// let engine = Engine::from_uri("montycat://admin:adminpass@localhost:21210/mystore").unwrap();
    /// let response = engine.create_owner("new_owner", "owner_password").await;
    /// ```
    /// # Errors
    /// Returns MontycatClientError if there is a communication error.
    ///
    pub async fn create_owner(
        &self,
        username: &str,
        password: &str,
    ) -> Result<Option<Vec<u8>>, MontycatClientError> {
        let request: Req = Req::new_raw_command(
            vec![
                "create-owner".into(),
                "username".into(),
                username.into(),
                "password".into(),
                password.into(),
            ],
            vec![self.username.to_owned(), self.password.to_owned()],
        );

        let response: Option<Vec<u8>> = send_data(
            &self.host,
            self.port,
            request.byte_down()?.as_slice(),
            None,
            None,
            self.use_tls,
        )
        .await?;

        Ok(response)
    }

    /// Removes an owner from the Montycat database.
    ///
    /// # Arguments
    /// * `username` - The username of the owner to remove
    ///
    /// # Returns
    /// * `Result<Option<Vec<u8>>, MontycatClientError>` - The response
    ///
    /// # Examples
    ///
    /// ```
    /// let engine = Engine::from_uri("montycat://admin:adminpass@localhost:21210/mystore").unwrap();
    /// let response = engine.remove_owner("new_owner").await;
    /// ```
    ///
    /// # Errors
    /// Returns MontycatClientError if there is a communication error.
    ///
    pub async fn remove_owner(
        &self,
        username: &str,
    ) -> Result<Option<Vec<u8>>, MontycatClientError> {
        let request: Req = Req::new_raw_command(
            vec!["remove-owner".into(), "username".into(), username.into()],
            vec![self.username.to_owned(), self.password.to_owned()],
        );

        let response: Option<Vec<u8>> = send_data(
            &self.host,
            self.port,
            request.byte_down()?.as_slice(),
            None,
            None,
            self.use_tls,
        )
        .await?;

        Ok(response)
    }

    /// Grants permissions to an owner on a store and optionally specific keyspaces.
    ///
    /// # Arguments
    ///
    /// * `username` - The username of the owner to grant permissions to
    /// * `store` - The store to grant permissions on. If None, uses the store set in the engine.
    /// * `permission` - The permission to grant (Read, Write, All)
    /// * `keyspaces` - Optional vector of keyspace names to limit the permissions to
    ///
    /// # Returns
    ///
    /// * `Result<Option<Vec<u8>>, MontycatClientError>` - The response from the server or an error
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// let engine = Engine::from_uri("montycat://admin:adminpass@localhost:21210/mystore").unwrap();
    /// let response = engine.grant_to("new_owner", ValidPermissions::All, None, None).await;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns MontycatClientError if there is a communication error.
    ///
    pub async fn grant_to(
        &self,
        username: &str,
        permission: ValidPermissions,
        store: Option<&str>,
        keyspaces: Option<Vec<&str>>,
    ) -> Result<Option<Vec<u8>>, MontycatClientError> {
        let store: &str = {
            if let Some(s) = store {
                s
            } else {
                self.store
                    .as_deref()
                    .ok_or(MontycatClientError::ClientStoreNotSet)?
            }
        };

        let mut vec: Vec<String> = vec![
            "grant-to".into(),
            "owner".into(),
            username.into(),
            "permission".into(),
            permission.as_str().into(),
            "store".into(),
            store.into(),
        ];

        if let Some(ks_vec) = keyspaces
            && !ks_vec.is_empty()
        {
            vec.push("keyspaces".into());
            vec.push(ks_vec.join(","));
        }

        let request: Req = Req::new_raw_command(
            vec,
            vec![self.username.to_owned(), self.password.to_owned()],
        );

        let response: Option<Vec<u8>> = send_data(
            &self.host,
            self.port,
            request.byte_down()?.as_slice(),
            None,
            None,
            self.use_tls,
        )
        .await?;

        Ok(response)
    }

    /// Revokes permissions from an owner on a store and optionally specific keyspaces.
    ///
    /// # Arguments
    ///
    /// * `username` - The username of the owner to revoke permissions from
    /// * `store` - The store to revoke permissions on. If None, uses the store set in the engine.
    /// * `permission` - The permission to revoke (Read, Write, All)
    /// * `keyspaces` - Optional vector of keyspace names to limit the revocation
    ///
    /// # Returns
    ///
    /// * `Result<Option<Vec<u8>>, MontycatClientError>` - The response from the server or an error
    ///
    /// # Examples
    ///
    /// ```rust
    /// let engine = Engine::from_uri("montycat://admin:adminpass@localhost:21210/mystore").unwrap();
    /// let response = engine.revoke_from("new_owner", ValidPermissions::All, None, None).await;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns MontycatClientError if there is a communication error.
    ///
    pub async fn revoke_from(
        &self,
        username: &str,
        permission: ValidPermissions,
        store: Option<&str>,
        keyspaces: Option<Vec<&str>>,
    ) -> Result<Option<Vec<u8>>, MontycatClientError> {
        let store: &str = {
            if let Some(s) = store {
                s
            } else {
                self.store
                    .as_deref()
                    .ok_or(MontycatClientError::ClientStoreNotSet)?
            }
        };

        let mut vec: Vec<String> = vec![
            "revoke-from".into(),
            "owner".into(),
            username.into(),
            "permission".into(),
            permission.as_str().into(),
            "store".into(),
            store.into(),
        ];

        if let Some(ks_vec) = keyspaces
            && !ks_vec.is_empty()
        {
            vec.push("keyspaces".into());
            vec.push(ks_vec.join(","));
        }

        let request: Req = Req::new_raw_command(
            vec,
            vec![self.username.to_owned(), self.password.to_owned()],
        );

        let response: Option<Vec<u8>> = send_data(
            &self.host,
            self.port,
            request.byte_down()?.as_slice(),
            None,
            None,
            self.use_tls,
        )
        .await?;

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== ValidPermissions Tests =====

    #[test]
    fn test_valid_permissions_read() {
        let perm = ValidPermissions::Read;
        assert_eq!(perm.as_str(), "read");
    }

    #[test]
    fn test_valid_permissions_write() {
        let perm = ValidPermissions::Write;
        assert_eq!(perm.as_str(), "write");
    }

    #[test]
    fn test_valid_permissions_all() {
        let perm = ValidPermissions::All;
        assert_eq!(perm.as_str(), "all");
    }

    // ===== Engine Tests =====

    #[test]
    fn test_engine_new() {
        let engine = Engine::new(
            "localhost".to_string(),
            21210,
            "testuser".to_string(),
            "testpass".to_string(),
            Some("teststore".to_string()),
            false,
        );

        assert_eq!(engine.host, "localhost");
        assert_eq!(engine.port, 21210);
        assert_eq!(engine.username, "testuser");
        assert_eq!(engine.password, "testpass");
        assert_eq!(engine.store, Some("teststore".to_string()));
        assert!(!engine.use_tls);
    }

    #[test]
    fn test_engine_new_without_store() {
        let engine = Engine::new(
            "127.0.0.1".to_string(),
            8080,
            "user".to_string(),
            "pass".to_string(),
            None,
            true,
        );

        assert_eq!(engine.host, "127.0.0.1");
        assert_eq!(engine.port, 8080);
        assert_eq!(engine.store, None);
        assert!(engine.use_tls);
    }

    #[test]
    fn test_engine_from_uri_valid() {
        let uri = "montycat://username:password@localhost:21210/mystore";
        let engine = Engine::from_uri(uri).unwrap();

        assert_eq!(engine.host, "localhost");
        assert_eq!(engine.port, 21210);
        assert_eq!(engine.username, "username");
        assert_eq!(engine.password, "password");
        assert_eq!(engine.store, Some("mystore".to_string()));
        assert!(!engine.use_tls);
    }

    #[test]
    fn test_engine_from_uri_without_store() {
        let uri = "montycat://user:pass@127.0.0.1:8080";
        let engine = Engine::from_uri(uri).unwrap();

        assert_eq!(engine.host, "127.0.0.1");
        assert_eq!(engine.port, 8080);
        assert_eq!(engine.username, "user");
        assert_eq!(engine.password, "pass");
        assert_eq!(engine.store, None);
    }

    #[test]
    fn test_engine_from_uri_with_special_characters() {
        let uri = "montycat://user%40email:p%40ssw0rd@example.com:9999/my-store_123";
        let engine = Engine::from_uri(uri).unwrap();

        assert_eq!(engine.host, "example.com");
        assert_eq!(engine.port, 9999);
        // URL encoding is preserved in username/password from url crate
        assert_eq!(engine.username, "user%40email");
        assert_eq!(engine.password, "p%40ssw0rd");
        assert_eq!(engine.store, Some("my-store_123".to_string()));
    }

    #[test]
    fn test_engine_from_uri_invalid_scheme() {
        let uri = "http://username:password@localhost:21210/mystore";
        let result = Engine::from_uri(uri);
        assert!(result.is_err());
    }

    #[test]
    fn test_engine_from_uri_missing_username() {
        let uri = "montycat://:password@localhost:21210/mystore";
        let result = Engine::from_uri(uri);
        assert!(result.is_err());
    }

    #[test]
    fn test_engine_from_uri_missing_password() {
        let uri = "montycat://username@localhost:21210/mystore";
        let result = Engine::from_uri(uri);
        assert!(result.is_err());
    }

    #[test]
    fn test_engine_from_uri_missing_host() {
        let uri = "montycat://username:password@:21210/mystore";
        let result = Engine::from_uri(uri);
        assert!(result.is_err());
    }

    #[test]
    fn test_engine_from_uri_missing_port() {
        let uri = "montycat://username:password@localhost/mystore";
        let result = Engine::from_uri(uri);
        assert!(result.is_err());
    }

    #[test]
    fn test_engine_from_uri_invalid_port() {
        let uri = "montycat://username:password@localhost:invalid/mystore";
        let result = Engine::from_uri(uri);
        assert!(result.is_err());
    }

    #[test]
    fn test_engine_from_uri_malformed() {
        let uri = "not-a-valid-uri";
        let result = Engine::from_uri(uri);
        assert!(result.is_err());
    }

    #[test]
    fn test_engine_get_credentials() {
        let engine = Engine::new(
            "localhost".to_string(),
            21210,
            "myuser".to_string(),
            "mypass".to_string(),
            Some("mystore".to_string()),
            false,
        );

        let creds = engine.get_credentials();
        assert_eq!(creds.len(), 2);
        assert_eq!(creds[0], "myuser");
        assert_eq!(creds[1], "mypass");
    }

    #[test]
    fn test_engine_enable_tls() {
        let mut engine = Engine::new(
            "localhost".to_string(),
            21210,
            "user".to_string(),
            "pass".to_string(),
            None,
            false,
        );

        assert!(!engine.use_tls);
        engine.enable_tls();
        assert!(engine.use_tls);
    }

    #[test]
    fn test_engine_serialization() {
        let engine = Engine::new(
            "localhost".to_string(),
            21210,
            "user".to_string(),
            "pass".to_string(),
            Some("store".to_string()),
            true,
        );

        let serialized = serde_json::to_string(&engine).unwrap();
        let deserialized: Engine = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.host, "localhost");
        assert_eq!(deserialized.port, 21210);
        assert_eq!(deserialized.username, "user");
        assert_eq!(deserialized.password, "pass");
        assert_eq!(deserialized.store, Some("store".to_string()));
        assert!(deserialized.use_tls);
    }

    #[test]
    fn test_engine_clone() {
        let engine1 = Engine::new(
            "localhost".to_string(),
            21210,
            "user".to_string(),
            "pass".to_string(),
            Some("store".to_string()),
            false,
        );

        let engine2 = engine1.clone();

        assert_eq!(engine1.host, engine2.host);
        assert_eq!(engine1.port, engine2.port);
        assert_eq!(engine1.username, engine2.username);
        assert_eq!(engine1.password, engine2.password);
        assert_eq!(engine1.store, engine2.store);
        assert_eq!(engine1.use_tls, engine2.use_tls);
    }

    #[test]
    fn test_engine_from_uri_with_ipv6() {
        let uri = "montycat://user:pass@[::1]:21210/store";
        let engine = Engine::from_uri(uri).unwrap();

        assert_eq!(engine.host, "[::1]");
        assert_eq!(engine.port, 21210);
    }
}

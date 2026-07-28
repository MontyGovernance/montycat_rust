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
/// ```rust, ignore
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

/// Capabilities that can be granted through data-mesh governance policies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyCapability {
    ProvisionKeyspace,
    RemoveKeyspace,
    ManageSnapshots,
    ManageSemantic,
    ManageSchema,
    ManageAccess,
}
impl PolicyCapability {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ProvisionKeyspace => "provision-keyspace",
            Self::RemoveKeyspace => "remove-keyspace",
            Self::ManageSnapshots => "manage-snapshots",
            Self::ManageSemantic => "manage-semantic",
            Self::ManageSchema => "manage-schema",
            Self::ManageAccess => "manage-access",
        }
    }
}

/// Keyspace storage types addressable by governance policies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyKeyspaceType {
    InMemory,
    Persistent,
    Distributed,
}
impl PolicyKeyspaceType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InMemory => "inmemory",
            Self::Persistent => "persistent",
            Self::Distributed => "distributed",
        }
    }
}

/// Compiled embedding models supported by Montycat semantic search.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticModel {
    MiniLm,
    BgeSmall,
    BgeBase,
    E5Small,
}
impl SemanticModel {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MiniLm => "minilm",
            Self::BgeSmall => "bge-small",
            Self::BgeBase => "bge-base",
            Self::E5Small => "e5-small",
        }
    }
}

/// Serialization formats accepted by policy manifest commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyFormat {
    Json,
    Yaml,
    Yml,
}
impl PolicyFormat {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Yaml => "yaml",
            Self::Yml => "yml",
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
/// ```rust, ignore
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
    /// ```rust, ignore
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
    /// ```rust, ignore
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
    /// ```rust, ignore
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
    /// ```rust, ignore
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
    /// ```rust, ignore
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
    /// ```rust, ignore
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

    /// Enables semantic (vector similarity) search.
    ///
    /// Without `store`, this is DB-wide: it flips the whole database on, sets the
    /// default embedding model and field, and enrolls every existing keyspace that
    /// has no semantic config yet (each gets a background backfill so its existing
    /// items become searchable). The chosen model is downloaded on demand on first
    /// enable, so this call may take a while the first time.
    ///
    /// With `store`, it is scoped: only that store's un-enrolled keyspaces are
    /// enrolled and backfilled; the DB-wide switch and default model/field are left
    /// untouched. Use this to (re-)enable one store without re-embedding the entire
    /// database.
    ///
    /// # Arguments
    /// * `model` - The embedding model key to use by default. One of 'minilm',
    ///   'bge-small', 'bge-base', 'e5-small'. None uses the server default
    ///   ('bge-small').
    /// * `field` - The JSON field of each value to embed. None embeds the whole value.
    /// * `store` - Restrict enrollment/backfill to this store only. If the DB-wide
    ///   switch is off, a scoped enable enrolls but nothing embeds until a DB-wide
    ///   enable.
    ///
    /// # Examples
    /// ```rust, ignore
    /// let engine = Engine::from_uri("montycat://username:password@localhost:21210/mystore").unwrap();
    /// let response = engine.enable_semantic_search(Some(SemanticModel::BgeSmall), None, None).await;
    /// ```
    ///
    /// # Errors
    /// Returns MontycatClientError if there is a communication error.
    ///
    pub async fn enable_semantic_search(
        &self,
        model: Option<SemanticModel>,
        field: Option<&str>,
        store: Option<&str>,
    ) -> Result<Option<Vec<u8>>, MontycatClientError> {
        let mut command: Vec<String> = vec!["enable-semantic-search".into()];
        if let Some(model) = model {
            command.push("model".into());
            command.push(model.as_str().into());
        }
        if let Some(field) = field {
            command.push("field".into());
            command.push(field.into());
        }
        if let Some(store) = store {
            command.push("store".into());
            command.push(store.into());
        }

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

    /// Disables semantic search.
    ///
    /// Without `store`, this is DB-wide: embedding and semantic queries stop across
    /// the whole database; stored vectors are kept by default so re-enabling
    /// resumes without a full re-embed.
    ///
    /// With `store`, it is scoped: only that store's keyspaces are unenrolled
    /// (their configs and resident graphs dropped); the DB-wide switch and all
    /// other stores are left untouched. This is the surgical way to reset one
    /// store's semantic state instead of nuking and re-backfilling the whole
    /// database.
    ///
    /// # Arguments
    /// * `drop_vectors` - If true, also clear stored vectors — every keyspace's
    ///   DB-wide, or the scoped store's when `store` is set. Required before
    ///   switching to a different embedding model.
    /// * `store` - Restrict the disable to this store only. None disables DB-wide.
    ///
    /// # Examples
    /// ```rust, ignore
    /// let engine = Engine::from_uri("montycat://username:password@localhost:21210/mystore").unwrap();
    /// let response = engine.disable_semantic_search(false, None).await;
    /// ```
    ///
    /// # Errors
    /// Returns MontycatClientError if there is a communication error.
    ///
    pub async fn disable_semantic_search(
        &self,
        drop_vectors: bool,
        store: Option<&str>,
    ) -> Result<Option<Vec<u8>>, MontycatClientError> {
        let mut command: Vec<String> = vec!["disable-semantic-search".into()];
        if drop_vectors {
            command.push("drop-vectors".into());
        }
        if let Some(store) = store {
            command.push("store".into());
            command.push(store.into());
        }

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

    /// Enables the DB-wide "wait for index" default.
    ///
    /// Writes block until their secondary indexes are updated before returning,
    /// so a write is immediately visible to index-backed reads (e.g.
    /// `lookup_values_where`) at the cost of higher write latency. Requires
    /// superowner credentials.
    ///
    /// # Examples
    /// ```rust, ignore
    /// let engine = Engine::from_uri("montycat://admin:adminpass@localhost:21210/mystore").unwrap();
    /// let response = engine.enable_wait_for_index().await;
    /// ```
    ///
    /// # Errors
    /// Returns MontycatClientError if there is a communication error.
    ///
    pub async fn enable_wait_for_index(&self) -> Result<Option<Vec<u8>>, MontycatClientError> {
        let request: Req = Req::new_raw_command(
            vec!["enable-wait-for-index".into()],
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

    /// Disables the DB-wide "wait for index" default.
    ///
    /// Writes return as soon as the data is committed and indexing happens
    /// asynchronously in the background (lower write latency; index-backed
    /// reads may briefly lag). This is the default behavior. Requires
    /// superowner credentials.
    ///
    /// # Examples
    /// ```rust, ignore
    /// let engine = Engine::from_uri("montycat://admin:adminpass@localhost:21210/mystore").unwrap();
    /// let response = engine.disable_wait_for_index().await;
    /// ```
    ///
    /// # Errors
    /// Returns MontycatClientError if there is a communication error.
    ///
    pub async fn disable_wait_for_index(&self) -> Result<Option<Vec<u8>>, MontycatClientError> {
        let request: Req = Req::new_raw_command(
            vec!["disable-wait-for-index".into()],
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

    /// Internal helper: send `command` (the full raw token list) authenticated
    /// with the engine's credentials.
    async fn admin_command(
        &self,
        command: Vec<String>,
    ) -> Result<Option<Vec<u8>>, MontycatClientError> {
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

    /// Enables server-side operation reporting (logging). Requires superowner credentials.
    ///
    /// # Errors
    /// Returns MontycatClientError if there is a communication error.
    pub async fn enable_reports(&self) -> Result<Option<Vec<u8>>, MontycatClientError> {
        self.admin_command(vec!["enable-reports".into()]).await
    }

    /// Disables server-side operation reporting (logging). Requires superowner credentials.
    ///
    /// # Errors
    /// Returns MontycatClientError if there is a communication error.
    pub async fn disable_reports(&self) -> Result<Option<Vec<u8>>, MontycatClientError> {
        self.admin_command(vec!["disable-reports".into()]).await
    }

    /// Allows clients to open keyspace subscriptions DB-wide. Requires superowner credentials.
    ///
    /// # Errors
    /// Returns MontycatClientError if there is a communication error.
    pub async fn allow_subscriptions(&self) -> Result<Option<Vec<u8>>, MontycatClientError> {
        self.admin_command(vec!["allow-subscriptions".into()]).await
    }

    /// Restricts (disallows) keyspace subscriptions DB-wide. Requires superowner credentials.
    ///
    /// # Errors
    /// Returns MontycatClientError if there is a communication error.
    pub async fn restrict_subscriptions(&self) -> Result<Option<Vec<u8>>, MontycatClientError> {
        self.admin_command(vec!["restrict-subscriptions".into()])
            .await
    }

    /// Samples the current depth of every background task queue (index, timer,
    /// counting) — an observability probe for whether the background runners are
    /// keeping up with the write rate. Requires superowner credentials.
    ///
    /// The response payload maps `"index" | "timer" | "counting"` to per-queue
    /// depth maps.
    ///
    /// # Errors
    /// Returns MontycatClientError if there is a communication error.
    pub async fn queue_depths(&self) -> Result<Option<Vec<u8>>, MontycatClientError> {
        self.admin_command(vec!["queue-depths".into()]).await
    }

    /// Sets the server-wide snapshot rate. Requires superowner credentials.
    ///
    /// # Arguments
    /// * `rate` - The snapshot rate value (server-defined units).
    ///
    /// # Errors
    /// Returns MontycatClientError if there is a communication error.
    pub async fn set_snapshot_rate(
        &self,
        rate: u64,
    ) -> Result<Option<Vec<u8>>, MontycatClientError> {
        self.admin_command(vec!["snapshot-rate".into(), rate.to_string()])
            .await
    }

    /// Sets how often the server scans for expired keys. Requires superowner credentials.
    ///
    /// # Arguments
    /// * `rate` - The check period in whole seconds (e.g. `rate = 10` → a scan every
    ///   10 seconds). Stored as-is, like the snapshot rate. Defaults to 1 second.
    ///
    /// # Errors
    /// Returns MontycatClientError if there is a communication error.
    pub async fn set_expiration_check_rate(
        &self,
        rate: u64,
    ) -> Result<Option<Vec<u8>>, MontycatClientError> {
        self.admin_command(vec!["expiration-check".into(), rate.to_string()])
            .await
    }

    /// Lists all owners in the Montycat database.
    ///
    /// # Examples
    /// ```rust, ignore
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
    /// ```rust, ignore
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
    /// ```rust, ignore
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
    /// ```rust, ignore
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
    /// ```rust, ignore
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

    async fn execute_governance_command(
        &self,
        command: Vec<String>,
    ) -> Result<Option<Vec<u8>>, MontycatClientError> {
        let request =
            Req::new_raw_command(command, vec![self.username.clone(), self.password.clone()]);
        send_data(
            &self.host,
            self.port,
            request.byte_down()?.as_slice(),
            None,
            None,
            self.use_tls,
        )
        .await
    }

    /// Enable semantic management for one keyspace without changing the
    /// existing DB/store-scoped API.
    pub async fn enable_semantic_search_for_keyspace(
        &self,
        store: &str,
        keyspace: &str,
        model: Option<SemanticModel>,
        field: Option<&str>,
    ) -> Result<Option<Vec<u8>>, MontycatClientError> {
        let mut command = vec!["enable-semantic-search".into()];
        if let Some(model) = model {
            command.extend(["model".into(), model.as_str().into()]);
        }
        if let Some(field) = field {
            command.extend(["field".into(), field.into()]);
        }
        command.extend([
            "store".into(),
            store.into(),
            "keyspace".into(),
            keyspace.into(),
        ]);
        self.execute_governance_command(command).await
    }

    /// Disable semantic management for one keyspace.
    pub async fn disable_semantic_search_for_keyspace(
        &self,
        store: &str,
        keyspace: &str,
        drop_vectors: bool,
    ) -> Result<Option<Vec<u8>>, MontycatClientError> {
        let mut command = vec!["disable-semantic-search".into()];
        if drop_vectors {
            command.push("drop-vectors".into());
        }
        command.extend([
            "store".into(),
            store.into(),
            "keyspace".into(),
            keyspace.into(),
        ]);
        self.execute_governance_command(command).await
    }

    pub async fn policy_view(
        &self,
        owner: Option<&str>,
        store: Option<&str>,
    ) -> Result<Option<Vec<u8>>, MontycatClientError> {
        let mut command = vec!["policy-view".into()];
        if let Some(owner) = owner {
            command.extend(["owner".into(), owner.into()]);
        }
        if let Some(store) = store {
            command.extend(["store".into(), store.into()]);
        }
        self.execute_governance_command(command).await
    }

    pub async fn policy_history(
        &self,
        owner: Option<&str>,
        store: Option<&str>,
        keyspace: Option<&str>,
    ) -> Result<Option<Vec<u8>>, MontycatClientError> {
        let mut command = vec!["policy-history".into()];
        if let Some(owner) = owner {
            command.extend(["owner".into(), owner.into()]);
        }
        if let Some(store) = store {
            command.extend(["store".into(), store.into()]);
        }
        if let Some(keyspace) = keyspace {
            command.extend(["keyspace".into(), keyspace.into()]);
        }
        self.execute_governance_command(command).await
    }

    pub async fn policy_explain(
        &self,
        capability: PolicyCapability,
        store: &str,
        owner: Option<&str>,
        keyspace: Option<&str>,
        keyspace_type: Option<PolicyKeyspaceType>,
        model: Option<SemanticModel>,
    ) -> Result<Option<Vec<u8>>, MontycatClientError> {
        Self::validate_policy_qualifiers(capability, keyspace_type.is_some(), model.is_some())?;
        let mut command = vec![
            "policy-explain".into(),
            "capability".into(),
            capability.as_str().into(),
            "store".into(),
            store.into(),
        ];
        if let Some(owner) = owner {
            command.extend(["owner".into(), owner.into()]);
        }
        if capability != PolicyCapability::ProvisionKeyspace
            && let Some(keyspace) = keyspace
        {
            command.extend(["keyspace".into(), keyspace.into()]);
        }
        if let Some(kind) = keyspace_type {
            command.extend(["type".into(), kind.as_str().into()]);
        }
        if let Some(model) = model {
            command.extend(["model".into(), model.as_str().into()]);
        }
        self.execute_governance_command(command).await
    }

    #[allow(clippy::too_many_arguments)]
    async fn policy_mutation(
        &self,
        operation: &str,
        owner: &str,
        capability: PolicyCapability,
        store: &str,
        keyspace: Option<&str>,
        types: &[PolicyKeyspaceType],
        models: &[SemanticModel],
    ) -> Result<Option<Vec<u8>>, MontycatClientError> {
        Self::validate_policy_qualifiers(capability, !types.is_empty(), !models.is_empty())?;
        let mut command = vec![
            operation.into(),
            "owner".into(),
            owner.into(),
            "capability".into(),
            capability.as_str().into(),
            "store".into(),
            store.into(),
        ];
        if capability != PolicyCapability::ProvisionKeyspace
            && let Some(keyspace) = keyspace
        {
            command.extend(["keyspace".into(), keyspace.into()]);
        }
        if !types.is_empty() {
            command.push("types".into());
            command.extend(types.iter().map(|value| value.as_str().into()));
        }
        if !models.is_empty() {
            command.push("models".into());
            command.extend(models.iter().map(|value| value.as_str().into()));
        }
        self.execute_governance_command(command).await
    }

    fn validate_policy_qualifiers(
        capability: PolicyCapability,
        has_types: bool,
        has_models: bool,
    ) -> Result<(), MontycatClientError> {
        if has_types && capability == PolicyCapability::ManageSnapshots {
            return Err(MontycatClientError::ClientGenericError(
                "types is not valid for manage-snapshots policies; snapshots are always in-memory"
                    .into(),
            ));
        }
        if has_models
            && capability != PolicyCapability::ProvisionKeyspace
            && capability != PolicyCapability::ManageSemantic
        {
            return Err(MontycatClientError::ClientGenericError(
                "models is only valid for provision-keyspace or manage-semantic policies".into(),
            ));
        }
        Ok(())
    }

    pub async fn policy_grant(
        &self,
        owner: &str,
        capability: PolicyCapability,
        store: &str,
        keyspace: Option<&str>,
        types: &[PolicyKeyspaceType],
        models: &[SemanticModel],
    ) -> Result<Option<Vec<u8>>, MontycatClientError> {
        self.policy_mutation(
            "policy-grant",
            owner,
            capability,
            store,
            keyspace,
            types,
            models,
        )
        .await
    }
    pub async fn policy_revoke(
        &self,
        owner: &str,
        capability: PolicyCapability,
        store: &str,
        keyspace: Option<&str>,
        types: &[PolicyKeyspaceType],
        models: &[SemanticModel],
    ) -> Result<Option<Vec<u8>>, MontycatClientError> {
        self.policy_mutation(
            "policy-revoke",
            owner,
            capability,
            store,
            keyspace,
            types,
            models,
        )
        .await
    }
    pub async fn policy_deny(
        &self,
        owner: &str,
        capability: PolicyCapability,
        store: &str,
        keyspace: Option<&str>,
        types: &[PolicyKeyspaceType],
        models: &[SemanticModel],
    ) -> Result<Option<Vec<u8>>, MontycatClientError> {
        self.policy_mutation(
            "policy-deny",
            owner,
            capability,
            store,
            keyspace,
            types,
            models,
        )
        .await
    }
    pub async fn policy_remove_denial(
        &self,
        owner: &str,
        capability: PolicyCapability,
        store: &str,
        keyspace: Option<&str>,
        types: &[PolicyKeyspaceType],
        models: &[SemanticModel],
    ) -> Result<Option<Vec<u8>>, MontycatClientError> {
        self.policy_mutation(
            "policy-remove-denial",
            owner,
            capability,
            store,
            keyspace,
            types,
            models,
        )
        .await
    }
    pub async fn policy_preview_grant(
        &self,
        owner: &str,
        capability: PolicyCapability,
        store: &str,
        keyspace: Option<&str>,
        types: &[PolicyKeyspaceType],
        models: &[SemanticModel],
    ) -> Result<Option<Vec<u8>>, MontycatClientError> {
        self.policy_mutation(
            "policy-preview-grant",
            owner,
            capability,
            store,
            keyspace,
            types,
            models,
        )
        .await
    }
    pub async fn policy_preview_revoke(
        &self,
        owner: &str,
        capability: PolicyCapability,
        store: &str,
        keyspace: Option<&str>,
        types: &[PolicyKeyspaceType],
        models: &[SemanticModel],
    ) -> Result<Option<Vec<u8>>, MontycatClientError> {
        self.policy_mutation(
            "policy-preview-revoke",
            owner,
            capability,
            store,
            keyspace,
            types,
            models,
        )
        .await
    }

    async fn policy_manifest(
        &self,
        operation: &str,
        document: &str,
        format: PolicyFormat,
    ) -> Result<Option<Vec<u8>>, MontycatClientError> {
        self.execute_governance_command(vec![
            operation.into(),
            "format".into(),
            format.as_str().into(),
            "document".into(),
            document.into(),
        ])
        .await
    }
    pub async fn policy_validate(
        &self,
        document: &str,
        format: PolicyFormat,
    ) -> Result<Option<Vec<u8>>, MontycatClientError> {
        self.policy_manifest("policy-validate", document, format)
            .await
    }
    pub async fn policy_plan(
        &self,
        document: &str,
        format: PolicyFormat,
    ) -> Result<Option<Vec<u8>>, MontycatClientError> {
        self.policy_manifest("policy-plan", document, format).await
    }
    pub async fn policy_apply(
        &self,
        document: &str,
        format: PolicyFormat,
    ) -> Result<Option<Vec<u8>>, MontycatClientError> {
        self.policy_manifest("policy-apply", document, format).await
    }
    pub async fn policy_export(
        &self,
        format: PolicyFormat,
    ) -> Result<Option<Vec<u8>>, MontycatClientError> {
        self.execute_governance_command(vec![
            "policy-export".into(),
            "format".into(),
            format.as_str().into(),
        ])
        .await
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
    fn test_policy_qualifiers_match_their_capabilities() {
        assert!(
            Engine::validate_policy_qualifiers(PolicyCapability::ProvisionKeyspace, true, false,)
                .is_ok()
        );
        assert!(
            Engine::validate_policy_qualifiers(PolicyCapability::ManageSemantic, false, true,)
                .is_ok()
        );
        assert!(
            Engine::validate_policy_qualifiers(PolicyCapability::ProvisionKeyspace, false, true,)
                .is_ok()
        );
        assert!(
            Engine::validate_policy_qualifiers(PolicyCapability::ManageSchema, true, false,)
                .is_ok()
        );

        let model_error =
            Engine::validate_policy_qualifiers(PolicyCapability::ManageSchema, false, true)
                .unwrap_err();
        assert_eq!(
            model_error.message(),
            "models is only valid for provision-keyspace or manage-semantic policies"
        );

        let type_error =
            Engine::validate_policy_qualifiers(PolicyCapability::ManageSnapshots, true, false)
                .unwrap_err();
        assert_eq!(
            type_error.message(),
            "types is not valid for manage-snapshots policies; snapshots are always in-memory"
        );
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

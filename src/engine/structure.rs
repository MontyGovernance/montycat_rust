use serde::{Deserialize, Serialize};
use url::Url;
use crate::{errors::MontycatClientError, request::structure::Req};
use super::utils::send_data;
use std::sync::Arc;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Engine {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub store: Option<String>,
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
    ///
    /// # Returns
    ///
    /// * `Arc<Engine>` - An Arc-wrapped Engine instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let engine = Engine::new("localhost".into(), 21210, "user".into(), "pass".into(), Some("mystore".into()));
    /// ```
    ///
    /// # Errors
    ///
    /// This function does not return errors. However, ensure that the provided parameters are valid.
    ///
    pub fn new(host: String, port: u16, username: String, password: String, store: Option<String>) -> Arc<Self> {
        Engine {
            host,
            port,
            username,
            password,
            store,
        }.into()
    }

    pub fn get_credentials(&self) -> Vec<String> {
        vec![self.username.clone(), self.password.clone()]
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
    pub fn from_uri(uri: &str) -> Result<Arc<Self>, MontycatClientError> {

        if !uri.starts_with("montycat://") {
            return Err(MontycatClientError::GenericError("URI must start with montycat://".into()));
        }

        let parsed: Url = Url::parse(uri).map_err(|e| MontycatClientError::EngineError(e.to_string()))?;

        let username: &str = parsed.username();
        if username.is_empty() {
            return Err(MontycatClientError::GenericError("Username must be provided".into()));
        }

        let password: &str = parsed.password().ok_or_else(|| {
            MontycatClientError::GenericError("Password must be provided".into())
        })?;

        let host: &str = parsed.host_str()
            .ok_or_else(|| MontycatClientError::GenericError("Host must be provided".into()))?;

        let port: u16 = parsed.port()
            .ok_or_else(|| MontycatClientError::GenericError("Port must be provided".into()))?;

        let store: Option<String> = parsed.path().strip_prefix('/').and_then(|p| {
            if p.is_empty() { None } else { Some(p.to_string()) }
        });

        let connection: Arc<Engine> = Self::new(
            host.to_string(),
            port,
            username.to_string(),
            password.to_string(),
            store,
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
                vec![self.username.clone(), self.password.clone()]
            );

            let response: Option<Vec<u8>> = send_data(
                &self.host,
                self.port,
                request.byte_down()?.as_slice(),
                None,
                None,
            ).await?;

            Ok(response)

        } else {
            Err(MontycatClientError::StoreNotSet)
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
                vec![self.username.clone(), self.password.clone()]
            );

            let response: Option<Vec<u8>> = send_data(
                &self.host,
                self.port,
                request.byte_down()?.as_slice(),
                None,
                None,
            ).await?;

            Ok(response)

        } else {
            Err(MontycatClientError::StoreNotSet)
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

        let request: Req = Req::new_raw_command(
            vec!["get-structure-available".into()],
            vec![self.username.clone(), self.password.clone()]
        );

        let response: Option<Vec<u8>> = send_data(
            &self.host,
            self.port,
            request.byte_down()?.as_slice(),
            None,
            None,
        ).await?;

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
            vec![self.username.clone(), self.password.clone()]
        );

        let response: Option<Vec<u8>> = send_data(
            &self.host,
            self.port,
            request.byte_down()?.as_slice(),
            None,
            None,
        ).await?;

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
    pub async fn create_owner(&self, username: &str, password: &str) -> Result<Option<Vec<u8>>, MontycatClientError> {

        let request: Req = Req::new_raw_command(
            vec!["create-owner".into(), "username".into(), username.into(), "password".into(), password.into()],
            vec![self.username.to_owned(), self.password.to_owned()]
        );

        let response: Option<Vec<u8>> = send_data(
            &self.host,
            self.port,
            request.byte_down()?.as_slice(),
            None,
            None,
        ).await?;

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
    pub async fn remove_owner(&self, username: &str) -> Result<Option<Vec<u8>>, MontycatClientError> {

        let request: Req = Req::new_raw_command(
            vec!["remove-owner".into(), "username".into(), username.into()],
            vec![self.username.to_owned(), self.password.to_owned()]
        );

        let response: Option<Vec<u8>> = send_data(
            &self.host,
            self.port,
            request.byte_down()?.as_slice(),
            None,
            None,
        ).await?;

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
                self.store.as_deref().ok_or(MontycatClientError::StoreNotSet)?
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

        if let Some(ks_vec) = keyspaces {
            if !ks_vec.is_empty() {
                vec.push("keyspaces".into());
                vec.push(ks_vec.join(",").into());
            }
        }

        let request: Req = Req::new_raw_command(
            vec,
            vec![self.username.to_owned(), self.password.to_owned()]
        );

        let response: Option<Vec<u8>> = send_data(
            &self.host,
            self.port,
            request.byte_down()?.as_slice(),
            None,
            None,
        ).await?;

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
    /// ```rust,no_run
    /// let engine = Engine::from_uri("montycat://admin:adminpass@localhost:21210/mystore").unwrap();
    /// let response = engine.revoke_from("new_owner", ValidPermissions::All, None, None).await;
    /// ```
    ///
    /// # Errors
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
                self.store.as_deref().ok_or(MontycatClientError::StoreNotSet)?
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

        if let Some(ks_vec) = keyspaces {
            if !ks_vec.is_empty() {
                vec.push("keyspaces".into());
                vec.push(ks_vec.join(",").into());
            }
        }

        let request: Req = Req::new_raw_command(
            vec,
            vec![self.username.to_owned(), self.password.to_owned()]
        );

        let response: Option<Vec<u8>> = send_data(
            &self.host,
            self.port,
            request.byte_down()?.as_slice(),
            None,
            None,
        ).await?;

        Ok(response)

    }

}
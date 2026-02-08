use super::super::pubtrait::Keyspace;
use crate::engine::structure::Engine;
use crate::engine::utils::send_data;
use crate::errors::MontycatClientError;
use crate::request::store_request::structure::StoreRequestClient;
use crate::request::structure::Req;
use crate::request::utis::functions::{convert_custom_key, is_custom_type};
use crate::tools::functions::{process_bulk_values, process_json_value, process_value};
use crate::traits::RuntimeSchema;
use serde::Serialize;
use std::any::type_name;

/// Represents an in-memory keyspace in the Montycat database.
///
/// # Fields
/// - `name`: The name of the keyspace.
/// - `persistent`: A boolean indicating if the keyspace is persistent.
/// - `distributed`: A boolean indicating if the keyspace is distributed.
/// - `engine`: The Montycat engine instance associated with the keyspace.
///
#[derive(Debug, Clone)]
pub struct InMemoryKeyspace {
    name: String,
    persistent: bool,
    distributed: bool,
    engine: Engine,
}

impl Keyspace for InMemoryKeyspace {
    /// Retrieves the associated Montycat engine.
    ///
    /// # Returns
    /// - `Engine`: The Montycat engine instance.
    ///
    fn get_engine(&self) -> Engine {
        self.engine.clone()
    }
    /// Retrieves the name of the keyspace.
    ///
    /// # Returns
    /// - `&str`: The name of the keyspace.
    ///
    fn get_name(&self) -> &str {
        &self.name
    }

    /// Checks if the keyspace is persistent.
    ///
    /// # Returns
    /// - `bool`: True if the keyspace is persistent, false otherwise.
    ///
    fn get_persistent(&self) -> bool {
        self.persistent
    }

    /// Checks if the keyspace is distributed.
    ///
    /// # Returns
    /// - `bool`: True if the keyspace is distributed, false otherwise.
    ///
    /// # Notes
    /// In Development
    ///
    fn get_distributed(&self) -> bool {
        self.distributed
    }
}

impl InMemoryKeyspace {
    /// Creates a new instance of `InMemoryKeyspace`.
    ///
    /// # Arguments
    /// - `name: &str`: The name of the keyspace.
    /// - `engine: &Engine`: A reference to the Montycat engine.
    ///
    /// # Returns
    /// - `InMemoryKeyspace`: A new instance of `InMemoryKeyspace`.
    ///
    /// # Examples
    ///
    /// ```rust, ignore
    /// let keyspace: InMemoryKeyspace = InMemoryKeyspace::new("test_keyspace", &engine);
    /// ```
    ///
    pub fn new(name: &str, engine: &Engine) -> Self {
        Self {
            name: name.to_owned(),
            persistent: false,
            distributed: false,
            engine: engine.clone(),
        }
    }

    /// Creates a new keyspace in the Montycat database.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Vec<u8>>, MontycatClientError>` - The response from the server or an error.
    ///
    /// # Examples
    ///
    /// ```rust, ignore
    /// let res: Result<Option<Vec<u8>>, MontycatClientError> = keyspace.create_keyspace().await;
    /// ```
    ///
    /// # Errors
    ///
    /// * `MontycatClientError::ClientStoreNotSet` - If the store is not set in the engine.
    /// * `MontycatClientError::ClientEngineError` - If there is an error with the engine.
    /// * `MontycatClientError::ClientValueParsingError` - If there is an error parsing the response.
    ///
    pub async fn create_keyspace(&self) -> Result<Option<Vec<u8>>, MontycatClientError> {
        let engine: Engine = self.get_engine();
        let name: &str = self.get_name();
        let persistent: bool = self.get_persistent();
        let distributed: bool = self.get_distributed();
        let store: String = engine
            .store
            .clone()
            .ok_or(MontycatClientError::ClientStoreNotSet)?;
        let use_tls: bool = engine.use_tls;

        let vec: Vec<String> = vec![
            "create-keyspace".into(),
            "store".into(),
            store,
            "keyspace".into(),
            name.to_owned(),
            "persistent".into(),
            if persistent { "y".into() } else { "n".into() },
            "distributed".into(),
            if distributed { "y".into() } else { "n".into() },
        ];

        let credentials: Vec<String> = engine.get_credentials();
        let query: Req = Req::new_raw_command(vec, credentials);
        let bytes: Vec<u8> = query.byte_down()?;
        let response: Option<Vec<u8>> = send_data(
            &engine.host,
            engine.port,
            bytes.as_slice(),
            None,
            None,
            use_tls,
        )
        .await?;

        Ok(response)
    }

    /// Inserts a value into the keyspace.
    ///
    /// # Arguments
    ///
    /// * `&self` - The keyspace instance.
    /// * `value` - The value to insert. Must implement `Serialize` and `MontycatSchema`.
    /// * `expire_sec` - Optional expiration time in seconds.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Vec<u8>>, MontycatClientError>` - The response from the server or an error.
    ///
    /// # Examples
    ///
    /// ```rust, ignore
    /// let value = YourType { /* fields */ };
    /// let res: Result<Option<Vec<u8>>, MontycatClientError> = keyspace.insert_value(value, Some(3600)).await;
    /// let parsed = MontycatResponse::<YourType>::parse_response(res);
    /// ```
    ///
    /// # Errors
    ///
    /// * `MontycatClientError::ClientStoreNotSet` - If the store is not set in the engine.
    /// * `MontycatClientError::ClientEngineError` - If there is an error with the engine.
    /// * `MontycatClientError::ClientValueParsingError` - If there is an error parsing the response.
    ///
    pub async fn insert_value<T>(
        &self,
        custom_key: Option<String>,
        value: T,
        expire_sec: Option<usize>,
    ) -> Result<Option<Vec<u8>>, MontycatClientError>
    where
        T: Serialize + RuntimeSchema + Send + 'static,
    {
        let mut key: Option<String> = None;
        let engine: Engine = self.get_engine();
        let name: &str = self.get_name();
        let persistent: bool = self.get_persistent();
        let distributed: bool = self.get_distributed();
        let store: String = engine
            .store
            .clone()
            .ok_or(MontycatClientError::ClientStoreNotSet)?;
        let use_tls: bool = engine.use_tls;
        let mut schema: Option<String> = None;
        let value_to_send: String = process_value(value)?;
        let type_name_retrieved: &str = type_name::<T>();

        if let Some(custom_type_name) = is_custom_type(type_name_retrieved) {
            schema = Some(custom_type_name.to_owned());
        };

        if let Some(custom_key_str) = &custom_key {
            key = Some(convert_custom_key(custom_key_str));
        }

        let command: String = if key.is_none() {
            "insert_value".to_string()
        } else {
            "insert_custom_key_value".to_string()
        };

        let new_store_request: StoreRequestClient = StoreRequestClient {
            schema,
            username: engine.username.clone(),
            password: engine.password.clone(),
            keyspace: name.to_owned(),
            store,
            persistent,
            distributed,
            value: value_to_send,
            command,
            expire: expire_sec.map(|sec| sec as u64).unwrap_or(0),
            key,
            ..Default::default()
        };

        let bytes: Vec<u8> = Req::new_store_command(new_store_request).byte_down()?;
        let response: Option<Vec<u8>> = send_data(
            &engine.host,
            engine.port,
            bytes.as_slice(),
            None,
            None,
            use_tls,
        )
        .await?;

        Ok(response)
    }

    /// Inserts a custom key into the keyspace.
    ///
    /// # Arguments
    ///
    /// * `custom_key` - The custom key to be inserted into the keyspace.
    /// * `expire_sec` - Optional expiration time in seconds.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Vec<u8>>, MontycatClientError>` - The response from the server or an error.
    ///
    /// # Examples
    ///
    /// ```rust, ignore
    /// let res: Result<Option<Vec<u8>>, MontycatClientError> = keyspace.insert_custom_key(Some("my_custom_key".into()), Some(3600)).await;
    /// ```
    ///
    /// # Errors
    ///
    /// * `MontycatClientError::ClientStoreNotSet` - If the store is not set in the engine.
    /// * `MontycatClientError::ClientEngineError` - If there is an error with the engine.
    /// * `MontycatClientError::ClientValueParsingError` - If there is an error parsing the response.
    ///
    pub async fn insert_custom_key(
        &self,
        custom_key: String,
        expire_sec: Option<usize>,
    ) -> Result<Option<Vec<u8>>, MontycatClientError> {
        let engine: Engine = self.get_engine();
        let name: &str = self.get_name();
        let persistent: bool = self.get_persistent();
        let distributed: bool = self.get_distributed();
        let store: String = engine
            .store
            .clone()
            .ok_or(MontycatClientError::ClientStoreNotSet)?;
        let use_tls: bool = engine.use_tls;

        let key: String = convert_custom_key(&custom_key);

        let command: String = "insert_custom_key".to_string();

        let new_store_request: StoreRequestClient = StoreRequestClient {
            username: engine.username.clone(),
            password: engine.password.clone(),
            keyspace: name.to_owned(),
            store,
            persistent,
            distributed,
            value: String::new(),
            command,
            expire: expire_sec.map(|sec| sec as u64).unwrap_or(0),
            key: Some(key),
            ..Default::default()
        };

        let bytes: Vec<u8> = Req::new_store_command(new_store_request).byte_down()?;
        let response: Option<Vec<u8>> = send_data(
            &engine.host,
            engine.port,
            bytes.as_slice(),
            None,
            None,
            use_tls,
        )
        .await?;

        Ok(response)
    }

    /// Inserts a simple value (without schema) into the keyspace.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to insert. Must implement `Serialize`.
    /// * `expire_sec` - Optional expiration time in seconds.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Vec<u8>>, MontycatClientError>` - The response from the server or an error.
    ///
    /// # Examples
    ///
    /// ```rust, ignore
    /// let value = vec!["simple_value1", "simple_value2"];
    ///
    /// let res: Result<Option<Vec<u8>>, MontycatClientError> = keyspace.insert_value_no_schema(value, Some(3600)).await;
    ///
    /// let parsed = MontycatResponse::<Vec<String>>::parse_response(res);
    /// ```
    ///
    /// # Errors
    ///
    /// * `MontycatClientError::ClientStoreNotSet` - If the store is not set in the engine.
    /// * `MontycatClientError::ClientEngineError` - If there is an error with the engine.
    /// * `MontycatClientError::ClientValueParsingError` - If there is an error parsing the response.
    ///
    pub async fn insert_value_no_schema<T>(
        &self,
        custom_key: Option<String>,
        value: T,
        expire_sec: Option<usize>,
    ) -> Result<Option<Vec<u8>>, MontycatClientError>
    where
        T: Serialize + Send + 'static,
    {
        let mut key: Option<String> = None;
        let engine: Engine = self.get_engine();
        let name: &str = self.get_name();
        let persistent: bool = self.get_persistent();
        let distributed: bool = self.get_distributed();
        let store: String = engine
            .store
            .clone()
            .ok_or(MontycatClientError::ClientStoreNotSet)?;
        let use_tls: bool = engine.use_tls;
        let value_to_send: String = process_json_value(&value)?;

        if let Some(custom_key_str) = &custom_key {
            key = Some(convert_custom_key(custom_key_str));
        }

        let command: String = if key.is_none() {
            "insert_value".to_string()
        } else {
            "insert_custom_key_value".to_string()
        };

        let new_store_request: StoreRequestClient = StoreRequestClient {
            username: engine.username.clone(),
            password: engine.password.clone(),
            keyspace: name.to_owned(),
            store,
            persistent,
            distributed,
            value: value_to_send,
            command,
            expire: expire_sec.map(|sec| sec as u64).unwrap_or(0),
            key,
            ..Default::default()
        };

        let bytes: Vec<u8> = Req::new_store_command(new_store_request).byte_down()?;
        let response: Option<Vec<u8>> = send_data(
            &engine.host,
            engine.port,
            bytes.as_slice(),
            None,
            None,
            use_tls,
        )
        .await?;

        Ok(response)
    }

    /// Retrieves keys from the keyspace with optional limit and volume filters.
    ///
    /// # Arguments
    ///
    /// * `volumes` - Optional vector of volume names to filter the keys.
    /// * `latest_volume` - Optional boolean to indicate if only the latest volume should be considered.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Vec<u8>>, MontycatClientError>` - The response from the server or an error.
    ///
    /// # Examples
    ///
    /// ```rust, ignore
    /// let res = keyspace.get_keys(Some(vec!["123456789".into()]), None).await;
    /// let parsed = MontycatResponse::<Vec<String>>::parse_response(res);
    /// ```
    ///
    /// # Errors
    ///
    /// * `MontycatClientError::ClientStoreNotSet` - If the store is not set in the engine.
    /// * `MontycatClientError::ClientEngineError` - If there is an error with the engine.
    /// * `MontycatClientError::ClientValueParsingError` - If there is an error parsing the response.
    ///
    pub async fn get_keys(
        &self,
        volumes: Option<Vec<String>>,
        latest_volume: Option<bool>,
    ) -> Result<Option<Vec<u8>>, MontycatClientError> {
        let engine: Engine = self.get_engine();
        let name: &str = self.get_name();
        let persistent: bool = self.get_persistent();
        let distributed: bool = self.get_distributed();
        let store: String = engine
            .store
            .clone()
            .ok_or(MontycatClientError::ClientStoreNotSet)?;
        let use_tls: bool = engine.use_tls;
        let command: String = "get_keys".to_string();

        let new_store_request: StoreRequestClient = StoreRequestClient {
            username: engine.username.clone(),
            password: engine.password.clone(),
            keyspace: name.to_owned(),
            store,
            persistent,
            distributed,
            command,
            volumes: volumes.unwrap_or_default(),
            latest_volume: latest_volume.unwrap_or_default(),
            ..Default::default()
        };

        let bytes: Vec<u8> = Req::new_store_command(new_store_request).byte_down()?;
        let response: Option<Vec<u8>> = send_data(
            &engine.host,
            engine.port,
            bytes.as_slice(),
            None,
            None,
            use_tls,
        )
        .await?;

        Ok(response)
    }

    /// Updates a value in the keyspace.
    ///
    /// # Arguments
    ///
    /// * `key` - Optional key of the value to update.
    /// * `custom_key` - Optional custom key of the value to update.
    /// * `value` - The new value to set. Must implement `Serialize`.
    /// * `expire_sec` - Optional expiration time in seconds.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Vec<u8>>, MontycatClientError>` - The response from the server or an error.
    ///
    /// # Examples
    ///
    /// ```rust, ignore
    /// let updates = serde_json::json!({ "field1": "new_value" });
    /// let res: Result<Option<Vec<u8>>, MontycatClientError> = keyspace.update_value(Some("key".into()), None, updates, Some(3600)).await;
    /// let parsed = MontycatResponse::<String>::parse_response(res);
    /// ```
    ///
    /// # Errors
    ///
    /// * `MontycatClientError::ClientStoreNotSet` - If the store is not set in the engine.
    /// * `MontycatClientError::ClientEngineError` - If there is an error with the engine.
    /// * `MontycatClientError::ClientValueParsingError` - If there is an error parsing the response.
    ///
    pub async fn update_value<T>(
        &self,
        key: Option<String>,
        custom_key: Option<String>,
        value: T,
        expire_sec: Option<usize>,
    ) -> Result<Option<Vec<u8>>, MontycatClientError>
    where
        T: Serialize + Send + 'static,
    {
        if key.is_none() && custom_key.is_none() || (key.is_some() && custom_key.is_some()) {
            return Err(MontycatClientError::ClientNoValidInputProvided);
        }

        let key: String = key
            .or(custom_key)
            .ok_or(MontycatClientError::ClientNoValidInputProvided)?;

        let engine: Engine = self.get_engine();
        let name: &str = self.get_name();
        let persistent: bool = self.get_persistent();
        let distributed: bool = self.get_distributed();
        let store: String = engine
            .store
            .clone()
            .ok_or(MontycatClientError::ClientStoreNotSet)?;
        let use_tls: bool = engine.use_tls;
        let command: String = "update_value".to_string();
        let value_to_send: String = process_json_value(&value)?;

        let new_store_request: StoreRequestClient = StoreRequestClient {
            key: Some(key),
            username: engine.username.clone(),
            password: engine.password.clone(),
            keyspace: name.to_owned(),
            store,
            persistent,
            distributed,
            value: value_to_send,
            command,
            expire: expire_sec.map(|sec| sec as u64).unwrap_or(0),
            ..Default::default()
        };

        let bytes: Vec<u8> = Req::new_store_command(new_store_request).byte_down()?;
        let response: Option<Vec<u8>> = send_data(
            &engine.host,
            engine.port,
            bytes.as_slice(),
            None,
            None,
            use_tls,
        )
        .await?;

        Ok(response)
    }

    /// Inserts multiple values into the keyspace in bulk.
    ///
    /// # Arguments
    ///
    /// * `bulk_values` - A vector of values to insert. Each value must implement `Serialize` and `RuntimeSchema`.
    /// * `expire_sec` - Optional expiration time in seconds for the inserted values.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Vec<u8>>, MontycatClientError>` - The response from the server or an error.
    ///
    /// # Examples
    ///
    /// ```rust, ignore
    /// let values = vec![YourType { /* fields */ }, YourType { /* fields */ }];
    /// let res: Result<Option<Vec<u8>>, MontycatClientError> = keyspace.insert_bulk(values, Some(3600)).await;
    /// let parsed = MontycatResponse::<Vec<String>>::parse_response(res);
    /// ```
    ///
    /// # Errors
    ///
    /// * `MontycatClientError::ClientStoreNotSet` - If the store is not set in the engine.
    /// * `MontycatClientError::ClientEngineError` - If there is an error with the engine.
    /// * `MontycatClientError::ClientValueParsingError` - If there is an error parsing the response.
    ///
    pub async fn insert_bulk<T>(
        &self,
        bulk_values: Vec<T>,
        expire_sec: Option<usize>,
    ) -> Result<Option<Vec<u8>>, MontycatClientError>
    where
        T: Serialize + RuntimeSchema + Send + 'static,
    {
        let engine: Engine = self.get_engine();
        let name: &str = self.get_name();
        let persistent: bool = self.get_persistent();
        let distributed: bool = self.get_distributed();
        let store: String = engine
            .store
            .clone()
            .ok_or(MontycatClientError::ClientStoreNotSet)?;
        let use_tls: bool = engine.use_tls;
        let command: String = "insert_value".to_string();

        let (value_to_send, schema) = process_bulk_values(bulk_values).await?;

        let new_store_request: StoreRequestClient = StoreRequestClient {
            schema,
            username: engine.username.clone(),
            password: engine.password.clone(),
            keyspace: name.to_owned(),
            store,
            persistent,
            distributed,
            value: value_to_send,
            command,
            expire: expire_sec.map(|sec| sec as u64).unwrap_or(0),
            ..Default::default()
        };

        let bytes: Vec<u8> = Req::new_store_command(new_store_request).byte_down()?;
        let response: Option<Vec<u8>> = send_data(
            &engine.host,
            engine.port,
            bytes.as_slice(),
            None,
            None,
            use_tls,
        )
        .await?;

        Ok(response)
    }

    /// Inserts multiple simple values (without schema) into the keyspace in bulk.
    ///
    /// # Arguments
    ///
    /// * `bulk_values` - A vector of values to insert. Each value must implement `Serialize`.
    /// * `expire_sec` - Optional expiration time in seconds for the inserted values.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Vec<u8>>, MontycatClientError>` - The response from the server or an error.
    ///
    /// # Examples
    ///
    /// ```rust, ignore
    /// let values = vec!["simple_value1", "simple_value2"];
    /// let res: Result<Option<Vec<u8>>, MontycatClientError> = keyspace.insert_bulk_no_schema(values, Some(3600)).await;
    /// let parsed = MontycatResponse::<Vec<String>>::parse_response(res);
    /// ```
    ///
    /// # Errors
    ///
    /// * `MontycatClientError::ClientStoreNotSet` - If the store is not set in the engine.
    /// * `MontycatClientError::ClientEngineError` - If there is an error with the engine.
    /// * `MontycatClientError::ClientValueParsingError` - If there is an error parsing the response.
    ///
    pub async fn insert_bulk_no_schema<T>(
        &self,
        bulk_values: Vec<T>,
        expire_sec: Option<usize>,
    ) -> Result<Option<Vec<u8>>, MontycatClientError>
    where
        T: Serialize + Send + 'static,
    {
        let engine: Engine = self.get_engine();
        let name: &str = self.get_name();
        let persistent: bool = self.get_persistent();
        let distributed: bool = self.get_distributed();
        let store: String = engine
            .store
            .clone()
            .ok_or(MontycatClientError::ClientStoreNotSet)?;
        let use_tls: bool = engine.use_tls;
        let command: String = "insert_value".to_string();

        let value_to_send: String = process_json_value(&bulk_values)?;

        let new_store_request: StoreRequestClient = StoreRequestClient {
            username: engine.username.clone(),
            password: engine.password.clone(),
            keyspace: name.to_owned(),
            store,
            persistent,
            distributed,
            value: value_to_send,
            command,
            expire: expire_sec.map(|sec| sec as u64).unwrap_or(0),
            ..Default::default()
        };

        let bytes: Vec<u8> = Req::new_store_command(new_store_request).byte_down()?;
        let response: Option<Vec<u8>> = send_data(
            &engine.host,
            engine.port,
            bytes.as_slice(),
            None,
            None,
            use_tls,
        )
        .await?;

        Ok(response)
    }

    /// Initiates snapshots for the keyspace.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Vec<u8>>, MontycatClientError>` - The response from the server or an error.
    ///
    /// # Examples
    ///
    /// ```rust, ignore
    /// let res: Result<Option<Vec<u8>>, MontycatClientError> = keyspace.do_snapshots_for_keyspace().await;
    /// ```
    ///
    /// # Errors
    ///
    /// * `MontycatClientError::ClientStoreNotSet` - If the store is not set in the engine.
    /// * `MontycatClientError::ClientEngineError` - If there is an error with the engine.
    /// * `MontycatClientError::ClientValueParsingError` - If there is an error parsing the response.
    ///
    pub async fn do_snapshots_for_keyspace(&self) -> Result<Option<Vec<u8>>, MontycatClientError> {
        let engine: Engine = self.get_engine();
        let name: &str = self.get_name();
        let store: String = engine
            .store
            .clone()
            .ok_or(MontycatClientError::ClientStoreNotSet)?;
        let use_tls: bool = engine.use_tls;

        let vec: Vec<String> = vec![
            "do-snapshots-for-keyspace".into(),
            "store".into(),
            store,
            "keyspace".into(),
            name.to_owned(),
        ];

        let credentials: Vec<String> = engine.get_credentials();
        let query: Req = Req::new_raw_command(vec, credentials);
        let bytes: Vec<u8> = query.byte_down()?;
        let response: Option<Vec<u8>> = send_data(
            &engine.host,
            engine.port,
            bytes.as_slice(),
            None,
            None,
            use_tls,
        )
        .await?;

        Ok(response)
    }

    /// Cleans snapshots for the keyspace.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Vec<u8>>, MontycatClientError>` - The response from the server or an error.
    ///
    /// # Examples
    ///
    /// ```rust, ignore
    /// let res: Result<Option<Vec<u8>>, MontycatClientError> = keyspace.clean_snapshots_for_keyspace().await;
    /// ```
    ///
    /// # Errors
    ///
    /// * `MontycatClientError::ClientStoreNotSet` - If the store is not set in the engine.
    /// * `MontycatClientError::ClientEngineError` - If there is an error with the engine.
    /// * `MontycatClientError::ClientValueParsingError` - If there is an error parsing the response.
    ///
    pub async fn clean_snapshots_for_keyspace(
        &self,
    ) -> Result<Option<Vec<u8>>, MontycatClientError> {
        let engine: Engine = self.get_engine();
        let name: &str = self.get_name();
        let store: String = engine
            .store
            .clone()
            .ok_or(MontycatClientError::ClientStoreNotSet)?;
        let use_tls: bool = engine.use_tls;

        let vec: Vec<String> = vec![
            "clean-snapshots-for-keyspace".into(),
            "store".into(),
            store,
            "keyspace".into(),
            name.to_owned(),
        ];

        let credentials: Vec<String> = engine.get_credentials();
        let query: Req = Req::new_raw_command(vec, credentials);
        let bytes: Vec<u8> = query.byte_down()?;
        let response: Option<Vec<u8>> = send_data(
            &engine.host,
            engine.port,
            bytes.as_slice(),
            None,
            None,
            use_tls,
        )
        .await?;

        Ok(response)
    }

    /// Stops snapshots for the keyspace.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Vec<u8>>, MontycatClientError>` - The response from the server or an error.
    ///
    /// # Examples
    ///
    /// ```rust, ignore
    /// let res: Result<Option<Vec<u8>>, MontycatClientError> = keyspace.stop_snapshots_for_keyspace().await;
    /// ```
    ///
    /// # Errors
    ///
    /// * `MontycatClientError::ClientStoreNotSet` - If the store is not set in the engine.
    /// * `MontycatClientError::ClientEngineError` - If there is an error with the engine.
    /// * `MontycatClientError::ClientValueParsingError` - If there is an error parsing the response.
    ///
    pub async fn stop_snapshots_for_keyspace(
        &self,
    ) -> Result<Option<Vec<u8>>, MontycatClientError> {
        let engine: Engine = self.get_engine();
        let name: &str = self.get_name();
        let store: String = engine
            .store
            .clone()
            .ok_or(MontycatClientError::ClientStoreNotSet)?;
        let use_tls: bool = engine.use_tls;

        let vec: Vec<String> = vec![
            "stop-snapshots-for-keyspace".into(),
            "store".into(),
            store,
            "keyspace".into(),
            name.to_owned(),
        ];

        let credentials: Vec<String> = engine.get_credentials();
        let query: Req = Req::new_raw_command(vec, credentials);
        let bytes: Vec<u8> = query.byte_down()?;
        let response: Option<Vec<u8>> = send_data(
            &engine.host,
            engine.port,
            bytes.as_slice(),
            None,
            None,
            use_tls,
        )
        .await?;

        Ok(response)
    }
}

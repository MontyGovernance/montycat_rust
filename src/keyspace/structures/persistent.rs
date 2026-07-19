use super::super::pubtrait::Keyspace;
use crate::engine::structure::Engine;
use crate::engine::utils::send_data;
use crate::errors::MontycatClientError;
use crate::request::store_request::structure::StoreRequestClient;
use crate::request::utis::functions::convert_custom_key;
use crate::request::{structure::Req, utis::functions::is_custom_type};
use crate::tools::functions::{process_bulk_values, process_json_value, process_value};
use crate::tools::structure::Limit;
use crate::traits::RuntimeSchema;
use serde::Serialize;
use std::any::type_name;
use std::collections::HashMap;

/// Represents a persistent keyspace in the Montycat database.
///
/// # Fields
/// - `name`: The name of the keyspace.
/// - `persistent`: A boolean indicating if the keyspace is persistent.
/// - `distributed`: A boolean indicating if the keyspace is distributed.
/// - `engine`: An instance of the `Engine` struct used for database operations.
///
/// # Examples
/// ```rust, ignore
/// let keyspace: PersistentKeyspace = PersistentKeyspace::new("my_keyspace", &engine);
/// ```
///
#[derive(Debug, Clone)]
pub struct PersistentKeyspace {
    pub name: String,
    pub persistent: bool,
    pub distributed: bool,
    pub engine: Engine,
}

impl Keyspace for PersistentKeyspace {
    /// Retrieves the engine associated with the keyspace.
    ///
    /// # Returns
    /// - `Engine`: The engine instance.
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

impl PersistentKeyspace {
    /// Creates a new PersistentKeyspace instance.
    ///
    /// # Arguments
    /// * `name` - The name of the keyspace.
    /// * `engine` - A reference to the Engine instance.
    ///
    /// # Returns
    /// * `PersistentKeyspace` - A new instance of PersistentKeyspace.
    ///
    /// # Examples
    /// ```rust, ignore,
    /// let keyspace: PersistentKeyspace = PersistentKeyspace::new("my_keyspace", &engine);
    /// ```
    ///
    pub fn new(name: &str, engine: &Engine) -> Self {
        Self {
            name: name.to_owned(),
            persistent: true,
            distributed: false,
            engine: engine.clone(),
        }
    }

    /// Creates a new persistent keyspace in the Montycat database.
    ///
    /// # Arguments
    ///
    /// * `cache` - Optional cache size for the keyspace. Defaults to 0 if None.
    /// * `compression` - Optional compression flag for the keyspace. Defaults to false if None.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Vec<u8>>, MontycatClientError>` - The response from the server or an error.
    ///
    /// # Examples
    ///
    /// ```rust, ignore,
    /// let res: Result<Option<Vec<u8>>, MontycatClientError> = key
    ///   .create_keyspace(Some(1024), Some(true)).await;
    /// ```
    ///
    /// # Errors
    ///
    /// * `MontycatClientError::ClientStoreNotSet` - If the store is not set in the engine.
    /// * `MontycatClientError::ClientEngineError` - If there is an error with the engine.
    /// * `MontycatClientError::ClientValueParsingError` - If there is an error parsing the response.
    ///
    pub async fn create_keyspace(
        &self,
        cache: Option<usize>,
        compression: Option<bool>,
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
            "cache".into(),
            cache.map_or("0".into(), |c| c.to_string()),
            "compression".into(),
            compression.map_or("n".into(), |c| if c { "y".into() } else { "n".into() }),
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

    /// Updates the cache size and compression settings of the persistent keyspace.
    ///
    /// # Arguments
    ///
    /// * `cache` - Optional new cache size for the keyspace. If None, the cache size remains unchanged.
    /// * `compression` - Optional new compression setting for the keyspace. If None, the compression setting remains unchanged.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Vec<u8>>, MontycatClientError>` - The response from the server or an error.
    ///
    /// # Examples
    ///
    /// ```rust, ignore,
    /// let res: Result<Option<Vec<u8>>, MontycatClientError> = keyspace
    /// .update_cache_and_compression(Some(2048), Some(false)).await;
    /// ```
    ///
    /// # Errors
    ///
    /// * `MontycatClientError::ClientStoreNotSet` - If the store is not set in the engine.
    /// * `MontycatClientError::ClientEngineError` - If there is an error with the engine.
    /// * `MontycatClientError::ClientValueParsingError` - If there is an error parsing the response.
    ///
    pub async fn update_cache_and_compression(
        &self,
        cache: Option<usize>,
        compression: Option<bool>,
    ) -> Result<Option<Vec<u8>>, MontycatClientError> {
        let engine: Engine = self.get_engine();
        let name: &str = self.get_name();
        let store: String = engine
            .store
            .clone()
            .ok_or(MontycatClientError::ClientStoreNotSet)?;
        let use_tls: bool = engine.use_tls;

        let vec: Vec<String> = vec![
            "update-cache-compression".into(),
            "store".into(),
            store,
            "keyspace".into(),
            name.to_owned(),
            "cache".into(),
            cache.map_or("0".into(), |c| c.to_string()),
            "compression".into(),
            compression.map_or("n".into(), |c| if c { "y".into() } else { "n".into() }),
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

    /// Inserts a value into the persistent keyspace.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to be inserted into the keyspace. It must implement `Serialize` and `MontycatSchema`.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Vec<u8>>, MontycatClientError>` - The response from the server or an error.
    ///
    /// # Examples
    ///
    /// ```rust, ignore,
    /// let value = YourType { /* fields */ };
    /// let res: Result<Option<Vec<u8>>, MontycatClientError> = keyspace.insert_value(value).await;
    /// let parsed = MontycatResponse::<YourType>::parse_response(res);
    /// ```
    ///
    /// # Errors
    /// * `MontycatClientError::ClientStoreNotSet` - If the store is not set in the engine.
    /// * `MontycatClientError::ClientEngineError` - If there is an error with the engine.
    /// * `MontycatClientError::ClientValueParsingError` - If there is an error parsing the response.
    ///
    pub async fn insert_value<T>(
        &self,
        custom_key: Option<String>,
        value: T,
        wait_for_index: Option<bool>,
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
            key,
            wait_for_index,
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

    /// Inserts a custom key into the persistent keyspace.
    ///
    /// # Arguments
    //
    /// * `custom_key` - The custom key to be inserted into the keyspace.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Vec<u8>>, MontycatClientError>` - The response from the server or an error.
    ///
    /// # Examples
    ///
    /// ```rust, ignore,
    /// let res: Result<Option<Vec<u8>>, MontycatClientError> = keyspace.insert_custom_key("my_custom_key".to_string()).await;
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
        wait_for_index: Option<bool>,
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
            key: Some(key),
            wait_for_index,
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

    /// Inserts a value into the persistent keyspace without enforcing a schema.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to be inserted into the keyspace. It must implement `Serialize`.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Vec<u8>>, MontycatClientError>` - The response from the server or an error.
    ///
    /// # Examples
    ///
    /// ```rust, ignore,
    /// let value = vec!["Hello"];
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

    /// Retrieves keys from the persistent keyspace with optional filtering and limiting.
    ///
    /// # Arguments
    ///
    /// * `limit` - Optional limit for the number of keys to retrieve.
    ///   - `start` is the offset (0-indexed, inclusive).
    ///   - `stop` is the inclusive end index. **`stop=0` is a sentinel meaning "return all"**,
    ///     but only valid when `volumes` or `latest_volume` is also provided.
    ///   - If `limit` is `None`, defaults to `Limit::default_limit()` (stop=0, start=0).
    /// * `volumes` - Optional list of volume names to filter the keys.
    /// * `latest_volume` - Optional flag to only consider the latest volume. Defaults to `false`.
    ///
    /// # Behavior
    ///
    /// At least one of `volumes`, `latest_volume`, or a nonzero `limit` must be provided.
    /// Providing none of these returns an error.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Vec<u8>>, MontycatClientError>` - The response from the server or an error.
    ///
    /// # Examples
    ///
    /// ```rust, ignore,
    /// // Get up to 10 keys starting from index 0 in the latest volume
    /// let res = keyspace.get_keys(Some(Limit::new(0, 10)), None, Some(true)).await;
    ///
    /// // Get ALL keys from volume "2" (stop=0 sentinel)
    /// let res = keyspace.get_keys(Some(Limit::new(0, 0)), Some(vec!["2".into()]), None).await;
    ///
    /// let parsed = MontycatResponse::<Vec<String>>::parse_response(res);
    /// ```
    ///
    /// # Errors
    ///
    /// * `MontycatClientError::ClientGenericError` - If none of volumes/latest_volume/limit are provided.
    /// * `MontycatClientError::ClientGenericError` - If a nonzero `stop` is less than `start`.
    /// * `MontycatClientError::ClientStoreNotSet` - If the store is not set in the engine.
    ///
    pub async fn get_keys(
        &self,
        limit: Option<Limit>,
        volumes: Option<Vec<String>>,
        latest_volume: Option<bool>,
    ) -> Result<Option<Vec<u8>>, MontycatClientError> {
        let has_volumes = volumes.as_ref().is_some_and(|v| !v.is_empty());
        let has_latest_volume = latest_volume.unwrap_or(false);
        let has_limit = limit
            .as_ref()
            .is_some_and(|lim| lim.start != 0 || lim.stop != 0);
        let is_volume_scoped = has_volumes || has_latest_volume;

        if !is_volume_scoped && !has_limit {
            return Err(MontycatClientError::ClientGenericError(
                "Please provide volumes/latest volume or limit.".into(),
            ));
        }

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

        let limit_map: HashMap<String, usize> = match limit {
            Some(lim) => {
                // stop=0 means "return all" when volume-scoped; a nonzero stop must be >= start
                if lim.stop > 0 && lim.start > lim.stop {
                    return Err(MontycatClientError::ClientGenericError(
                        "Limit start cannot be greater than stop".into(),
                    ));
                }
                lim.to_map()
            }
            None => Limit::default_limit().to_map(),
        };

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
            limit_output: limit_map,
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
    ///
    /// # Returns
    ///
    /// * `Result<Option<Vec<u8>>, MontycatClientError>` - The response from the server or an error.
    ///
    /// # Examples
    ///
    /// ```rust, ignore,
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
        wait_for_index: Option<bool>,
    ) -> Result<Option<Vec<u8>>, MontycatClientError>
    where
        T: Serialize + Send + 'static,
    {
        if key.is_none() && custom_key.is_none() || (key.is_some() && custom_key.is_some()) {
            return Err(MontycatClientError::ClientNoValidInputProvided);
        }

        // A custom key must be hashed to its numeric key like every other op;
        // sending it raw makes the server fail to parse it as a key.
        let key: String = match key {
            Some(k) => k,
            None => convert_custom_key(
                &custom_key.ok_or(MontycatClientError::ClientNoValidInputProvided)?,
            ),
        };

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
            wait_for_index,
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
    ///
    /// # Returns
    ///
    /// * `Result<Option<Vec<u8>>, MontycatClientError>` - The response from the server or an error.
    ///
    /// # Examples
    ///
    /// ```rust, ignore,
    /// let values = vec![YourType { /* fields */ }, YourType { /* fields */ }];
    /// let res: Result<Option<Vec<u8>>, MontycatClientError> = keyspace.insert_bulk(values).await;
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
        wait_for_index: Option<bool>,
    ) -> Result<Option<Vec<u8>>, MontycatClientError>
    where
        T: Serialize + RuntimeSchema + Send + 'static + Clone,
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
        let command: String = "insert_bulk".to_string();

        let (serialized_values, schema) = process_bulk_values(bulk_values).await?;

        let new_store_request: StoreRequestClient = StoreRequestClient {
            schema,
            username: engine.username.clone(),
            password: engine.password.clone(),
            keyspace: name.to_owned(),
            store,
            persistent,
            distributed,
            bulk_values: serialized_values,
            command,
            wait_for_index,
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

    /// Inserts multiple values into the keyspace in bulk without enforcing a schema.
    ///
    /// # Arguments
    ///
    /// * `bulk_values` - A vector of values to insert. Each value must implement `Serialize`.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Vec<u8>>, MontycatClientError>` - The response from the server or an error.
    ///
    /// # Examples
    ///
    /// ```rust, ignore,
    /// let values = vec!["value1", "value2", "value3"];
    /// let res: Result<Option<Vec<u8>>, MontycatClientError> = keyspace.insert_bulk_no_schema(values).await;
    /// let parsed = MontycatResponse::<Vec<serde_json::Value>>::parse_response(res);
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
        wait_for_index: Option<bool>,
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
        let command: String = "insert_bulk".to_string();

        // One JSON string per value — the server's insert_bulk creates one
        // record per bulk_values element (see process_bulk_values).
        let serialized_values: Vec<String> = bulk_values
            .iter()
            .map(process_json_value)
            .collect::<Result<Vec<String>, MontycatClientError>>()?;

        let new_store_request: StoreRequestClient = StoreRequestClient {
            username: engine.username.clone(),
            password: engine.password.clone(),
            keyspace: name.to_owned(),
            store,
            persistent,
            distributed,
            bulk_values: serialized_values,
            command,
            wait_for_index,
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
}

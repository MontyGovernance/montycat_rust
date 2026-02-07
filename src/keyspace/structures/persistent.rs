use super::super::pubtrait::Keyspace;
use crate::engine::structure::Engine;
use crate::engine::utils::send_data;
use crate::errors::MontycatClientError;
use crate::request::store_request::structure::StoreRequestClient;
use crate::request::utis::functions::{convert_custom_key, fulfil_subscription_request};
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
/// ```rust,no_run
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
    /// ```rust,no_run
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


    // /// Subscribes to changes in the persistent keyspace.
    // ///
    // /// # Arguments
    // ///
    // /// * `key` - Optional key to subscribe to.
    // /// * `custom_key` - Optional custom key to subscribe to.
    // ///
    // /// # Returns
    // ///
    // /// * `Result<(), MontycatClientError>` - An empty result or an error.
    // ///
    // /// # Examples
    // ///
    // /// ```rust,no_run
    // /// let callback = Arc::new(|data: &Vec<u8>| {
    // ///   println!("Received data: {:?}", data);
    // /// });
    // ///
    // /// let res: Result<(), MontycatClientError> = keyspace.subscribe(Some("my_key".into()), None, callback).await;
    // /// ```
    // ///
    // /// # Errors
    // ///
    // /// * `MontycatClientError::ClientStoreNotSet` - If the store is not set in the engine.
    // /// * `MontycatClientError::ClientSelectedBothKeyAndCustomKey` - If both key and custom_key are provided.
    // ///
    // pub async fn subscribe(&self, key: Option<String>, custom_key: Option<String>, callback: Arc<dyn Fn(&Vec<u8>) + Send + Sync>) -> Result<(), MontycatClientError> {

    //     let engine: Engine = self.get_engine();
    //     let name: &str = self.get_name();
    //     let store: &String = engine.store.as_ref().ok_or(MontycatClientError::ClientStoreNotSet)?;
    //     let use_tls: bool = engine.use_tls;

    //     let key: Option<String> = {
    //         if key.is_some() && custom_key.is_some() {
    //             return Err(MontycatClientError::ClientSelectedBothKeyAndCustomKey);
    //         }
    //         key.or(custom_key)
    //     };

    //     let port: u16 = engine.port + 1;
    //     let request_bytes = fulfil_subscription_request(store, name, key, &engine.username, &engine.password)?;
    //     let _response: Option<Vec<u8>> = send_data(&engine.host, port, request_bytes.as_slice(), Some(callback), None, use_tls).await?;

    //     Ok(())

    // }

    /// Subscribes to changes in the persistent keyspace.
    ///
    /// # Arguments
    ///
    /// * `key` - Optional key to subscribe to.
    /// * `custom_key` - Optional custom key to subscribe to.
    /// * `callback` - Callback function to handle incoming subscription data.
    ///
    /// # Returns
    ///
    /// * `Result<tokio::sync::watch::Sender<bool>, MontycatClientError>` - A sender to stop the subscription or an error.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use montycat::engine::utils::StreamCallback;
    /// use std::sync::Arc;
    ///
    /// let callback: StreamCallback = Arc::new(|data: &Vec<u8>| {
    ///   println!("Received data: {:?}", data);
    /// });
    ///
    /// let stop_tx = keyspace.subscribe(Some("my_key".into()), None, callback).await?;
    /// // To stop the subscription:
    /// // stop_tx.send(true)?;
    /// ```
    ///
    /// # Errors
    ///
    /// * `MontycatClientError::ClientStoreNotSet` - If the store is not set in the engine.
    /// * `MontycatClientError::ClientSelectedBothKeyAndCustomKey` - If both key and custom_key are provided.
    pub async fn subscribe(
        &self,
        key: Option<String>,
        custom_key: Option<String>,
        callback: crate::engine::utils::StreamCallback,
    ) -> Result<tokio::sync::watch::Sender<bool>, MontycatClientError> {
        let (stop_tx, mut stop_rx) = tokio::sync::watch::channel::<bool>(false);

        let engine = self.get_engine();
        let name = self.get_name();
        let store = engine
            .store
            .as_ref()
            .ok_or(MontycatClientError::ClientStoreNotSet)?;
        let use_tls = engine.use_tls;

        let key = {
            if key.is_some() && custom_key.is_some() {
                return Err(MontycatClientError::ClientSelectedBothKeyAndCustomKey);
            }
            key.or(custom_key)
        };

        let port = engine.port + 1;
        let request_bytes =
            fulfil_subscription_request(store, name, key, &engine.username, &engine.password)?;

        let host = engine.host.clone();
        tokio::spawn(async move {
            let _ = send_data(
                &host,
                port,
                request_bytes.as_slice(),
                Some(callback),
                Some(&mut stop_rx),
                use_tls,
            )
            .await;
        });

        Ok(stop_tx)
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
    /// ```rust,no_run
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
    /// ```rust,no_run
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
    /// ```rust,no_run
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
    /// ```rust,no_run
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
    /// ```rust,no_run
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
    /// * `limit` - Optional limit for the number of keys to retrieve. If None, defaults to a standard limit.
    /// * `volumes` - Optional list of volume names to filter the keys. If None, retrieves from all volumes.
    /// * `latest_volume` - Optional flag to indicate if only the latest volume should be considered. Defaults to false if None.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Vec<u8>>, MontycatClientError>` - The response from the server or an error.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// let res: Result<Option<Vec<u8>>, MontycatClientError> = keyspace
    ///   .get_keys(Some(Limit::new(0, 10)), None, Some(true)).await;
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
        limit: Option<Limit>,
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

        let limit_map: HashMap<String, usize> = match limit {
            Some(lim) => {
                if lim.start > lim.stop {
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
    /// ```rust,no_run
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
    /// ```rust,no_run
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
    /// ```rust,no_run
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

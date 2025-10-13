use crate::engine::structure::Engine;
use crate::request::utis::functions::fulfil_subscription_request;
use crate::tools::structure::Limit;
use std::sync::Arc;
use super::super::pubtrait::Keyspace;
use crate::errors::MontycatClientError;
use crate::request::{structure::Req, utis::functions::is_custom_type};
use crate::engine::utils::send_data;
use crate::request::store_request::structure::StoreRequestClient;
use crate::traits::RuntimeSchema;
use std::collections::HashMap;
use serde::Serialize;
use crate::tools::functions::{process_bulk_values, process_json_value, process_value};
use std::any::type_name;


#[derive(Debug, Clone)]
pub struct PersistentKeyspace {
    pub name: String,
    pub persistent: bool,
    pub distributed: bool,
    pub engine: Arc<Engine>
}

impl Keyspace for PersistentKeyspace {

    fn get_engine(&self) -> Arc<Engine> {
        Arc::clone(&self.engine)
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_persistent(&self) -> bool {
        self.persistent
    }

    fn get_distributed(&self) -> bool {
        self.distributed
    }

    fn new(name: &str,  engine: Arc<Engine>) -> Arc<Self> {
        Arc::new(Self {
            name: name.to_owned(),
            persistent: true,
            distributed: false,
            engine
        })
    }
}

impl PersistentKeyspace {

    pub async fn subscribe(&self, key: Option<String>, custom_key: Option<String>, callback: Arc<dyn Fn(&Vec<u8>) + Send + Sync>) -> Result<(), MontycatClientError> {

        let engine: Arc<Engine> = self.get_engine();
        let name: &str = self.get_name();
        let store: &String = engine.store.as_ref().ok_or(MontycatClientError::ClientStoreNotSet)?;

        let key: Option<String> = {
            if key.is_some() && custom_key.is_some() {
                return Err(MontycatClientError::ClientSelectedBothKeyAndCustomKey);
            }
            key.or(custom_key)
        };

        let port: u16 = engine.port + 1;
        let request_bytes = fulfil_subscription_request(store, name, key, &engine.username, &engine.password)?;
        let _response: Option<Vec<u8>> = send_data(&engine.host, port, request_bytes.as_slice(), Some(callback), None).await?;

        Ok(())

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
    pub async fn create_keyspace(&self, cache: Option<usize>, compression: Option<bool>) -> Result<Option<Vec<u8>>, MontycatClientError> {

        let engine: Arc<Engine> = self.get_engine();
        let name: &str = self.get_name();
        let persistent: bool = self.get_persistent();
        let distributed: bool = self.get_distributed();

        let store = engine.store.clone().ok_or(MontycatClientError::ClientStoreNotSet)?;

        let vec: Vec<String> = vec![
            "create-keyspace".into(),
            "store".into(), store,
            "keyspace".into(), name.to_owned(),
            "persistent".into(), if persistent { "y".into() } else { "n".into() },
            "distributed".into(), if distributed { "y".into() } else { "n".into() },
            "cache".into(), cache.map_or("0".into(), |c| c.to_string()),
            "compression".into(), compression.map_or("n".into(), |c| if c { "y".into() } else { "n".into() }),
        ];

        let credentials: Vec<String> = engine.get_credentials();
        let query: Req = Req::new_raw_command(vec, credentials);
        let bytes: Vec<u8> = query.byte_down()?;
        let response: Option<Vec<u8>> = send_data(&engine.host, engine.port, bytes.as_slice(), None, None).await?;

        return Ok(response)

    }

    /// Inserts a value into the persistent keyspace.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to be inserted into the keyspace. It must implement `Serialize` and `MontycatSchema`.
    ///
    /// # Returns
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
    pub async fn insert_value<T>(&self, value: T) -> Result<Option<Vec<u8>>, MontycatClientError>
    where
        T: Serialize + RuntimeSchema + Send + 'static,
    {
        let engine: Arc<Engine> = self.get_engine();
        let name: &str = self.get_name();
        let persistent: bool = self.get_persistent();
        let distributed: bool = self.get_distributed();
        let store: String = engine.store.clone().ok_or(MontycatClientError::ClientStoreNotSet)?;
        let command: String = "insert_value".to_string();
        let mut schema: Option<String> = None;
        let value_to_send: String = process_value(value)?;

        let type_name_retrieved: &str = type_name::<T>();

        if let Some(custom_type_name) = is_custom_type(type_name_retrieved) {
            schema = Some(custom_type_name.to_owned());
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
            ..Default::default()
        };

        let bytes: Vec<u8> = Req::new_store_command(new_store_request).byte_down()?;
        let response: Option<Vec<u8>> = send_data(&engine.host, engine.port, bytes.as_slice(), None, None).await?;

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
    pub async fn insert_value_no_schema<T>(&self, value: T) -> Result<Option<Vec<u8>>, MontycatClientError>
    where
        T: Serialize + Send + 'static,
    {
        let engine: Arc<Engine> = self.get_engine();
        let name: &str = self.get_name();
        let persistent: bool = self.get_persistent();
        let distributed: bool = self.get_distributed();
        let store: String = engine.store.clone().ok_or(MontycatClientError::ClientStoreNotSet)?;
        let command: String = "insert_value".to_string();
        let value_to_send: String = process_json_value(&value)?;

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
        let response: Option<Vec<u8>> = send_data(&engine.host, engine.port, bytes.as_slice(), None, None).await?;

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
    pub async fn get_keys(&self, limit: Option<Limit>, volumes: Option<Vec<String>>, latest_volume: Option<bool>) -> Result<Option<Vec<u8>>, MontycatClientError> {

        let engine: Arc<Engine> = self.get_engine();
        let name: &str = self.get_name();
        let persistent: bool = self.get_persistent();
        let distributed: bool = self.get_distributed();
        let store: String = engine.store.clone().ok_or(MontycatClientError::ClientStoreNotSet)?;
        let command: String = "get_keys".to_string();

        let limit_map: HashMap<String, usize> = match limit {
            Some(lim) => {

                if lim.start > lim.stop {
                    return Err(MontycatClientError::ClientGenericError("Limit start cannot be greater than stop".into()));
                }

                lim.to_map()
            },
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
        let response: Option<Vec<u8>> = send_data(&engine.host, engine.port, bytes.as_slice(), None, None).await?;

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
    pub async fn update_value<T>(&self, key: Option<String>, custom_key: Option<String>, value: T) -> Result<Option<Vec<u8>>, MontycatClientError>
    where
        T: Serialize + Send + 'static,
    {

        if key.is_none() && custom_key.is_none() || (key.is_some() && custom_key.is_some()) {
            return Err(MontycatClientError::ClientNoValidInputProvided);
        }

        let key: String = key.or(custom_key).ok_or(MontycatClientError::ClientNoValidInputProvided)?;

        let engine: Arc<Engine> = self.get_engine();
        let name: &str = self.get_name();
        let persistent: bool = self.get_persistent();
        let distributed: bool = self.get_distributed();
        let store: String = engine.store.clone().ok_or(MontycatClientError::ClientStoreNotSet)?;
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
        let response: Option<Vec<u8>> = send_data(&engine.host, engine.port, bytes.as_slice(), None, None).await?;

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
    pub async fn insert_bulk<T>(&self, bulk_values: Vec<T>) -> Result<Option<Vec<u8>>, MontycatClientError>
    where
        T: Serialize + RuntimeSchema + Send + 'static + Clone,
    {
        let engine: Arc<Engine> = self.get_engine();
        let name: &str = self.get_name();
        let persistent: bool = self.get_persistent();
        let distributed: bool = self.get_distributed();
        let store: String = engine.store.clone().ok_or(MontycatClientError::ClientStoreNotSet)?;
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
        let response: Option<Vec<u8>> = send_data(&engine.host, engine.port, bytes.as_slice(), None, None).await?;

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
    pub async fn insert_bulk_no_schema<T>(&self, bulk_values: Vec<T>) -> Result<Option<Vec<u8>>, MontycatClientError>
    where
        T: Serialize + Send + 'static,
    {
        let engine: Arc<Engine> = self.get_engine();
        let name: &str = self.get_name();
        let persistent: bool = self.get_persistent();
        let distributed: bool = self.get_distributed();
        let store: String = engine.store.clone().ok_or(MontycatClientError::ClientStoreNotSet)?;
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
        let response: Option<Vec<u8>> = send_data(&engine.host, engine.port, bytes.as_slice(), None, None).await?;

        Ok(response)

    }

}
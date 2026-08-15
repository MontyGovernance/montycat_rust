use crate::{
    Limit, MontycatClientError, ResultOrder,
    engine::{structure::Engine, utils::send_data},
    request::{
        store_request::structure::StoreRequestClient,
        structure::Req,
        utis::functions::{
            convert_custom_key, fulfil_subscription_request, merge_bulk_keys_values, merge_keys,
        },
    },
    tools::functions::{define_type, process_json_value},
};
use async_trait::async_trait;
use serde::Serialize;
use std::collections::HashMap;

/// PubTrait defines the public interface for keyspace operations.
///
/// # Trait Methods
/// - `new`: Creates a new instance of the keyspace.
/// - `get_engine`: Retrieves the associated engine.
/// - `get_name`: Retrieves the name of the keyspace.
/// - `get_persistent`: Checks if the keyspace is persistent.
/// - `get_distributed`: Checks if the keyspace is distributed.
/// - `remove_keyspace`: Removes the keyspace from the store.
/// - `get_value`: Retrieves a value by key or custom key.
/// - `delete_key`: Deletes a value by key or custom key.
/// - `list_all_depending_keys`: Lists all keys that depend on a given key or custom key.
/// - `get_bulk`: Retrieves multiple values by a list of keys.
/// - `delete_bulk`: Deletes multiple values by a list of keys.
/// - `get_len`: Gets the length of the keyspace.
/// - `enforce_schema`: Enforces a schema on the keyspace.
/// - `remove_enforced_schema`: Removes an enforced schema from the keyspace.
/// - `update_bulk`: Updates multiple key-value pairs in the keyspace.
///
/// # Errors
/// - `MontycatClientError::ClientStoreNotSet`: If the store is not set in the engine.
/// - `MontycatClientError::ClientEngineError`: If there is an error with the engine.
/// - `MontycatClientError::ClientValueParsingError`: If there is an error parsing the response.
/// - `MontycatClientError::ClientSelectedBothKeyAndCustomKey`: If both key and custom_key are provided.
/// - `MontycatClientError::ClientNoValidInputProvided`: If neither key nor custom_key are provided.
#[allow(clippy::too_many_arguments)]
#[async_trait]
pub trait Keyspace
where
    Self: Sized + Send + Sync,
{
    fn get_engine(&self) -> Engine;
    fn get_name(&self) -> &str;
    fn get_persistent(&self) -> bool;
    fn get_distributed(&self) -> bool;

    /// Remove keyspace
    ///
    /// # Returns
    /// * `Result<Option<Vec<u8>>, MontycatClientError>` - The response from the server or an error
    ///
    /// # Examples
    ///
    /// ```rust, ignore
    /// let res: Result<Option<Vec<u8>>, MontycatClientError> = keyspace.remove_keyspace().await;
    /// ```
    ///
    /// # Errors
    /// * `MontycatClientError::ClientStoreNotSet` - If the store is not set in the engine
    /// * `MontycatClientError::ClientEngineError` - If there is an error with the engine
    /// * `MontycatClientError::ClientValueParsingError` - If there is an error parsing the response
    ///
    async fn remove_keyspace(&self) -> Result<Option<Vec<u8>>, MontycatClientError> {
        let engine: Engine = self.get_engine();
        let name: &str = self.get_name();
        let persistent: bool = self.get_persistent();
        let store: String = engine
            .store
            .clone()
            .ok_or(MontycatClientError::ClientStoreNotSet)?;

        let vec: Vec<String> = vec![
            "remove-keyspace".into(),
            "store".into(),
            store,
            "keyspace".into(),
            name.to_owned(),
            "persistent".into(),
            if persistent { "y".into() } else { "n".into() },
        ];

        let credentials: Vec<String> = engine.get_credentials();
        let query: Req = Req::new_raw_command(vec, credentials);
        let bytes: Vec<u8> = query.byte_down()?;
        let response: Option<Vec<u8>> =
            send_data(&engine, bytes.as_slice(), None, None, None).await?;

        Ok(response)
    }

    /// Get value by key or custom key
    ///
    /// # Arguments
    ///
    /// * `key` - The key to retrieve the value for
    /// * `custom_key` - An optional custom key to retrieve the value for
    /// * `with_pointers` - Whether to include pointers in the returned value
    /// * `key_included` - Whether to include the key in the returned value
    /// * `pointers_metadata` - Whether to include metadata about pointers in the returned value
    ///
    /// # Behavior
    ///
    /// If both key and custom_key are provided, an error is returned
    /// If neither is provided, an error is returned
    /// If pointers_metadata is true, with_pointers must be false
    /// If with_pointers is true, pointers_metadata must be false
    /// If key_included is true, the returned value will include the key
    /// If pointers_metadata is true, the returned value will include metadata about pointers
    /// If custom_key is provided, it will be converted to the internal key format
    /// If key is provided, it will be used as is
    /// Returns the raw response bytes from the server
    ///
    /// # Examples
    ///
    /// Retrieve value with a standard ordered key
    ///
    /// ```rust, ignore
    /// let value: Result<Option<Vec<u8>>> = keyspace.get_value(
    ///     Some("298989599989124434694729184587200373152"),
    ///     None, false, false, false
    /// ).await?;
    /// ```
    ///
    /// Or with a custom key
    ///
    /// ```rust, ignore
    /// let value: Result<Option<Vec<u8>>> = keyspace.get_value(
    ///    None, Some("MyCustomKey123"), true, true, false
    /// ).await?;
    /// ```
    ///
    ///
    /// # Errors
    ///
    /// Returns MontycatClientError if both key and custom_key are provided
    /// Returns MontycatClientError if neither key nor custom_key are provided
    /// Returns MontycatClientError if pointers_metadata and with_pointers are both true
    /// Returns MontycatClientError if the store is not set in the engine
    ///
    async fn get_value(
        &self,
        key: Option<&str>,
        custom_key: Option<&str>,
        with_pointers: bool,
        key_included: bool,
        with_pointers_metadata: bool,
    ) -> Result<Option<Vec<u8>>, MontycatClientError> {
        if key.is_some() && custom_key.is_some() {
            return Err(MontycatClientError::ClientSelectedBothKeyAndCustomKey);
        }

        if key.is_none() && custom_key.is_none() {
            return Err(MontycatClientError::ClientNoValidInputProvided);
        }

        let mut key: String = key.unwrap_or("").to_owned();

        if let Some(custom_key_unwrapped) = custom_key {
            key = convert_custom_key(custom_key_unwrapped);
        }

        let engine: Engine = self.get_engine();
        let name: &str = self.get_name();
        let persistent: bool = self.get_persistent();
        let distributed: bool = self.get_distributed();
        let store: String = engine
            .store
            .clone()
            .ok_or(MontycatClientError::ClientStoreNotSet)?;
        let command: String = "get_value".to_string();

        let new_store_req: StoreRequestClient = StoreRequestClient {
            key: key.to_owned().into(),
            keyspace: name.to_owned(),
            store,
            persistent,
            distributed,
            command,
            with_pointers,
            key_included,
            pointers_metadata: with_pointers_metadata,
            username: engine.username.clone(),
            password: engine.password.clone(),
            ..Default::default()
        };

        let bytes: Vec<u8> = Req::new_store_command(new_store_req).byte_down()?;
        let response: Option<Vec<u8>> =
            send_data(&engine, bytes.as_slice(), None, None, None).await?;

        Ok(response)
    }

    /// Delete value by key or custom key
    ///
    /// # Arguments
    ///
    /// * `key` - The key to delete the value for
    /// * `custom_key` - An optional custom key to delete the value for
    ///
    /// # Behavior
    ///
    /// If both key and custom_key are provided, an error is returned
    /// If neither is provided, an error is returned
    /// If custom_key is provided, it will be converted to the internal key format
    /// If key is provided, it will be used as is
    /// Returns the raw response bytes from the server
    ///
    /// # Examples
    ///
    /// Delete value with a standard ordered key
    ///
    /// ```rust, ignore
    /// let res: Result<Option<Vec<u8>>, MontycatClientError> = keyspace.delete_key(
    ///     "298989599989124434694729184587200373152",
    ///    None
    /// ).await;
    /// ```
    /// Or with a custom key
    /// ```rust, ignore
    /// let res: Result<Option<Vec<u8>>, MontycatClientError> = keyspace.delete_key(
    ///     None,
    ///     Some("MyCustomKey123")
    /// ).await;
    /// ```
    ///
    /// # Errors
    /// * Returns MontycatClientError if both key and custom_key are provided
    /// * Returns MontycatClientError if neither key nor custom_key are provided
    /// * Returns MontycatClientError if the store is not set in the engine
    ///
    async fn delete_key(
        &self,
        key: Option<&str>,
        custom_key: Option<&str>,
        wait_for_index: Option<bool>,
    ) -> Result<Option<Vec<u8>>, MontycatClientError> {
        if key.is_some() && custom_key.is_some() {
            return Err(MontycatClientError::ClientSelectedBothKeyAndCustomKey);
        }

        if key.is_none() && custom_key.is_none() {
            return Err(MontycatClientError::ClientNoValidInputProvided);
        }

        let mut key: String = key.unwrap_or("").to_owned();

        if let Some(custom_key_unwrapped) = custom_key {
            key = convert_custom_key(custom_key_unwrapped);
        }

        let engine: Engine = self.get_engine();
        let name: &str = self.get_name();
        let persistent: bool = self.get_persistent();
        let distributed: bool = self.get_distributed();
        let store: String = engine
            .store
            .clone()
            .ok_or(MontycatClientError::ClientStoreNotSet)?;
        let command: String = "delete_key".to_string();

        let new_store_req: StoreRequestClient = StoreRequestClient {
            key: key.to_owned().into(),
            keyspace: name.to_owned(),
            store,
            persistent,
            distributed,
            command,
            username: engine.username.clone(),
            password: engine.password.clone(),
            wait_for_index,
            ..Default::default()
        };

        let bytes: Vec<u8> = Req::new_store_command(new_store_req).byte_down()?;
        let response: Option<Vec<u8>> =
            send_data(&engine, bytes.as_slice(), None, None, None).await?;

        Ok(response)
    }

    /// List all keys that depend on the given key or custom key
    ///
    /// # Arguments
    ///
    /// * `key` - The key to list dependencies for
    /// * `custom_key` - An optional custom key to list dependencies for
    ///
    /// # Behavior
    ///
    /// * If both key and custom_key are provided, an error is returned
    /// * If neither is provided, an error is returned
    /// * If custom_key is provided, it will be converted to the internal key format
    /// * If key is provided, it will be used as is
    /// * Returns the raw response bytes from the server
    ///
    /// # Examples
    ///
    /// Retrieve dependencies with a standard ordered key
    ///
    /// ```rust, ignore
    /// let value: Result<Option<Vec<u8>>> = keyspace.list_all_depending_keys(
    ///     "298989599989124434694729184587200373152",
    ///    None
    /// ).await?;
    /// ```
    ///
    /// Or with a custom key
    ///
    /// ```rust, ignore
    /// let value: Result<Option<Vec<u8>>> = keyspace.list_all_depending_keys(
    ///    None, Some("MyCustomKey123")
    /// ).await?;
    /// ```
    ///
    /// # Errors
    ///
    /// * Returns MontycatClientError if both key and custom_key are provided
    /// * Returns MontycatClientError if neither key nor custom_key are provided
    /// * Returns MontycatClientError if the store is not set in the engine
    ///
    async fn list_all_depending_keys(
        &self,
        key: &str,
        custom_key: Option<&str>,
    ) -> Result<Option<Vec<u8>>, MontycatClientError> {
        if !key.is_empty() && custom_key.is_some() {
            return Err(MontycatClientError::ClientSelectedBothKeyAndCustomKey);
        }

        let mut key: String = key.to_owned();

        if let Some(custom_key_unwrapped) = custom_key {
            key = convert_custom_key(custom_key_unwrapped);
        }

        let engine: Engine = self.get_engine();
        let name: &str = self.get_name();
        let persistent: bool = self.get_persistent();
        let distributed: bool = self.get_distributed();
        let store: String = engine
            .store
            .clone()
            .ok_or(MontycatClientError::ClientStoreNotSet)?;
        let command: String = "list_all_depending_keys".to_string();

        let new_store_req: StoreRequestClient = StoreRequestClient {
            key: key.to_owned().into(),
            keyspace: name.to_owned(),
            store,
            persistent,
            distributed,
            command,
            username: engine.username.clone(),
            password: engine.password.clone(),
            ..Default::default()
        };

        let bytes: Vec<u8> = Req::new_store_command(new_store_req).byte_down()?;
        let response: Option<Vec<u8>> =
            send_data(&engine, bytes.as_slice(), None, None, None).await?;

        Ok(response)
    }

    /// Get multiple values by a list of keys
    ///
    /// # Arguments
    ///
    /// * `bulk_keys` - A vector of keys to retrieve values for
    /// * `bulk_custom_keys` - A vector of custom keys to retrieve values for
    /// * `with_pointers` - Whether to include pointers in the returned values
    /// * `key_included` - Whether to include the keys in the returned values
    /// * `with_pointers_metadata` - Whether to include metadata about pointers in the returned values
    /// * `limit` - An optional `Limit` struct to limit the number of returned values.
    ///   - `start` is the offset (0-indexed, inclusive).
    ///   - `stop` is the inclusive end index. **`stop=0` is a sentinel meaning "return all"**,
    ///     valid only when `volumes` or `latest_volume` is also provided.
    ///   - A nonzero `stop` must be >= `start`.
    /// * `volumes` - An optional vector of volume names to filter the returned values.
    /// * `latest_volume` - An optional boolean to indicate whether to only return values from the latest volume.
    ///
    /// # Behavior
    ///
    /// * Sends a request to the server to retrieve values for the provided keys
    /// * Returns the raw response bytes from the server
    ///
    /// # Examples
    ///
    /// ```rust, ignore
    /// let keys = vec![
    ///     "298989599989124434694729184587200373152".to_string(),
    ///     "298989599989124434694729184587200373153".to_string(),
    /// ];
    ///
    /// let values: Result<Option<Vec<u8>>, MontycatClientError> = keyspace.get_bulk(keys).await;
    /// ```
    ///
    /// # Errors
    /// * Returns MontycatClientError if the store is not set in the engine
    /// * Returns MontycatClientError if there is an error with the engine
    /// * Returns MontycatClientError if there is an error parsing the response
    /// * Returns MontycatClientError if both with_pointers and with_pointers_metadata are true
    /// * Returns MontycatClientError if multiple conflicting options are provided (keys, volumes, latest_volume)
    ///
    #[allow(clippy::too_many_arguments)]
    async fn get_bulk(
        &self,
        bulk_keys: Option<Vec<String>>,
        bulk_custom_keys: Option<Vec<String>>,
        with_pointers: bool,
        key_included: bool,
        with_pointers_metadata: bool,
        limit: Option<Limit>,
        order: Option<ResultOrder>,
        volumes: Option<Vec<String>>,
        latest_volume: Option<bool>,
    ) -> Result<Option<Vec<u8>>, MontycatClientError> {
        let processed_keys: Vec<String> = if bulk_keys.is_some() || bulk_custom_keys.is_some() {
            merge_keys(bulk_keys, bulk_custom_keys).await?
        } else {
            Vec::new()
        };

        let has_keys = !processed_keys.is_empty();
        let has_volumes = volumes.as_ref().is_some_and(|v| !v.is_empty());
        let has_latest_volume = latest_volume.unwrap_or(false);
        let has_limit = limit
            .as_ref()
            .is_some_and(|lim| lim.start != 0 || lim.stop != 0);

        let has_range_options = has_volumes || has_latest_volume || has_limit;

        if (has_keys && has_range_options) || (!has_keys && !has_range_options) {
            return Err(MontycatClientError::ClientGenericError(
                "Please provide keys OR (volumes/latest volume and/or limit).".into(),
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
        let command: String = "get_bulk".to_string();

        let limit_map: HashMap<String, usize> = match &limit {
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

        let new_store_req: StoreRequestClient = StoreRequestClient {
            bulk_keys: processed_keys,
            keyspace: name.to_owned(),
            volumes: volumes.unwrap_or_default(),
            latest_volume: latest_volume.unwrap_or(false),
            store,
            persistent,
            distributed,
            command,
            limit_output: limit_map,
            order,
            username: engine.username.clone(),
            password: engine.password.clone(),
            with_pointers,
            key_included,
            pointers_metadata: with_pointers_metadata,
            ..Default::default()
        };

        let bytes: Vec<u8> = Req::new_store_command(new_store_req).byte_down()?;
        let response: Option<Vec<u8>> =
            send_data(&engine, bytes.as_slice(), None, None, None).await?;

        Ok(response)
    }

    /// Delete multiple values by a list of keys
    ///
    /// # Arguments
    ///
    /// * `bulk_keys` - A vector of keys to delete values for
    /// * `bulk_custom_keys` - A vector of custom keys to delete values for
    ///
    /// # Behavior
    ///
    /// * Sends a request to the server to delete values for the provided keys
    /// * Returns the raw response bytes from the server
    ///
    /// # Examples
    ///
    /// ```rust, ignore
    /// let keys = vec![
    ///     "298989599989124434694729184587200373152".to_string(),
    ///     "298989599989124434694729184587200373153".to_string(),
    /// ];
    ///
    /// let custom_keys = vec![
    ///     "MyCustomKey1".to_string(),
    ///     "MyCustomKey2".to_string(),
    /// ];
    ///
    /// let res: Result<Option<Vec<u8>>, MontycatClientError> = keyspace.delete_bulk(Some(keys), Some(custom_keys)).await;
    /// ```
    /// # Errors
    ///
    /// * Returns MontycatClientError if neither bulk_keys nor bulk_custom_keys are provided
    /// * Returns MontycatClientError if the store is not set in the engine
    /// * Returns MontycatClientError if there is an error with the engine
    /// * Returns MontycatClientError if there is an error parsing the response
    ///
    async fn delete_bulk(
        &self,
        bulk_keys: Option<Vec<String>>,
        bulk_custom_keys: Option<Vec<String>>,
        wait_for_index: Option<bool>,
    ) -> Result<Option<Vec<u8>>, MontycatClientError> {
        let keys_processed: Vec<String> = merge_keys(bulk_keys, bulk_custom_keys).await?;

        let engine: Engine = self.get_engine();
        let name: &str = self.get_name();
        let persistent: bool = self.get_persistent();
        let distributed: bool = self.get_distributed();
        let store: String = engine
            .store
            .clone()
            .ok_or(MontycatClientError::ClientStoreNotSet)?;
        let command: String = "delete_bulk".to_string();

        let new_store_req: StoreRequestClient = StoreRequestClient {
            bulk_keys: keys_processed,
            keyspace: name.to_owned(),
            store,
            persistent,
            distributed,
            command,
            username: engine.username.clone(),
            password: engine.password.clone(),
            wait_for_index,
            ..Default::default()
        };

        let bytes: Vec<u8> = Req::new_store_command(new_store_req).byte_down()?;
        let response: Option<Vec<u8>> =
            send_data(&engine, bytes.as_slice(), None, None, None).await?;

        Ok(response)
    }

    /// Get the length of the keyspace
    ///
    /// # Returns
    ///
    /// * `Result<Option<Vec<u8>>, MontycatClientError>` - The response from the server or an error
    ///
    /// # Examples
    ///
    /// ```rust, ignore
    /// let len: Result<Option<Vec<u8>>, MontycatClientError> = keyspace.get_len().await;
    /// ```
    ///
    /// # Errors
    ///
    /// * `MontycatClientError::ClientStoreNotSet` - If the store is not set in the engine
    /// * `MontycatClientError::ClientEngineError` - If there is an error with the engine
    /// * `MontycatClientError::ClientValueParsingError` - If there is an error parsing the response
    ///
    async fn get_len(&self) -> Result<Option<Vec<u8>>, MontycatClientError> {
        let engine: Engine = self.get_engine();
        let name: &str = self.get_name();
        let persistent: bool = self.get_persistent();
        let distributed: bool = self.get_distributed();
        let store: String = engine
            .store
            .clone()
            .ok_or(MontycatClientError::ClientStoreNotSet)?;
        let command: String = "get_len".to_string();

        let new_store_req: StoreRequestClient = StoreRequestClient {
            keyspace: name.to_owned(),
            store,
            persistent,
            distributed,
            command,
            username: engine.username.clone(),
            password: engine.password.clone(),
            ..Default::default()
        };

        let bytes: Vec<u8> = Req::new_store_command(new_store_req).byte_down()?;
        let response: Option<Vec<u8>> =
            send_data(&engine, bytes.as_slice(), None, None, None).await?;

        Ok(response)
    }

    /// Enforce schema on the keyspace
    ///
    /// # Arguments
    ///
    /// * `schema_params` - A tuple containing a HashMap of field names to types and the schema name
    ///
    /// # Behavior
    ///
    /// Sends a request to the server to enforce the provided schema on the keyspace
    /// Returns the raw response bytes from the server
    ///
    /// # Examples
    ///
    /// ```rust, ignore
    ///
    /// #[derive(Serialize, RuntimeSchema, Deserialize, Debug, Clone)]
    /// struct MyStruct {
    ///   field1: String,
    ///   field2: i32,
    /// }
    ///
    /// let res: Result<Option<Vec<u8>>, MontycatClientError> = keyspace.enforce_schema(MyStruct::schema_params()).await;
    /// ```
    ///
    /// # Errors
    ///
    /// * Returns MontycatClientError if there is an error defining the type
    /// * Returns MontycatClientError if there is an error serializing the schema types
    /// * Returns MontycatClientError if the store is not set in the engine
    /// * Returns MontycatClientError if there is an error with the engine
    /// * Returns MontycatClientError if there is an error parsing the response
    ///
    async fn enforce_schema(
        &self,
        schema_params: (std::collections::HashMap<&str, &str>, &str),
    ) -> Result<Option<Vec<u8>>, MontycatClientError> {
        let (fields, schema_name) = schema_params;

        let mut schema_types: HashMap<String, (&'static str, bool)> = HashMap::new();

        for (field_name, field_type) in fields.into_iter() {
            let type_def = define_type(field_type)?;
            schema_types.insert(field_name.to_string(), type_def);
        }

        let schema_types_as_string: String = serde_json::to_string(&schema_types)
            .map_err(|e| MontycatClientError::ClientValueParsingError(e.to_string()))?;

        let engine: Engine = self.get_engine();
        let name: &str = self.get_name();
        let persistent: bool = self.get_persistent();
        let store: String = engine
            .store
            .clone()
            .ok_or(MontycatClientError::ClientStoreNotSet)?;

        let vec: Vec<String> = vec![
            "enforce-schema".into(),
            "store".into(),
            store,
            "keyspace".into(),
            name.to_owned(),
            "persistent".into(),
            if persistent { "y".into() } else { "n".into() },
            "schema_name".into(),
            schema_name.to_string(),
            "schema_content".into(),
            schema_types_as_string,
        ];

        let credentials: Vec<String> = engine.get_credentials();
        let query: Req = Req::new_raw_command(vec, credentials);
        let bytes: Vec<u8> = query.byte_down()?;
        let response: Option<Vec<u8>> =
            send_data(&engine, bytes.as_slice(), None, None, None).await?;

        Ok(response)
    }

    /// Remove enforced schema from the keyspace
    ///
    /// # Arguments
    ///
    /// * `schema_name` - The name of the schema to remove
    ///
    /// # Behavior
    ///
    /// Sends a request to the server to remove the enforced schema from the keyspace
    /// Returns the raw response bytes from the server
    ///
    /// # Examples
    ///
    /// ```rust, ignore
    /// let res: Result<Option<Vec<u8>>, MontycatClientError> = keyspace.remove_enforced_schema(MyStruct::schema_params()).await;
    /// ```
    ///
    /// # Errors
    ///
    /// * Returns MontycatClientError if there is an error defining the type
    /// * Returns MontycatClientError if there is an error serializing the schema types
    /// * Returns MontycatClientError if the store is not set in the engine
    /// * Returns MontycatClientError if there is an error with the engine
    /// * Returns MontycatClientError if there is an error parsing the response
    ///
    async fn remove_enforced_schema(
        &self,
        schema_name: (HashMap<&str, &str>, &str),
    ) -> Result<Option<Vec<u8>>, MontycatClientError> {
        let (_fields, schema_name) = schema_name;

        let engine: Engine = self.get_engine();
        let name: &str = self.get_name();
        let persistent: bool = self.get_persistent();
        let store: String = engine
            .store
            .clone()
            .ok_or(MontycatClientError::ClientStoreNotSet)?;

        let vec: Vec<String> = vec![
            "remove-enforced-schema".into(),
            "store".into(),
            store,
            "keyspace".into(),
            name.to_owned(),
            "persistent".into(),
            if persistent { "y".into() } else { "n".into() },
            "schema_name".into(),
            schema_name.to_string(),
        ];

        let credentials: Vec<String> = engine.get_credentials();
        let query: Req = Req::new_raw_command(vec, credentials);
        let bytes: Vec<u8> = query.byte_down()?;
        let response: Option<Vec<u8>> =
            send_data(&engine, bytes.as_slice(), None, None, None).await?;

        Ok(response)
    }

    /// List all schemas in the keyspace
    ///
    /// # Behavior
    ///
    /// Sends a request to the server to list all schemas in the keyspace
    /// Returns the raw response bytes from the server
    ///
    /// # Examples
    ///
    /// ```rust, ignore
    /// let res: Result<Option<Vec<u8>>, MontycatClientError> = keyspace.list_all_schemas_in_keyspace().await;
    /// ```
    ///
    /// # Errors
    /// * Returns MontycatClientError if the store is not set in the engine
    /// * Returns MontycatClientError if there is an error with the engine
    /// * Returns MontycatClientError if there is an error parsing the response
    ///
    async fn list_all_schemas_in_keyspace(&self) -> Result<Option<Vec<u8>>, MontycatClientError> {
        let engine: Engine = self.get_engine();
        let name: &str = self.get_name();
        let persistent: bool = self.get_persistent();
        let distributed: bool = self.get_distributed();
        let store: String = engine
            .store
            .clone()
            .ok_or(MontycatClientError::ClientStoreNotSet)?;
        let command: String = "list_all_schemas_in_keyspace".to_string();

        let new_store_request: StoreRequestClient = StoreRequestClient {
            username: engine.username.clone(),
            password: engine.password.clone(),
            keyspace: name.to_owned(),
            store,
            persistent,
            distributed,
            command,
            ..Default::default()
        };

        let bytes: Vec<u8> = Req::new_store_command(new_store_request).byte_down()?;
        let response: Option<Vec<u8>> =
            send_data(&engine, bytes.as_slice(), None, None, None).await?;

        Ok(response)
    }

    /// Update multiple key-value pairs in the keyspace
    ///
    /// # Arguments
    ///
    /// * `bulk_keys_values` - A vector of HashMaps containing key-value pairs to update
    /// * `bulk_custom_keys_values` - A vector of HashMaps containing custom key-value pairs to update
    /// * `vectors` - Optional numeric keys mapped to precomputed vectors
    /// * `custom_vectors` - Optional custom keys mapped to precomputed vectors
    /// * `wait_for_index` - Optional override for waiting until indexes are updated
    ///
    /// # Behavior
    ///
    /// Merges the provided key-value pairs and sends a request to the server to update them in the keyspace
    /// Returns the raw response bytes from the server
    ///
    /// # Examples
    ///
    /// ```rust, ignore
    ///
    /// let bulk_keys_values = vec![
    ///     hashmap![("298989599989124434694729184587200373152".to_string(), "value1".to_string())],
    ///     hashmap![("298989599989124434694729184587200373153".to_string(), "value2".to_string())],
    /// ];
    ///
    /// let bulk_custom_keys_values = vec![
    ///     hashmap![("MyCustomKey1".to_string(), "custom_value1".to_string())],
    ///     hashmap![("MyCustomKey2".to_string(), "custom_value2".to_string())],
    /// ];
    ///
    /// let res: Result<Option<Vec<u8>>, MontycatClientError> = keyspace
    ///     .update_bulk(bulk_keys_values, bulk_custom_keys_values, None, None, None)
    ///     .await;
    ///
    /// let parsed = MontycatResponse::<Vec<serde_json::Value>>::parse_response(res);
    ///
    /// ```
    ///
    /// # Errors
    ///
    /// * Returns MontycatClientError if neither bulk_keys_values nor bulk_custom_keys_values are provided
    /// * Returns MontycatClientError if there is an error merging the key-value pairs
    /// * Returns MontycatClientError if there is an error processing the JSON value
    /// * Returns MontycatClientError if the store is not set in the engine
    /// * Returns MontycatClientError if there is an error with the engine
    /// * Returns MontycatClientError if there is an error parsing the response
    ///
    async fn update_bulk<T>(
        &self,
        bulk_keys_values: Vec<HashMap<String, T>>,
        bulk_custom_keys_values: Vec<HashMap<String, T>>,
        vectors: Option<HashMap<String, Vec<f32>>>,
        custom_vectors: Option<HashMap<String, Vec<f32>>>,
        wait_for_index: Option<bool>,
    ) -> Result<Option<Vec<u8>>, MontycatClientError>
    where
        T: Serialize + Send + 'static,
    {
        if bulk_keys_values.is_empty() && bulk_custom_keys_values.is_empty() {
            return Err(MontycatClientError::ClientNoValidInputProvided);
        }

        let bulk: HashMap<String, String> =
            merge_bulk_keys_values(bulk_keys_values, bulk_custom_keys_values).await?;
        let mut semantic_vectors = vectors.unwrap_or_default();
        for (key, vector) in custom_vectors.unwrap_or_default() {
            semantic_vectors.insert(convert_custom_key(&key), vector);
        }

        let engine: Engine = self.get_engine();
        let name: &str = self.get_name();
        let persistent: bool = self.get_persistent();
        let distributed: bool = self.get_distributed();
        let store: String = engine
            .store
            .clone()
            .ok_or(MontycatClientError::ClientStoreNotSet)?;
        let command: String = "update_bulk".to_string();

        let new_store_request: StoreRequestClient = StoreRequestClient {
            bulk_keys_values: bulk,
            username: engine.username.clone(),
            password: engine.password.clone(),
            keyspace: name.to_owned(),
            store,
            persistent,
            distributed,
            command,
            wait_for_index,
            semantic_vectors,
            ..Default::default()
        };

        let bytes: Vec<u8> = Req::new_store_command(new_store_request).byte_down()?;
        let response: Option<Vec<u8>> =
            send_data(&engine, bytes.as_slice(), None, None, None).await?;

        Ok(response)
    }

    /// Lookup keys in the keyspace based on provided filters
    ///
    /// # Arguments
    ///
    /// * `filters` - A serializable object representing the filters to apply
    /// * `limit` - An optional Limit struct to limit the number of results
    /// * `schema` - An optional schema name to apply during the lookup
    ///
    /// # Behavior
    ///
    /// Sends a request to the server to lookup keys based on the provided search_criteria and limit
    /// Returns the raw response bytes from the server
    ///
    /// # Examples
    ///
    /// ```rust, ignore
    /// use serde_json::json;
    ///
    /// let search_criteria = json!({
    ///     "field1": "value1",
    ///     "field2": { "num": 10 }
    /// });
    ///
    /// let limit = Some(Limit { start: 0, stop: 10 });
    ///
    /// let res: Result<Option<Vec<u8>>, MontycatClientError> = keyspace.lookup_keys_where(search_criteria, limit, Some("MySchema".to_string())).await;
    ///
    /// let parsed = MontycatResponse::<Vec<serde_json::Value>>::parse_response(res);
    ///
    /// ```
    ///
    /// # Errors
    ///
    /// * Returns MontycatClientError if there is an error processing the JSON value
    /// * Returns MontycatClientError if the store is not set in the engine
    /// * Returns MontycatClientError if there is an error with the engine
    /// * Returns MontycatClientError if there is an error parsing the response
    ///
    async fn lookup_keys_where<T>(
        &self,
        search_criteria: T,
        limit: Option<Limit>,
        order: Option<ResultOrder>,
        schema_name: Option<(HashMap<&str, &str>, &str)>,
    ) -> Result<Option<Vec<u8>>, MontycatClientError>
    where
        T: Serialize + Send + 'static,
    {
        let schema = {
            if let Some((_, schema_name)) = schema_name {
                Some(schema_name.to_string())
            } else {
                None
            }
        };

        let engine: Engine = self.get_engine();
        let name: &str = self.get_name();
        let persistent: bool = self.get_persistent();
        let distributed: bool = self.get_distributed();
        let store: String = engine
            .store
            .clone()
            .ok_or(MontycatClientError::ClientStoreNotSet)?;
        let command: String = "lookup_keys".to_string();

        let filters_serialized: String = process_json_value(&search_criteria)?;

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
            schema,
            limit_output: limit_map,
            order,
            search_criteria: filters_serialized,
            username: engine.username.clone(),
            password: engine.password.clone(),
            keyspace: name.to_owned(),
            store,
            persistent,
            distributed,
            command,
            ..Default::default()
        };

        let bytes: Vec<u8> = Req::new_store_command(new_store_request).byte_down()?;
        let response: Option<Vec<u8>> =
            send_data(&engine, bytes.as_slice(), None, None, None).await?;

        Ok(response)
    }

    /// Lookup values in the keyspace based on provided filters
    ///
    /// # Arguments
    ///
    /// * `filters` - A serializable object representing the filters to apply
    /// * `limit` - An optional Limit struct to limit the number of results
    /// * `with_pointers` - Whether to include pointers in the returned values
    /// * `key_included` - Whether to include the key in the returned values
    /// * `pointers_metadata` - Whether to include metadata about pointers in the returned values
    /// * `schema` - An optional schema name to apply during the lookup
    ///
    /// # Behavior
    ///
    /// Sends a request to the server to lookup values based on the provided filters and limit
    /// Returns the raw response bytes from the server
    ///
    /// # Examples
    ///
    /// ```rust, ignore
    /// use serde_json::json;
    ///
    /// let search_criteria = json!({
    ///     "field1": "value1",
    ///    "field2": { "num": 10 }
    /// });
    ///
    /// let limit = Some(Limit { start: 0, stop: 10 });
    ///
    /// let res: Result<Option<Vec<u8>>, MontycatClientError> = keyspace.lookup_values_where(search_criteria, limit, true, true, false, Some("MySchema".to_string())).await;
    ///
    /// let parsed = MontycatResponse::<Vec<serde_json::Value>>::parse_response(res);
    /// ```
    ///
    /// # Errors
    ///
    /// * Returns MontycatClientError if there is an error processing the JSON value
    /// * Returns MontycatClientError if the store is not set in the engine
    /// * Returns MontycatClientError if there is an error with the engine
    /// * Returns MontycatClientError if there is an error parsing the response
    ///
    async fn lookup_values_where<T>(
        &self,
        search_criteria: T,
        limit: Option<Limit>,
        order: Option<ResultOrder>,
        with_pointers: bool,
        key_included: bool,
        pointers_metadata: bool,
        schema_name: Option<(HashMap<&str, &str>, &str)>,
    ) -> Result<Option<Vec<u8>>, MontycatClientError>
    where
        T: Serialize + Send + 'static,
    {
        let schema = {
            if let Some((_, schema_name)) = schema_name {
                Some(schema_name.to_string())
            } else {
                None
            }
        };

        let engine: Engine = self.get_engine();
        let name: &str = self.get_name();
        let persistent: bool = self.get_persistent();
        let distributed: bool = self.get_distributed();
        let store: String = engine
            .store
            .clone()
            .ok_or(MontycatClientError::ClientStoreNotSet)?;
        let command: String = "lookup_values".to_string();

        let filters_serialized: String = process_json_value(&search_criteria)?;

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
            with_pointers,
            key_included,
            pointers_metadata,
            schema,
            limit_output: limit_map,
            order,
            search_criteria: filters_serialized,
            username: engine.username.clone(),
            password: engine.password.clone(),
            keyspace: name.to_owned(),
            store,
            persistent,
            distributed,
            command,
            ..Default::default()
        };

        let bytes: Vec<u8> = Req::new_store_command(new_store_request).byte_down()?;
        let response: Option<Vec<u8>> =
            send_data(&engine, bytes.as_slice(), None, None, None).await?;

        Ok(response)
    }

    /// Shared core for `semantic_search_get_keys` / `semantic_search_get_values`.
    /// The server command is the same either way (`semantic_search`); the two
    /// public methods differ only in which value-inclusion flags they pass, so
    /// the wire call lives here once. `search_criteria` carries the raw query
    /// text (the engine trims it as a plain string) — not a JSON filter map.
    #[doc(hidden)]
    async fn semantic_search_core(
        &self,
        query: &str,
        semantic_vector: Option<Vec<f32>>,
        limit: Option<Limit>,
        min_score: Option<f32>,
        filters: Option<serde_json::Value>,
        with_pointers: bool,
        key_included: bool,
        pointers_metadata: bool,
    ) -> Result<Option<Vec<u8>>, MontycatClientError> {
        if semantic_vector.is_none() && query.trim().is_empty() {
            return Err(MontycatClientError::ClientNoValidInputProvided);
        }
        if semantic_vector.as_ref().is_some_and(|vector| {
            vector.is_empty() || vector.iter().any(|value| !value.is_finite())
        }) {
            return Err(MontycatClientError::ClientNoValidInputProvided);
        }

        // Hybrid metadata pre-filter: JSON-encoded like lookup criteria,
        // omitted from the wire when None. An empty criteria object is
        // rejected rather than sent — it would match nothing server-side, so
        // the caller almost certainly meant the unfiltered method (same guard
        // the Python/Node/Dart clients apply).
        let semantic_filter: Option<String> = match &filters {
            Some(criteria) => {
                if criteria.as_object().is_none_or(|map| map.is_empty()) {
                    return Err(MontycatClientError::ClientNoValidInputProvided);
                }
                Some(process_json_value(criteria)?)
            }
            None => None,
        };

        let engine: Engine = self.get_engine();
        let name: &str = self.get_name();
        let persistent: bool = self.get_persistent();
        let distributed: bool = self.get_distributed();
        let store: String = engine
            .store
            .clone()
            .ok_or(MontycatClientError::ClientStoreNotSet)?;
        let command: String = "semantic_search".to_string();

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
            with_pointers,
            key_included,
            pointers_metadata,
            min_score,
            semantic_filter,
            semantic_vector,
            limit_output: limit_map,
            search_criteria: query.to_owned(),
            username: engine.username.clone(),
            password: engine.password.clone(),
            keyspace: name.to_owned(),
            store,
            persistent,
            distributed,
            command,
            ..Default::default()
        };

        let bytes: Vec<u8> = Req::new_store_command(new_store_request).byte_down()?;
        let response: Option<Vec<u8>> =
            send_data(&engine, bytes.as_slice(), None, None, None).await?;

        Ok(response)
    }

    /// Semantic (vector similarity) search returning ranked keys only.
    ///
    /// Ranks stored items by how close their embeddings are to the embedding of
    /// `query` and returns just the matched key and score for each hit — the
    /// lightweight variant when you only need identity + ranking (e.g. to then
    /// `get_bulk` a page, or to test membership). Use `semantic_search_get_values`
    /// when you want the values inline.
    ///
    /// Semantic search must be enabled first (see `Engine::enable_semantic_search`).
    /// The keyspace is embedded in the background as items are written, so results
    /// reflect whatever has been embedded so far.
    ///
    /// # Arguments
    ///
    /// * `query` - The natural-language query text; may be empty when `vector` is supplied
    /// * `vector` - Optional precomputed query vector that bypasses text embedding
    /// * `limit` - An optional Limit over the ranked hits; None lets the server
    ///   apply its default top-k (10)
    /// * `min_score` - Drop hits whose cosine similarity (in [-1, 1]) is below
    ///   this value; None applies no score filter
    ///
    /// # Examples
    ///
    /// ```rust, ignore
    /// let res = keyspace.semantic_search_get_keys("astronomy and outer space", None, Some(Limit { start: 0, stop: 3 }), None).await;
    /// let parsed = MontycatResponse::<Vec<serde_json::Value>>::parse_response(res);
    /// // each hit: {"__key__": ..., "__score__": ...}
    /// ```
    ///
    /// # Errors
    ///
    /// * `MontycatClientError::ClientNoValidInputProvided` - If neither query text nor a valid vector is supplied
    /// * `MontycatClientError::ClientStoreNotSet` - If the store is not set in the engine
    /// * `MontycatClientError::ClientEngineError` - If there is an error with the engine
    ///
    async fn semantic_search_get_keys(
        &self,
        query: &str,
        vector: Option<Vec<f32>>,
        limit: Option<Limit>,
        min_score: Option<f32>,
    ) -> Result<Option<Vec<u8>>, MontycatClientError> {
        self.semantic_search_core(query, vector, limit, min_score, None, false, false, false)
            .await
    }

    /// Hybrid semantic search returning ranked keys only, restricted by a
    /// metadata filter.
    ///
    /// Same ranking as `semantic_search_get_keys`, but only items matching
    /// `filters` are considered — a hard AND constraint through the same
    /// criteria stack as `lookup_keys_where` (indexed fields, timestamps,
    /// pointers). Scores stay pure cosine; the filter never boosts, it only
    /// restricts. A filter matching nothing returns `[]`.
    ///
    /// A separate method (not a parameter on `semantic_search_get_keys`) so
    /// existing integrations keep their exact signature.
    ///
    /// # Arguments
    ///
    /// * `query` - The natural-language query text; may be empty when `vector` is supplied
    /// * `vector` - Optional precomputed query vector that bypasses text embedding
    /// * `filters` - Metadata criteria, same shape `lookup_keys_where` takes
    /// * `limit` - An optional Limit over the ranked hits; None lets the server
    ///   apply its default top-k (10)
    /// * `min_score` - Drop hits whose cosine similarity (in [-1, 1]) is below
    ///   this value; None applies no score filter
    ///
    /// # Examples
    ///
    /// ```rust, ignore
    /// // only rank items whose indexed `category` equals "space"
    /// let res = keyspace.semantic_search_get_keys_where("astronomy", None, serde_json::json!({"category": "space"}), None, None).await;
    /// let parsed = MontycatResponse::<Vec<serde_json::Value>>::parse_response(res);
    /// // each hit: {"__key__": ..., "__score__": ...}
    /// ```
    ///
    /// # Errors
    ///
    /// * `MontycatClientError::ClientNoValidInputProvided` - If neither query text nor a valid vector is supplied, or filters are empty
    /// * `MontycatClientError::ClientStoreNotSet` - If the store is not set in the engine
    /// * `MontycatClientError::ClientEngineError` - If there is an error with the engine
    ///
    async fn semantic_search_get_keys_where(
        &self,
        query: &str,
        vector: Option<Vec<f32>>,
        filters: serde_json::Value,
        limit: Option<Limit>,
        min_score: Option<f32>,
    ) -> Result<Option<Vec<u8>>, MontycatClientError> {
        self.semantic_search_core(
            query,
            vector,
            limit,
            min_score,
            Some(filters),
            false,
            false,
            false,
        )
        .await
    }

    /// Semantic (vector similarity) search returning ranked hits with their values.
    ///
    /// Ranks stored items by how close their embeddings are to the embedding of
    /// `query` and returns the value inline with each hit — the key is always
    /// included so every value is tagged with its key. Use
    /// `semantic_search_get_keys` when you only need keys + scores.
    ///
    /// Semantic search must be enabled first (see `Engine::enable_semantic_search`).
    /// The keyspace is embedded in the background as items are written, so results
    /// reflect whatever has been embedded so far.
    ///
    /// # Arguments
    ///
    /// * `query` - The natural-language query text; may be empty when `vector` is supplied
    /// * `vector` - Optional precomputed query vector that bypasses text embedding
    /// * `limit` - An optional Limit over the ranked hits; None lets the server
    ///   apply its default top-k (10)
    /// * `min_score` - Drop hits whose cosine similarity (in [-1, 1]) is below
    ///   this value; None applies no score filter
    /// * `with_pointers` - Whether to include pointers (foreign values) in each
    ///   returned value
    /// * `pointers_metadata` - Whether to include pointer metadata in each
    ///   returned value
    ///
    /// # Examples
    ///
    /// ```rust, ignore
    /// let res = keyspace.semantic_search_get_values("recipes for dinner", None, None, Some(0.3), false, false).await;
    /// let parsed = MontycatResponse::<Vec<serde_json::Value>>::parse_response(res);
    /// // each hit: {"__key__": ..., "__score__": ..., "__value__": ...} — the
    /// // same dunder envelope `lookup_values_where` returns with key_included=true,
    /// // plus the score
    /// ```
    ///
    /// # Errors
    ///
    /// * `MontycatClientError::ClientNoValidInputProvided` - If neither query text nor a valid vector is supplied
    /// * `MontycatClientError::ClientStoreNotSet` - If the store is not set in the engine
    /// * `MontycatClientError::ClientEngineError` - If there is an error with the engine
    ///
    async fn semantic_search_get_values(
        &self,
        query: &str,
        vector: Option<Vec<f32>>,
        limit: Option<Limit>,
        min_score: Option<f32>,
        with_pointers: bool,
        pointers_metadata: bool,
    ) -> Result<Option<Vec<u8>>, MontycatClientError> {
        self.semantic_search_core(
            query,
            vector,
            limit,
            min_score,
            None,
            with_pointers,
            true,
            pointers_metadata,
        )
        .await
    }

    /// Hybrid semantic search returning ranked hits with their values,
    /// restricted by a metadata filter.
    ///
    /// Same ranking as `semantic_search_get_values`, but only items matching
    /// `filters` are considered — a hard AND constraint through the same
    /// criteria stack as `lookup_keys_where` (indexed fields, timestamps,
    /// pointers). Scores stay pure cosine; the filter never boosts, it only
    /// restricts. A filter matching nothing returns `[]`.
    ///
    /// A separate method (not a parameter on `semantic_search_get_values`) so
    /// existing integrations keep their exact signature.
    ///
    /// # Arguments
    ///
    /// * `query` - The natural-language query text; may be empty when `vector` is supplied
    /// * `vector` - Optional precomputed query vector that bypasses text embedding
    /// * `filters` - Metadata criteria, same shape `lookup_keys_where` takes
    /// * `limit` - An optional Limit over the ranked hits; None lets the server
    ///   apply its default top-k (10)
    /// * `min_score` - Drop hits whose cosine similarity (in [-1, 1]) is below
    ///   this value; None applies no score filter
    /// * `with_pointers` - Whether to include pointers (foreign values) in each
    ///   returned value
    /// * `pointers_metadata` - Whether to include pointer metadata in each
    ///   returned value
    ///
    /// # Examples
    ///
    /// ```rust, ignore
    /// let res = keyspace.semantic_search_get_values_where("astronomy", None, serde_json::json!({"category": "space"}), None, None, false, false).await;
    /// let parsed = MontycatResponse::<Vec<serde_json::Value>>::parse_response(res);
    /// // each hit: {"__key__": ..., "__score__": ..., "__value__": ...}
    /// ```
    ///
    /// # Errors
    ///
    /// * `MontycatClientError::ClientNoValidInputProvided` - If neither query text nor a valid vector is supplied, or filters are empty
    /// * `MontycatClientError::ClientStoreNotSet` - If the store is not set in the engine
    /// * `MontycatClientError::ClientEngineError` - If there is an error with the engine
    ///
    async fn semantic_search_get_values_where(
        &self,
        query: &str,
        vector: Option<Vec<f32>>,
        filters: serde_json::Value,
        limit: Option<Limit>,
        min_score: Option<f32>,
        with_pointers: bool,
        pointers_metadata: bool,
    ) -> Result<Option<Vec<u8>>, MontycatClientError> {
        self.semantic_search_core(
            query,
            vector,
            limit,
            min_score,
            Some(filters),
            with_pointers,
            true,
            pointers_metadata,
        )
        .await
    }

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
    /// ```rust, ignore,
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
    ///
    async fn subscribe(
        &self,
        subscription_port: Option<u16>,
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
        let persistent = self.get_persistent();

        let key = {
            if key.is_some() && custom_key.is_some() {
                return Err(MontycatClientError::ClientSelectedBothKeyAndCustomKey);
            }
            key.or(custom_key)
        };

        let port = subscription_port.unwrap_or(engine.port + 1);
        let request_bytes = fulfil_subscription_request(
            store,
            name,
            persistent,
            key,
            &engine.username,
            &engine.password,
        )?;

        // The spawned task outlives this scope, so it owns an `Engine` clone
        // rather than borrowing. Subscriptions are never pooled (contract §5);
        // the clone only carries connection parameters.
        tokio::spawn(async move {
            let _ = send_data(
                &engine,
                request_bytes.as_slice(),
                Some(callback),
                Some(&mut stop_rx),
                Some(port),
            )
            .await;
        });

        Ok(stop_tx)
    }
}

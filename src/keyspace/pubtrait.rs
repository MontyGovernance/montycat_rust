use crate::{
    Limit, MontycatClientError,
    engine::{structure::Engine, utils::send_data},
    request::{
        store_request::structure::StoreRequestClient,
        structure::Req,
        utis::functions::{convert_custom_key, merge_bulk_keys_values, merge_keys},
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
/// - `MontycatClientError::ClientSelectedBothPointersValueAndMetadata`: If both with_pointers and pointers_metadata are true.
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
        let use_tls: bool = engine.use_tls;

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
        if key.is_none() && custom_key.is_none() {
            return Err(MontycatClientError::ClientSelectedBothKeyAndCustomKey);
        }

        if key.is_none() && custom_key.is_none() {
            return Err(MontycatClientError::ClientNoValidInputProvided);
        }

        let mut key: String = key.unwrap_or("").to_owned();

        if with_pointers_metadata && with_pointers {
            return Err(MontycatClientError::ClientSelectedBothPointersValueAndMetadata);
        }

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
        let use_tls: bool = engine.use_tls;
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
        let use_tls: bool = engine.use_tls;
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
            ..Default::default()
        };

        let bytes: Vec<u8> = Req::new_store_command(new_store_req).byte_down()?;
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
        let use_tls: bool = engine.use_tls;
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

    /// Get multiple values by a list of keys
    ///
    /// # Arguments
    ///
    /// * `bulk_keys` - A vector of keys to retrieve values for
    /// * `bulk_custom_keys` - A vector of custom keys to retrieve values for
    /// * `with_pointers` - Whether to include pointers in the returned values
    /// * `key_included` - Whether to include the keys in the returned values
    /// * `with_pointers_metadata` - Whether to include metadata about pointers in the returned values
    /// * `limit` - An optional Limit struct to limit the number of returned values
    /// * `volumes` - An optional vector of volume names to filter the returned values
    /// * `latest_volume` - An optional boolean to indicate whether to only return values from the latest volume
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
        volumes: Option<Vec<String>>,
        latest_volume: Option<bool>,
    ) -> Result<Option<Vec<u8>>, MontycatClientError> {
        if with_pointers && with_pointers_metadata {
            return Err(MontycatClientError::ClientSelectedBothPointersValueAndMetadata);
        }

        let processed_keys: Vec<String> = merge_keys(bulk_keys, bulk_custom_keys).await?;

        let selected_options = [
            !processed_keys.is_empty(),
            volumes.as_ref().is_some_and(|v| !v.is_empty()),
            latest_volume.unwrap_or(false),
        ]
        .iter()
        .filter(|&&x| x)
        .count();

        if selected_options != 1 {
            return Err(MontycatClientError::ClientGenericError(
                "Multiple conflicting options provided. Please provide exactly one of the following: keys, volumes, or latest volume.".into()
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
        let command: String = "get_bulk".to_string();

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

        let new_store_req: StoreRequestClient = StoreRequestClient {
            bulk_keys: processed_keys,
            keyspace: name.to_owned(),
            store,
            persistent,
            distributed,
            command,
            limit_output: limit_map,
            username: engine.username.clone(),
            password: engine.password.clone(),
            with_pointers,
            key_included,
            pointers_metadata: with_pointers_metadata,
            ..Default::default()
        };

        let bytes: Vec<u8> = Req::new_store_command(new_store_req).byte_down()?;
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
        let use_tls: bool = engine.use_tls;
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
            ..Default::default()
        };

        let bytes: Vec<u8> = Req::new_store_command(new_store_req).byte_down()?;
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
        let use_tls: bool = engine.use_tls;
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
        let use_tls: bool = engine.use_tls;

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
        let use_tls: bool = engine.use_tls;

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
        let use_tls: bool = engine.use_tls;
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

    /// Update multiple key-value pairs in the keyspace
    ///
    /// # Arguments
    ///
    /// * `bulk_keys_values` - A vector of HashMaps containing key-value pairs to update
    /// * `bulk_custom_keys_values` - A vector of HashMaps containing custom key-value pairs to update
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
    /// let res: Result<Option<Vec<u8>>, MontycatClientError> = keyspace.update_bulk(bulk_keys_values, bulk_custom_keys_values).await;
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
    ) -> Result<Option<Vec<u8>>, MontycatClientError>
    where
        T: Serialize + Send + 'static,
    {
        if bulk_keys_values.is_empty() && bulk_custom_keys_values.is_empty() {
            return Err(MontycatClientError::ClientNoValidInputProvided);
        }

        let bulk: HashMap<String, String> =
            merge_bulk_keys_values(bulk_keys_values, bulk_custom_keys_values).await?;

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

        let new_store_request: StoreRequestClient = StoreRequestClient {
            bulk_keys_values: bulk,
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
        let use_tls: bool = engine.use_tls;
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
        let use_tls: bool = engine.use_tls;
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

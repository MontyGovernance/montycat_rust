use std::{sync::Arc};
use crate::{
    MontycatClientError,
    engine::{structure::Engine, utils::send_data},
    request::{
        store_request::structure::StoreRequestClient,
        structure::Req,
        utis::functions::{convert_custom_key, merge_keys}
    }, tools::functions::define_type};
use async_trait::async_trait;
use hashbrown::HashMap as BrownHashMap;

#[async_trait]
pub trait Keyspace
where Self: Sized + Send + Sync

{

    fn new(name: &str, engine: Arc<Engine>) -> Arc<Self> where Self: Sized;

    fn get_engine(&self) -> Arc<Engine>;
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
    /// ```rust,no_run
    /// let res: Result<Option<Vec<u8>>, MontycatClientError> = keyspace.remove_keyspace().await;
    /// ```
    ///
    /// # Errors
    /// * `MontycatClientError::StoreNotSet` - If the store is not set in the engine
    /// * `MontycatClientError::EngineError` - If there is an error with the engine
    /// * `MontycatClientError::ValueParsingError` - If there is an error parsing the response
    ///
    async fn remove_keyspace(&self) -> Result<Option<Vec<u8>>, MontycatClientError> {

        let engine: Arc<Engine> = self.get_engine();
        let name: &str = self.get_name();
        let persistent: bool = self.get_persistent();
        let store: String = engine.store.clone().ok_or(MontycatClientError::StoreNotSet)?;

        let vec: Vec<String> = vec![
            "remove-keyspace".into(),
            "store".into(), store,
            "keyspace".into(), name.to_owned(),
            "persistent".into(), if persistent { "y".into() } else { "n".into() },
        ];

        let credentials: Vec<String> = engine.get_credentials();
        let query: Req = Req::new_raw_command(vec, credentials);
        let bytes: Vec<u8> = query.byte_down()?;
        let response: Option<Vec<u8>> = send_data(&engine.host, engine.port, bytes.as_slice(), None, None).await?;

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
    /// ```rust,no_run
    /// let value: Result<Option<Vec<u8>>> = keyspace.get_value(
    ///     Some("298989599989124434694729184587200373152"),
    ///     None, false, false, false
    /// ).await?;
    /// ```
    ///
    /// Or with a custom key
    ///
    /// ```rust,no_run
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
    async fn get_value(&self, key: Option<&str>, custom_key: Option<&str>, with_pointers: bool, key_included: bool, with_pointers_metadata: bool) -> Result<Option<Vec<u8>>, MontycatClientError> {

        if !key.is_some() && !custom_key.is_some() {
            return Err(MontycatClientError::SelectedBothKeyAndCustomKey);
        }

        if key.is_none() && custom_key.is_none() {
            return Err(MontycatClientError::NoValidInputProvided);
        }

        let mut key: String = key.unwrap_or("").to_owned();

        if with_pointers_metadata && with_pointers {
            return Err(MontycatClientError::SelectedBothPointersValueAndMetadata);
        }

        if let Some(custom_key_unwrapped) = custom_key {
            key = convert_custom_key(custom_key_unwrapped);
        }

        let engine: Arc<Engine> = self.get_engine();
        let name: &str = self.get_name();
        let persistent: bool = self.get_persistent();
        let distributed: bool = self.get_distributed();
        let store: String = engine.store.clone().ok_or(MontycatClientError::StoreNotSet)?;
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
        let response: Option<Vec<u8>> = send_data(&engine.host, engine.port, bytes.as_slice(), None, None).await?;

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
    /// ```rust,no_run
    /// let res: Result<Option<Vec<u8>>, MontycatClientError> = keyspace.delete_key(
    ///     "298989599989124434694729184587200373152",
    ///    None
    /// ).await;
    /// ```
    /// Or with a custom key
    /// ```rust,no_run
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
    async fn delete_key(&self, key: &str, custom_key: Option<&str>) -> Result<Option<Vec<u8>>, MontycatClientError> {

        if !key.is_empty() && custom_key.is_some() {
            return Err(MontycatClientError::SelectedBothKeyAndCustomKey);
        }

        let mut key: String = key.to_owned();

        if let Some(custom_key_unwrapped) = custom_key {
            key = convert_custom_key(custom_key_unwrapped);
        }

        let engine: Arc<Engine> = self.get_engine();
        let name: &str = self.get_name();
        let persistent: bool = self.get_persistent();
        let distributed: bool = self.get_distributed();
        let store: String = engine.store.clone().ok_or(MontycatClientError::StoreNotSet)?;
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
        let response: Option<Vec<u8>> = send_data(&engine.host, engine.port, bytes.as_slice(), None, None).await?;

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
    /// ```rust,no_run
    /// let value: Result<Option<Vec<u8>>> = keyspace.list_all_depending_keys(
    ///     "298989599989124434694729184587200373152",
    ///    None
    /// ).await?;
    /// ```
    ///
    /// Or with a custom key
    ///
    /// ```rust,no_run
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
    async fn list_all_depending_keys(&self, key: &str, custom_key: Option<&str>) -> Result<Option<Vec<u8>>, MontycatClientError> {

        if !key.is_empty() && custom_key.is_some() {
            return Err(MontycatClientError::SelectedBothKeyAndCustomKey);
        }

        let mut key: String = key.to_owned();

        if let Some(custom_key_unwrapped) = custom_key {
            key = convert_custom_key(custom_key_unwrapped);
        }

        let engine: Arc<Engine> = self.get_engine();
        let name: &str = self.get_name();
        let persistent: bool = self.get_persistent();
        let distributed: bool = self.get_distributed();
        let store: String = engine.store.clone().ok_or(MontycatClientError::StoreNotSet)?;
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
        let response: Option<Vec<u8>> = send_data(&engine.host, engine.port, bytes.as_slice(), None, None).await?;

        Ok(response)

    }

    /// Get multiple values by a list of keys
    ///
    /// # Arguments
    ///
    /// * `bulk_keys` - A vector of keys to retrieve values for
    ///
    /// # Behavior
    ///
    /// * Sends a request to the server to retrieve values for the provided keys
    /// * Returns the raw response bytes from the server
    ///
    /// # Examples
    ///
    /// ```rust,no_run
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
    ///
    async fn get_bulk(&self, bulk_keys: Option<Vec<String>>, bulk_custom_keys: Option<Vec<String>>, with_pointers: bool, key_included: bool, with_pointers_metadata: bool) -> Result<Option<Vec<u8>>, MontycatClientError> {

        if with_pointers && with_pointers_metadata {
            return Err(MontycatClientError::SelectedBothPointersValueAndMetadata);
        }

        let processed_keys: Vec<String> = merge_keys(bulk_keys, bulk_custom_keys).await?;

        let engine: Arc<Engine> = self.get_engine();
        let name: &str = self.get_name();
        let persistent: bool = self.get_persistent();
        let distributed: bool = self.get_distributed();
        let store: String = engine.store.clone().ok_or(MontycatClientError::StoreNotSet)?;
        let command: String = "get_bulk".to_string();

        let new_store_req: StoreRequestClient = StoreRequestClient {
            bulk_keys: processed_keys,
            keyspace: name.to_owned(),
            store,
            persistent,
            distributed,
            command,
            username: engine.username.clone(),
            password: engine.password.clone(),
            with_pointers,
            key_included,
            pointers_metadata: with_pointers_metadata,
            ..Default::default()
        };

        let bytes: Vec<u8> = Req::new_store_command(new_store_req).byte_down()?;
        let response: Option<Vec<u8>> = send_data(&engine.host, engine.port, bytes.as_slice(), None, None).await?;

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
    /// ```rust,no_run
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
    async fn delete_bulk(&self, bulk_keys: Option<Vec<String>>, bulk_custom_keys: Option<Vec<String>>) -> Result<Option<Vec<u8>>, MontycatClientError> {

        let keys_processed: Vec<String> = merge_keys(bulk_keys, bulk_custom_keys).await?;

        let engine: Arc<Engine> = self.get_engine();
        let name: &str = self.get_name();
        let persistent: bool = self.get_persistent();
        let distributed: bool = self.get_distributed();
        let store: String = engine.store.clone().ok_or(MontycatClientError::StoreNotSet)?;
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
        let response: Option<Vec<u8>> = send_data(&engine.host, engine.port, bytes.as_slice(), None, None).await?;

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
    /// ```rust,no_run
    /// let len: Result<Option<Vec<u8>>, MontycatClientError> = keyspace.get_len().await;
    /// ```
    ///
    /// # Errors
    ///
    /// * `MontycatClientError::StoreNotSet` - If the store is not set in the engine
    /// * `MontycatClientError::EngineError` - If there is an error with the engine
    /// * `MontycatClientError::ValueParsingError` - If there is an error parsing the response
    ///
    async fn get_len(&self) -> Result<Option<Vec<u8>>, MontycatClientError> {

        let engine: Arc<Engine> = self.get_engine();
        let name: &str = self.get_name();
        let persistent: bool = self.get_persistent();
        let distributed: bool = self.get_distributed();
        let store: String = engine.store.clone().ok_or(MontycatClientError::StoreNotSet)?;
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
        let response: Option<Vec<u8>> = send_data(&engine.host, engine.port, bytes.as_slice(), None, None).await?;

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
    /// ```
    /// rust,no_run
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
    async fn enforce_schema(&self, schema_params: (std::collections::HashMap<&str, &str>, &str)) -> Result<Option<Vec<u8>>, MontycatClientError> {

        let (fields, schema_name) = schema_params;

        let mut schema_types: BrownHashMap<String, String> = BrownHashMap::new();

        for (field_name, field_type) in fields.into_iter() {
            let type_def = define_type(field_type)?;
            schema_types.insert(field_name.to_string(), type_def.to_string());
        }

        let schema_types_as_string: String = serde_json::to_string(&schema_types)
            .map_err(|e| MontycatClientError::ValueParsingError(e.to_string()))?;

        let engine: Arc<Engine> = self.get_engine();
        let name: &str = self.get_name();
        let persistent: bool = self.get_persistent();
        let store: String = engine.store.clone().ok_or(MontycatClientError::StoreNotSet)?;

        let vec: Vec<String> = vec![
                "enforce-schema".into(),
                "store".into(), store,
                "keyspace".into(), name.to_owned(),
                "persistent".into(), if persistent { "y".into() } else { "n".into() },
                "schema_name".into(), schema_name.to_string(),
                "schema_content".into(), schema_types_as_string,
            ];

        let credentials: Vec<String> = engine.get_credentials();
        let query: Req = Req::new_raw_command(vec, credentials);
        let bytes: Vec<u8> = query.byte_down()?;
        let response: Option<Vec<u8>> = send_data(&engine.host, engine.port, bytes.as_slice(), None, None).await?;

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
    /// ```rust,no_run
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
    async fn remove_enforced_schema(&self, schema_name: (std::collections::HashMap<&str, &str>, &str)) -> Result<Option<Vec<u8>>, MontycatClientError> {

        let (_fields, schema_name) = schema_name;

        let engine: Arc<Engine> = self.get_engine();
        let name: &str = self.get_name();
        let persistent: bool = self.get_persistent();
        let store: String = engine.store.clone().ok_or(MontycatClientError::StoreNotSet)?;

        let vec: Vec<String> = vec![
                "remove-enforced-schema".into(),
                "store".into(), store,
                "keyspace".into(), name.to_owned(),
                "persistent".into(), if persistent { "y".into() } else { "n".into() },
                "schema_name".into(), schema_name.to_string(),
            ];

        let credentials: Vec<String> = engine.get_credentials();
        let query: Req = Req::new_raw_command(vec, credentials);
        let bytes: Vec<u8> = query.byte_down()?;
        let response: Option<Vec<u8>> = send_data(&engine.host, engine.port, bytes.as_slice(), None, None).await?;

        Ok(response)

    }

}

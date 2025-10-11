use crate::engine::structure::Engine;
use crate::request::store_request::structure::StoreRequestClient;
use crate::request::utis::functions::is_custom_type;
use std::sync::Arc;
use super::super::pubtrait::Keyspace;
use crate::errors::MontycatClientError;
use crate::request::structure::Req;
use crate::engine::utils::send_data;
use crate::traits::RuntimeSchema;
use crate::tools::functions::{process_json_value, process_value};
use serde::Serialize;
use std::any::type_name;

#[derive(Debug, Clone)]
pub struct  InMemoryKeyspace {
    pub name: String,
    pub persistent: bool,
    pub distributed: bool,
    pub engine: Arc<Engine>
}

impl Keyspace for InMemoryKeyspace {

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
            persistent: false,
            distributed: false,
            engine
        })
    }

}

impl InMemoryKeyspace {

    /// Creates a new keyspace in the Montycat database.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Vec<u8>>, MontycatClientError>` - The response from the server or an error.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// let res: Result<Option<Vec<u8>>, MontycatClientError> = keyspace.create_keyspace().await;
    /// ```
    ///
    /// # Errors
    ///
    /// * `MontycatClientError::StoreNotSet` - If the store is not set in the engine.
    /// * `MontycatClientError::EngineError` - If there is an error with the engine.
    /// * `MontycatClientError::ValueParsingError` - If there is an error parsing the response.
    ///
    pub async fn create_keyspace(&self) -> Result<Option<Vec<u8>>, MontycatClientError> {

        let engine: Arc<Engine> = self.get_engine();
        let name: &str = self.get_name();
        let persistent: bool = self.get_persistent();
        let distributed: bool = self.get_distributed();
        let store: String = engine.store.clone().ok_or(MontycatClientError::StoreNotSet)?;

        let vec: Vec<String> = vec![
            "create-keyspace".into(),
            "store".into(), store,
            "keyspace".into(), name.to_owned(),
            "persistent".into(), if persistent { "y".into() } else { "n".into() },
            "distributed".into(), if distributed { "y".into() } else { "n".into() },
        ];

        let credentials: Vec<String> = engine.get_credentials();
        let query: Req = Req::new_raw_command(vec, credentials);
        let bytes: Vec<u8> = query.byte_down()?;
        let response: Option<Vec<u8>> = send_data(&engine.host, engine.port, bytes.as_slice(), None, None).await?;

        return Ok(response)
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
    /// ```rust,no_run
    /// let value = YourType { /* fields */ };
    /// let res: Result<Option<Vec<u8>>, MontycatClientError> = keyspace.insert_value(value, Some(3600)).await;
    /// let parsed = MontycatResponse::<YourType>::parse_response(res);
    /// ```
    ///
    /// # Errors
    ///
    /// * `MontycatClientError::StoreNotSet` - If the store is not set in the engine.
    /// * `MontycatClientError::EngineError` - If there is an error with the engine.
    /// * `MontycatClientError::ValueParsingError` - If there is an error parsing the response.
    ///
    pub async fn insert_value<T>(&self, value: T, expire_sec: Option<usize>) -> Result<Option<Vec<u8>>, MontycatClientError>
    where
        T: Serialize + RuntimeSchema + Send + 'static,
    {
        let engine: Arc<Engine> = self.get_engine();
        let name: &str = self.get_name();
        let persistent: bool = self.get_persistent();
        let distributed: bool = self.get_distributed();
        let store: String = engine.store.clone().ok_or(MontycatClientError::StoreNotSet)?;
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
            expire: expire_sec.map(|sec| sec as u64).unwrap_or(0),
            ..Default::default()
        };

        let bytes: Vec<u8> = Req::new_store_command(new_store_request).byte_down()?;
        let response: Option<Vec<u8>> = send_data(&engine.host, engine.port, bytes.as_slice(), None, None).await?;

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
    /// ```rust,no_run
    /// let res = keyspace.get_keys(Some(vec!["123456789".into()]), None).await;
    /// let parsed = MontycatResponse::<Vec<String>>::parse_response(res);
    /// ```
    ///
    /// # Errors
    ///
    /// * `MontycatClientError::StoreNotSet` - If the store is not set in the engine.
    /// * `MontycatClientError::EngineError` - If there is an error with the engine.
    /// * `MontycatClientError::ValueParsingError` - If there is an error parsing the response.
    ///
    pub async fn get_keys(&self, volumes: Option<Vec<String>>, latest_volume: Option<bool>) -> Result<Option<Vec<u8>>, MontycatClientError> {

        let engine: Arc<Engine> = self.get_engine();
        let name: &str = self.get_name();
        let persistent: bool = self.get_persistent();
        let distributed: bool = self.get_distributed();
        let store: String = engine.store.clone().ok_or(MontycatClientError::StoreNotSet)?;
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
        let response: Option<Vec<u8>> = send_data(&engine.host, engine.port, bytes.as_slice(), None, None).await?;

        Ok(response)

    }

    pub async fn update_value<T>(&self, key: Option<String>, custom_key: Option<String>, value: T, expire_sec: Option<usize>) -> Result<Option<Vec<u8>>, MontycatClientError>
    where
        T: Serialize + Send + 'static,
    {

        if key.is_none() && custom_key.is_none() || (key.is_some() && custom_key.is_some()) {
            return Err(MontycatClientError::NoValidInputProvided);
        }

        let key: String = key.or(custom_key).ok_or(MontycatClientError::NoValidInputProvided)?;

        let engine: Arc<Engine> = self.get_engine();
        let name: &str = self.get_name();
        let persistent: bool = self.get_persistent();
        let distributed: bool = self.get_distributed();
        let store: String = engine.store.clone().ok_or(MontycatClientError::StoreNotSet)?;
        let command: String = "update_value".to_string();
        let value_to_send: String = process_json_value(&value)?;

        println!("Value to send in update_value: {}", value_to_send);

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
        let response: Option<Vec<u8>> = send_data(&engine.host, engine.port, bytes.as_slice(), None, None).await?;

        Ok(response)

    }


}


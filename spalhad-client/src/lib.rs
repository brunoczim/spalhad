use std::hash::Hash;

use anyhow::Result;
use reqwest::StatusCode;
use serde::{Serialize, de::DeserializeOwned};
use spalhad_spec::{GetResponse, Key, PutRequest, PutResponse};

#[derive(Debug, Clone)]
pub struct Client {
    base_url: String,
    inner: reqwest::Client,
}

impl Default for Client {
    fn default() -> Self {
        Self::new("http://localhost:3000/kv")
    }
}

impl Client {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self { inner: reqwest::Client::new(), base_url: base_url.into() }
    }

    pub async fn get<K, V>(&self, key_data: K) -> Result<Option<V>>
    where
        K: Hash + Eq,
        V: DeserializeOwned,
    {
        self.get_raw(Key::hashing(key_data)).await
    }

    pub async fn put<K, V>(&self, key_data: K, value: V) -> Result<bool>
    where
        K: Hash + Eq,
        V: Serialize,
    {
        self.put_raw(Key::hashing(key_data), value).await
    }

    pub async fn get_raw<V>(&self, key: Key) -> Result<Option<V>>
    where
        V: DeserializeOwned,
    {
        let url = format!("{}/{}", self.base_url, key);
        let request = self.inner.get(url).build()?;
        let response = self.inner.execute(request).await?;
        if response.status() == StatusCode::NOT_FOUND {
            Ok(None)
        } else {
            let get_response: GetResponse<V> = response.json().await?;
            Ok(Some(get_response.value))
        }
    }

    pub async fn put_raw<V>(&self, key: Key, value: V) -> Result<bool>
    where
        V: Serialize,
    {
        let url = format!("{}/{}", self.base_url, key);
        let body = PutRequest { value };
        let request = self.inner.post(url).json(&body).build()?;
        let response = self.inner.execute(request).await?;
        let put_response: PutResponse = response.json().await?;
        Ok(put_response.new)
    }
}

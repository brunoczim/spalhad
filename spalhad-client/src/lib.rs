use std::{hash::Hash, sync::Arc};

use anyhow::{Result, bail};
use reqwest::StatusCode;
use serde::{Serialize, de::DeserializeOwned};
use spalhad_spec::{
    cluster::{RunId, RunIdResponse},
    kv::{GetResponse, Key, PutRequest, PutResponse},
};

#[derive(Debug)]
struct Inner {
    base_url: Box<str>,
    http_impl: reqwest::Client,
}

#[derive(Debug, Clone)]
pub struct Client {
    inner: Arc<Inner>,
}

impl Default for Client {
    fn default() -> Self {
        Self::new("http://localhost:3000/kv")
    }
}

impl Client {
    pub fn new(base_url: impl AsRef<str>) -> Self {
        Self {
            inner: Arc::new(Inner {
                base_url: Box::from(base_url.as_ref()),
                http_impl: reqwest::Client::new(),
            }),
        }
    }

    pub fn base_url(&self) -> &str {
        &self.inner.base_url
    }

    fn http_impl(&self) -> &reqwest::Client {
        &self.inner.http_impl
    }

    pub async fn run_id(&self) -> Result<RunId> {
        let url = format!("{}/sync/run_id", self.base_url());
        let request = self.http_impl().get(url).build()?;
        let response = self.http_impl().execute(request).await?;
        if response.status() != StatusCode::OK {
            bail!(
                "Request failed with status {}, body {}",
                response.status(),
                response.text().await?,
            )
        }
        let run_id_response: RunIdResponse = response.json().await?;
        Ok(run_id_response.run_id)
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
        let url = format!("{}/kv/{}", self.base_url(), key);
        let request = self.http_impl().get(url).build()?;
        let response = self.http_impl().execute(request).await?;
        if response.status() == StatusCode::NOT_FOUND {
            Ok(None)
        } else if response.status() != StatusCode::OK {
            bail!(
                "Request failed with status {}, body {}",
                response.status(),
                response.text().await?,
            )
        } else {
            let get_response: GetResponse<V> = response.json().await?;
            Ok(Some(get_response.value))
        }
    }

    pub async fn put_raw<V>(&self, key: Key, value: V) -> Result<bool>
    where
        V: Serialize,
    {
        let url = format!("{}/kv/{}", self.base_url(), key);
        let body = PutRequest { value };
        let request = self.http_impl().post(url).json(&body).build()?;
        let response = self.http_impl().execute(request).await?;
        if response.status() != StatusCode::OK {
            bail!(
                "Request failed with status {}, body {}",
                response.status(),
                response.text().await?,
            )
        }
        let put_response: PutResponse = response.json().await?;
        Ok(put_response.new)
    }
}

use std::{hash::Hash, sync::Arc, time::Duration};

use anyhow::Result;
use reqwest::StatusCode;
use serde::{Serialize, de::DeserializeOwned};
use spalhad_spec::{
    cluster::{
        ActivateRequest,
        ActivateResponse,
        IsActiveResponse,
        RunId,
        RunIdResponse,
    },
    kv::{GetResponse, Key, PutRequest, PutResponse},
};
use thiserror::Error;

#[derive(Debug)]
struct Inner {
    base_url: Box<str>,
    http_impl: reqwest::Client,
}

#[derive(Debug, Clone, Error)]
#[error(
    "Request failed with status {} and body {}",
    .status_code.as_u16(),
    .text_body,
)]
pub struct ResponseError {
    pub status_code: StatusCode,
    pub text_body: String,
    pub json_body: Option<spalhad_spec::Error>,
}

impl ResponseError {
    async fn new(response: reqwest::Response) -> Result<Self> {
        let status_code = response.status();
        let text_body = response.text().await?;
        let json_body = serde_json::from_str(&text_body).ok();
        Ok(Self { status_code, text_body, json_body })
    }

    async fn bail<T>(response: reqwest::Response) -> Result<T> {
        Err(Self::new(response).await?)?
    }
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
        Self::with_timeout(base_url, Duration::from_secs(90))
            .expect("bad default timeout")
    }

    pub fn with_timeout(
        base_url: impl AsRef<str>,
        timeout: Duration,
    ) -> Result<Self> {
        Ok(Self {
            inner: Arc::new(Inner {
                base_url: Box::from(base_url.as_ref()),
                http_impl: reqwest::Client::builder()
                    .timeout(timeout)
                    .build()?,
            }),
        })
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
            ResponseError::bail(response).await
        } else {
            let run_id_response: RunIdResponse = response.json().await?;
            Ok(run_id_response.run_id)
        }
    }

    pub async fn activate(&self, run_id: RunId) -> Result<ActivateResponse> {
        let url = format!("{}/sync/activate", self.base_url());
        let body = ActivateRequest { run_id };
        let request = self.http_impl().post(url).json(&body).build()?;
        let response = self.http_impl().execute(request).await?;
        if response.status() == StatusCode::OK {
            let activate_response: ActivateResponse = response.json().await?;
            Ok(activate_response)
        } else {
            ResponseError::bail(response).await
        }
    }

    pub async fn is_active(&self) -> Result<ActivateResponse> {
        let url = format!("{}/sync/active", self.base_url(),);
        let request = self.http_impl().get(url).build()?;
        let response = self.http_impl().execute(request).await?;
        if response.status() == StatusCode::OK {
            let activate_response: IsActiveResponse = response.json().await?;
            Ok(activate_response)
        } else {
            ResponseError::bail(response).await
        }
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
            let error = ResponseError::new(response).await?;
            if error.json_body.is_some() { Ok(None) } else { Err(error.into()) }
        } else if response.status() == StatusCode::OK {
            let get_response: GetResponse<V> = response.json().await?;
            Ok(Some(get_response.value))
        } else {
            ResponseError::bail(response).await
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
        if response.status() == StatusCode::OK {
            let put_response: PutResponse = response.json().await?;
            Ok(put_response.new)
        } else {
            ResponseError::bail(response).await
        }
    }

    pub async fn get_internal<V>(&self, key: Key) -> Result<Option<V>>
    where
        V: DeserializeOwned,
    {
        let url = format!("{}/internal-kv/{}", self.base_url(), key);
        let request = self.http_impl().get(url).build()?;
        let response = self.http_impl().execute(request).await?;
        if response.status() == StatusCode::NOT_FOUND {
            let error = ResponseError::new(response).await?;
            if error.json_body.is_some() { Ok(None) } else { Err(error.into()) }
        } else if response.status() == StatusCode::OK {
            let get_response: GetResponse<V> = response.json().await?;
            Ok(Some(get_response.value))
        } else {
            ResponseError::bail(response).await
        }
    }

    pub async fn put_internal<V>(&self, key: Key, value: V) -> Result<bool>
    where
        V: Serialize,
    {
        let url = format!("{}/internal-kv/{}", self.base_url(), key);
        let body = PutRequest { value };
        let request = self.http_impl().post(url).json(&body).build()?;
        let response = self.http_impl().execute(request).await?;
        if response.status() == StatusCode::OK {
            let put_response: PutResponse = response.json().await?;
            Ok(put_response.new)
        } else {
            ResponseError::bail(response).await
        }
    }
}

use serde::{Deserialize, Serialize};

pub use key::Key;

mod key;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Error {
    pub trace: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PutRequest<V> {
    pub value: V,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetResponse<V> {
    pub value: V,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PutResponse {
    pub new: bool,
}

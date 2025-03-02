use std::{fmt, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Visitor};
use thiserror::Error;

use crate::hex;

#[derive(Debug, Error)]
#[error("failed to generate node ID")]
pub struct GenNodeIdError {
    #[source]
    inner: getrandom::Error,
}

#[derive(Debug, Error)]
#[error("string is not a valid hexadecimal key")]
pub struct ParseNodeIdError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId {
    bytes: [u8; Self::SIZE],
}

impl NodeId {
    pub const SIZE: usize = 16;

    pub fn generate() -> Result<Self, GenNodeIdError> {
        let mut bytes = [0; Self::SIZE];
        getrandom::fill(&mut bytes)
            .map_err(|inner| GenNodeIdError { inner })?;
        Ok(Self { bytes })
    }

    pub fn from_bytes(bytes: [u8; Self::SIZE]) -> Self {
        Self { bytes }
    }

    pub fn as_bytes(&self) -> &[u8; Self::SIZE] {
        &self.bytes
    }

    pub fn into_bytes(self) -> [u8; Self::SIZE] {
        self.bytes
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        hex::render(&self.bytes, f)
    }
}

impl FromStr for NodeId {
    type Err = ParseNodeIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut bytes = [0; Self::SIZE];
        if hex::parse(s, &mut bytes) {
            Ok(Self { bytes })
        } else {
            Err(ParseNodeIdError)
        }
    }
}

impl Serialize for NodeId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for NodeId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct NodeIdVisitor;

        impl Visitor<'_> for NodeIdVisitor {
            type Value = NodeId;

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                v.parse().map_err(E::custom)
            }

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(
                    formatter,
                    "expected {} hexadecimal characters",
                    NodeId::SIZE * 2
                )
            }
        }

        deserializer.deserialize_str(NodeIdVisitor)
    }
}

use std::{fmt, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Visitor};
use thiserror::Error;

use crate::hex;

#[derive(Debug, Error)]
#[error("failed to generate run ID")]
pub struct GenRunIdError {
    #[source]
    inner: getrandom::Error,
}

#[derive(Debug, Error)]
#[error("string is not a valid hexadecimal key")]
pub struct ParseRunIdError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RunId {
    bytes: [u8; Self::SIZE],
}

impl RunId {
    const SIZE: usize = 16;

    pub fn generate() -> Result<Self, GenRunIdError> {
        let mut bytes = [0; Self::SIZE];
        getrandom::fill(&mut bytes).map_err(|inner| GenRunIdError { inner })?;
        Ok(Self { bytes })
    }
}

impl fmt::Display for RunId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        hex::render(&self.bytes, f)
    }
}

impl FromStr for RunId {
    type Err = ParseRunIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut bytes = [0; Self::SIZE];
        if hex::parse(s, &mut bytes) {
            Ok(Self { bytes })
        } else {
            Err(ParseRunIdError)
        }
    }
}

impl Serialize for RunId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for RunId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct RunIdVisitor;

        impl Visitor<'_> for RunIdVisitor {
            type Value = RunId;

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
                    RunId::SIZE * 2
                )
            }
        }

        deserializer.deserialize_str(RunIdVisitor)
    }
}

use std::{
    fmt,
    hash::{BuildHasher, Hash, Hasher},
    str::FromStr,
};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Visitor};
use sha3::{Digest, Sha3_256};
use thiserror::Error;

use crate::hex;

#[derive(Debug, Error)]
#[error("string is not a valid hexadecimal key")]
pub struct ParseKeyError;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Key {
    bytes: [u8; Self::SIZE],
}

impl Key {
    pub const SIZE: usize = 32;

    pub fn hashing<T>(key_data: T) -> Self
    where
        T: Hash + Eq,
    {
        struct KeyHasherBuilder;

        impl BuildHasher for KeyHasherBuilder {
            type Hasher = KeyHasher;

            fn build_hasher(&self) -> Self::Hasher {
                KeyHasher::default()
            }
        }

        #[derive(Default)]
        struct KeyHasher {
            inner: Sha3_256,
        }

        impl Hasher for KeyHasher {
            fn write(&mut self, bytes: &[u8]) {
                self.inner.update(bytes);
            }

            fn finish(&self) -> u64 {
                panic!("key hasher is incompatible with 64-bit hash")
            }
        }

        let mut hasher = KeyHasherBuilder.build_hasher();
        key_data.hash(&mut hasher);
        let mut bytes = [0; Self::SIZE];
        bytes[..].copy_from_slice(hasher.inner.finalize().as_slice());
        Self::from_bytes(bytes)
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

    pub fn partition(&self, nodes: usize) -> usize {
        let total = nodes.to_le_bytes();
        let mut divisor = [0; Key::SIZE];
        divisor[.. total.len()].copy_from_slice(&total);
        let mut quotient = [0; Key::SIZE];
        let mut remainder = [0; Key::SIZE];
        hex::divide_le(&self.bytes, &divisor, &mut quotient, &mut remainder);

        const INDEX_SIZE: usize = (usize::BITS as usize) / 8;
        let mut index_bytes = [0; INDEX_SIZE];
        index_bytes[..].copy_from_slice(&remainder[.. INDEX_SIZE]);
        let index = usize::from_le_bytes(index_bytes);
        index
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        hex::render(&self.bytes, f)
    }
}

impl FromStr for Key {
    type Err = ParseKeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut bytes = [0; Self::SIZE];
        if hex::parse(s, &mut bytes) {
            Ok(Self { bytes })
        } else {
            Err(ParseKeyError)
        }
    }
}

impl Serialize for Key {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Key {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct KeyVisitor;

        impl Visitor<'_> for KeyVisitor {
            type Value = Key;

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
                    Key::SIZE * 2
                )
            }
        }

        deserializer.deserialize_str(KeyVisitor)
    }
}

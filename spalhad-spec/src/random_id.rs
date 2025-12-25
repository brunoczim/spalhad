use std::{cell::RefCell, fmt, marker::PhantomData, str::FromStr};

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Visitor};
use thiserror::Error;

use crate::hex;

#[derive(Debug, Error)]
#[error("string is not a valid hexadecimal key")]
pub struct ParseRandomIdError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RandomId<const N: usize> {
    bytes: [u8; N],
}

impl<const N: usize> RandomId<{ N }> {
    pub const SIZE: usize = N;

    pub fn generate() -> Self {
        thread_local! {
            static PRNG: RefCell<ChaCha20Rng> =
                RefCell::new(SeedableRng::from_os_rng());
        }

        let mut bytes = [0; N];
        PRNG.with_borrow_mut(|rand| rand.fill(&mut bytes));

        Self { bytes }
    }

    pub fn from_bytes(bytes: [u8; N]) -> Self {
        Self { bytes }
    }

    pub fn as_bytes(&self) -> &[u8; N] {
        &self.bytes
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.bytes[..]
    }

    pub fn into_bytes(self) -> [u8; N] {
        self.bytes
    }
}

impl<const N: usize> AsRef<[u8; N]> for RandomId<{ N }> {
    fn as_ref(&self) -> &[u8; N] {
        self.as_bytes()
    }
}

impl<const N: usize> AsRef<[u8]> for RandomId<{ N }> {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl<const N: usize> From<[u8; N]> for RandomId<{ N }> {
    fn from(bytes: [u8; N]) -> Self {
        Self::from_bytes(bytes)
    }
}

impl<const N: usize> From<RandomId<{ N }>> for [u8; N] {
    fn from(id: RandomId<{ N }>) -> Self {
        id.into_bytes()
    }
}

impl<const N: usize> fmt::Display for RandomId<{ N }> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        hex::render(&self.bytes, f)
    }
}

impl<const N: usize> FromStr for RandomId<{ N }> {
    type Err = ParseRandomIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut bytes = [0; N];
        if hex::parse(s, &mut bytes) {
            Ok(Self { bytes })
        } else {
            Err(ParseRandomIdError)
        }
    }
}

impl<const N: usize> Serialize for RandomId<{ N }> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de, const N: usize> Deserialize<'de> for RandomId<{ N }> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(RandomIdVisitor(PhantomData))
    }
}

struct RandomIdVisitor<const N: usize>(PhantomData<RandomId<{ N }>>);

impl<const N: usize> Visitor<'_> for RandomIdVisitor<{ N }> {
    type Value = RandomId<{ N }>;

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        v.parse().map_err(E::custom)
    }

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "expected {} hexadecimal characters", N * 2)
    }
}

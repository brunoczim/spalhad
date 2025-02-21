use std::{
    fmt,
    hash::{BuildHasher, Hash, Hasher},
};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Visitor};
use sha3::{Digest, Sha3_256};

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

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn into_bytes(self) -> [u8; Self::SIZE] {
        self.bytes
    }

    pub fn divide_le(
        &self,
        divisor: &[u8; Self::SIZE],
        quotient: &mut [u8; Self::SIZE],
        remainder: &mut [u8; Self::SIZE],
    ) {
        for byte in &mut *remainder {
            *byte = 0;
        }
        for byte in &mut *quotient {
            *byte = 0;
        }
        for i in (0 .. Self::SIZE * 8).rev() {
            let mut carry = 0;
            for byte in &mut *remainder {
                let new_byte = (*byte << 1) | carry;
                carry = *byte & 1;
                *byte = new_byte;
            }
            let mut carry = 0;
            for byte in &mut *quotient {
                let new_byte = (*byte << 1) | carry;
                carry = *byte & 1;
                *byte = new_byte;
            }
            remainder[0] |= self.bytes[i / 8] >> (i % 8);
            if (*remainder).into_iter().rev().ge((*divisor).into_iter().rev()) {
                quotient[0] |= 1;
                let mut borrow = 0;
                for (dest, src) in
                    remainder.iter_mut().zip((*divisor).into_iter())
                {
                    let (byte, borrow_a) = dest.overflowing_sub(src);
                    let (byte, borrow_b) = byte.overflowing_sub(borrow);
                    *dest = byte;
                    borrow = u8::from(borrow_a | borrow_b);
                }
            }
        }
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.bytes {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
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
                let mut bytes = [0; Key::SIZE];
                let mut byte_iter = bytes.iter_mut();
                let mut chars = v.chars();
                loop {
                    let Some(byte) = byte_iter.next() else {
                        return if chars.next().is_some() {
                            Err(E::custom("too many characters"))
                        } else {
                            Ok(Key::from_bytes(bytes))
                        };
                    };
                    let Some(low) = chars.next() else {
                        Err(E::custom(
                            "expected at least low hexadecimal digit",
                        ))?
                    };
                    let Some(high) = chars.next() else {
                        Err(E::custom(
                            "expected at least high hexadecimal digit",
                        ))?
                    };
                    let Some(low) = low.to_digit(16) else {
                        Err(E::custom("invalid hexadecimal digit"))?
                    };
                    let Some(high) = high.to_digit(16) else {
                        Err(E::custom("invalid hexadecimal digit"))?
                    };
                    *byte = (low | (high << 4)) as u8;
                }
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

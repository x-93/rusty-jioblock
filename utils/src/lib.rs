pub mod hex {
    /// Small ToHex trait used by consensus core for debugging
    pub trait ToHex {
        fn to_hex(&self) -> String;
    }

    impl ToHex for Vec<u8> {
        fn to_hex(&self) -> String {
            hex::encode(self)
        }
    }

    impl ToHex for [u8] {
        fn to_hex(&self) -> String {
            hex::encode(self)
        }
    }
}

pub mod mem_size {
    /// Trait to estimate memory usage
    pub trait MemSizeEstimator {
        fn estimate_mem_bytes(&self) -> usize {
            // Use size_of_val to support unsized `Self` behind references and provide a conservative estimate.
            std::mem::size_of_val(self)
        }
    }
}

pub use serde_bytes;

pub mod serde_bytes_fixed_ref {
    use serde::{Deserializer, Serializer, Deserialize};

    // We expose generic serialize/deserialize helpers that expect a type that can convert to/from bytes.
    // consensus_core uses this module on `jio_hashes::Hash` fields; the local `jio_hashes` crate derives serde
    // so we attempt to (de)serialize the bytes directly through serde_bytes.

    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: AsRef<[u8]> + ?Sized,
    {
        // Serialize as bytes
        serializer.serialize_bytes(value.as_ref())
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: From<Vec<u8>>,
    {
        let bytes: Vec<u8> = serde_bytes::ByteBuf::deserialize(deserializer)?.into_vec();
        Ok(T::from(bytes))
    }
}

// Provide a tiny placeholder module so other crates can reference types in tests if needed.
// (Not part of public API; used to satisfy references in generated modules.)
#[allow(dead_code)]
mod super_placeholder_hash {
    // This is intentionally empty; real Hash lives in `jio_hashes` crate.
    pub struct HashPlaceholder;
}

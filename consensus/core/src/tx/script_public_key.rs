use alloc::borrow::Cow;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use jio_utils::hex::ToHex;
use smallvec::SmallVec;
use std::{
    collections::HashSet,
    str::{self, FromStr},
};
use wasm_bindgen::prelude::*;
use workflow_wasm::prelude::*;
use serde::de::{Error, Visitor};
use serde::{Deserializer, Serializer};
use std::fmt;
use faster_hex;

/// Size of the underlying script vector of a script.
pub const SCRIPT_VECTOR_SIZE: usize = 36;

/// Used as the underlying type for script public key data, optimized for the common p2pk script size (34).
pub type ScriptVec = SmallVec<[u8; SCRIPT_VECTOR_SIZE]>;

/// Represents the ScriptPublicKey Version
pub type ScriptPublicKeyVersion = u16;

/// Alias the `smallvec!` macro to ease maintenance
pub use smallvec::smallvec as scriptvec;
use wasm_bindgen::prelude::wasm_bindgen;

//Represents a Set of [`ScriptPublicKey`]s
pub type ScriptPublicKeys = HashSet<ScriptPublicKey>;

#[wasm_bindgen(typescript_custom_section)]
const TS_SCRIPT_PUBLIC_KEY: &'static str = r#"
/**
 * Interface defines the structure of a Script Public Key.
 * 
 * @category Consensus
 */
export interface IScriptPublicKey {
    version : number;
    script: HexString;
}
"#;

/// Represents a Jiopad ScriptPublicKey
/// @category Consensus
#[derive(Default, PartialEq, Eq, Clone, Hash, PartialOrd, Ord)]
#[wasm_bindgen(inspectable)]
pub struct ScriptPublicKey {
    pub version: ScriptPublicKeyVersion,
    pub(super) script: ScriptVec, // Kept private to preserve read-only semantics
}

impl std::fmt::Debug for ScriptPublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScriptPublicKey").field("version", &self.version).field("script", &self.script.to_hex()).finish()
    }
}

impl std::fmt::Display for ScriptPublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.script.to_hex())
    }
}

impl ScriptPublicKey {
    pub fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, hex::FromHexError> {
        let bytes = hex::decode(hex)?;
        Ok(ScriptPublicKey {
            version: 0,
            script: bytes.into(),
        })
    }
}

#[derive(Default, Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Hash)]
#[serde(rename_all = "camelCase")]
#[serde(rename = "ScriptPublicKey")]
struct ScriptPublicKeyInternal<'a> {
    version: ScriptPublicKeyVersion,
    script: &'a [u8],
}

impl Serialize for ScriptPublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            let mut hex = vec![0u8; self.script.len() * 2 + 4];
            faster_hex::hex_encode(&self.version.to_be_bytes(), &mut hex).map_err(serde::ser::Error::custom)?;
            faster_hex::hex_encode(&self.script, &mut hex[4..]).map_err(serde::ser::Error::custom)?;
            serializer.serialize_str(unsafe { str::from_utf8_unchecked(&hex) })
        } else {
            ScriptPublicKeyInternal { version: self.version, script: &self.script }.serialize(serializer)
        }
    }
}

impl FromStr for ScriptPublicKey {
    type Err = faster_hex::Error;
    fn from_str(hex_str: &str) -> Result<Self, Self::Err> {
        let hex_len = hex_str.len();
        if hex_len < 4 {
            return Err(faster_hex::Error::InvalidLength(hex_len));
        }
        let mut bytes = vec![0u8; hex_len / 2];
        faster_hex::hex_decode(hex_str.as_bytes(), bytes.as_mut_slice())?;
        let version = u16::from_be_bytes(bytes[0..2].try_into().unwrap());
        Ok(Self { version, script: SmallVec::from_slice(&bytes[2..]) })
    }
}

impl ScriptPublicKey {
    pub fn new(version: ScriptPublicKeyVersion, script: ScriptVec) -> Self {
        Self { version, script }
    }

    pub fn from_vec(version: ScriptPublicKeyVersion, script: Vec<u8>) -> Self {
        Self { version, script: ScriptVec::from_vec(script) }
    }

    pub fn version(&self) -> ScriptPublicKeyVersion {
        self.version
    }

    pub fn script(&self) -> &[u8] {
        &self.script
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "ScriptPublicKey | HexString")]
    pub type ScriptPublicKeyT;
}

#[wasm_bindgen]
impl ScriptPublicKey {
    #[wasm_bindgen(constructor)]
    pub fn constructor(version: u16, script: JsValue) -> Result<ScriptPublicKey, JsError> {
        let script = script.try_as_vec_u8()?;
        Ok(ScriptPublicKey::new(version, script.into()))
    }

    #[wasm_bindgen(getter = script)]
    pub fn script_as_hex(&self) -> String {
        self.script.to_hex()
    }
}

//
// Borsh serializers need to be manually implemented for `ScriptPublicKey` since
// smallvec does not currently support Borsh
//

impl BorshSerialize for ScriptPublicKey {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        borsh::BorshSerialize::serialize(&self.version, writer)?;
        // Vectors and slices are all serialized internally the same way
        borsh::BorshSerialize::serialize(&self.script.as_slice(), writer)?;
        Ok(())
    }
}

impl BorshDeserialize for ScriptPublicKey {
    fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
        let version: ScriptPublicKeyVersion = BorshDeserialize::deserialize(buf)?;
        let script: Vec<u8> = BorshDeserialize::deserialize(buf)?;
        Ok(Self::from_vec(version, script))
    }
}

impl<'de> Deserialize<'de> for ScriptPublicKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Default)]
        struct ScriptPublicKeyVisitor {
        }

        impl<'de> Visitor<'de> for ScriptPublicKeyVisitor {
            type Value = ScriptPublicKey;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a ScriptPublicKey")
            }

            #[cfg(target_arch = "wasm32")]
            fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
            where 
                E: serde::de::Error,
            {
                self.visit_u32(v as u32)
            }

            #[cfg(target_arch = "wasm32")]
            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_u32(v as u32)
            }

            #[cfg(target_arch = "wasm32")]
            fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_u32(v as u32)
            }

            #[cfg(target_arch = "wasm32")]
            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_u32(v as u32)
            }

            #[cfg(target_arch = "wasm32")]
            fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                use wasm_bindgen::convert::RefFromWasmAbi;
                let instance_ref = unsafe { Self::Value::ref_from_abi(v) }; // todo add checks for safecast
                Ok(instance_ref.clone())
            }

            #[cfg(target_arch = "wasm32")]
            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_u32(v as u32)
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(ScriptPublicKey::from_str(v).map_err(Error::custom)?)
            }

            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                self.visit_str(v)
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: Error,
            {
                self.visit_str(&v)
            }

            fn visit_map<A>(self, mut access: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                #[derive(Deserialize, Copy, Clone)]
                #[serde(field_identifier, rename_all = "lowercase")]
                enum Field {
                    Version,
                    Script,
                }

                #[derive(Debug, Clone, Deserialize)]
                #[serde(untagged)]
                pub enum Value<'a> {
                    U16(u16),
                    #[serde(borrow)]
                    String(Cow<'a, str>),
                }

                impl From<Value<'_>> for u16 {
                    fn from(value: Value<'_>) -> Self {
                        let Value::U16(v) = value else { panic!("unexpected conversion: {value:?}") };
                        v
                    }
                }

                impl TryFrom<Value<'_>> for Vec<u8> {
                    type Error = faster_hex::Error;

                    fn try_from(value: Value) -> Result<Self, Self::Error> {
                        match value {
                            Value::U16(_) => {
                                panic!("unexpected conversion: {value:?}")
                            }
                            Value::String(script) => {
                                let mut script_bytes = vec![0u8; script.len() / 2];
                                faster_hex::hex_decode(script.as_bytes(), script_bytes.as_mut_slice())?;

                                Ok(script_bytes)
                            }
                        }
                    }
                }

                let mut version: Option<u16> = None;
                let mut script: Option<Vec<u8>> = None;

                while let Some((key, value)) = access.next_entry::<Field, Value>()? {
                    match key {
                        Field::Version => {
                            version = Some(value.into());
                        }
                        Field::Script => script = Some(value.try_into().map_err(Error::custom)?),
                    }
                    if version.is_some() && script.is_some() {
                        break;
                    }
                }

                let (version, script) = match (version, script) {
                    (Some(version), Some(script)) => Ok((version, script)),
                    (None, _) => Err(serde::de::Error::missing_field("version")),
                    (_, None) => Err(serde::de::Error::missing_field("script")),
                }?;

                Ok(ScriptPublicKey::from_vec(version, script))
            }
        }

        if deserializer.is_human_readable() {
            deserializer.deserialize_any(ScriptPublicKeyVisitor::default())
        } else {
            #[derive(Deserialize)]
            struct ScriptPublicKeyInternal {
                version: u16,
                script: Vec<u8>,
            }

            ScriptPublicKeyInternal::deserialize(deserializer)
                .map(|ScriptPublicKeyInternal { script, version }| {
                    Self { version, script: SmallVec::from_slice(&script) }
                })
        }
    }
}

#[wasm_bindgen]
impl ScriptPublicKey {
    #[wasm_bindgen(constructor)]
    pub fn wasm_new(version: u8, script: &[u8]) -> Self {
        Self::from_vec(version.into(), script.to_vec())
    }

    #[wasm_bindgen(js_name = fromHex)]
    pub fn from_hex_js(hex: &str) -> Result<ScriptPublicKey, JsValue> {
        Self::from_hex(hex).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen(js_name = toString)]
    pub fn to_string_js(&self) -> String {
        self.to_string()
    }

    #[wasm_bindgen(js_name = fromJsValue)]
    pub fn from_js_value(value: &JsValue) -> Result<Self, JsValue> {
        if let Some(hex_str) = value.as_string() {
            Self::from_hex(&hex_str).map_err(|e| JsValue::from_str(&e.to_string()))
        } else if value.is_object() {
            let obj = js_sys::Object::from(value.clone());
            let version: u8 = js_sys::Reflect::get(&obj, &JsValue::from_str("version"))
                .map_err(|e| JsValue::from_str(&format!("Failed to get version: {:?}", e)))?
                .as_f64()
                .ok_or_else(|| JsValue::from_str("version must be a number"))?
                as u8;

            let script = js_sys::Uint8Array::new(&js_sys::Reflect::get(&obj, &JsValue::from_str("script"))
                .map_err(|e| JsValue::from_str(&format!("Failed to get script: {:?}", e)))?)
                .to_vec();

            Ok(Self::from_vec(version.into(), script))
        } else {
            Err(JsValue::from_str("Expected hex string or {version, script} object"))
        }
    }
}

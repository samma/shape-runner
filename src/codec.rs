use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};

pub trait ShapeCodec {
    fn encode<T: Serialize>(&self, value: &T) -> Result<Vec<u8>>;
    fn decode<T: DeserializeOwned>(&self, data: &[u8]) -> Result<T>;
}

// MessagePack codec (fast internal format)
pub struct MsgPackCodec;

impl ShapeCodec for MsgPackCodec {
    fn encode<T: Serialize>(&self, value: &T) -> Result<Vec<u8>> {
        let buf = rmp_serde::to_vec_named(value)?;
        Ok(buf)
    }

    fn decode<T: DeserializeOwned>(&self, data: &[u8]) -> Result<T> {
        let value = rmp_serde::from_slice(data)?;
        Ok(value)
    }
}

// Optional JSON codec for debugging
pub struct JsonCodec;

impl ShapeCodec for JsonCodec {
    fn encode<T: Serialize>(&self, value: &T) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec(value)?)
    }

    fn decode<T: DeserializeOwned>(&self, data: &[u8]) -> Result<T> {
        Ok(serde_json::from_slice(data)?)
    }
}

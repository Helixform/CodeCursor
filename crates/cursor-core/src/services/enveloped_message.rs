use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct EnvelopedMessage {
    pub data: Vec<u8>,
    pub flags: u8,
}

impl EnvelopedMessage {
    pub fn new<T>(data: T, flags: u8) -> Self
    where
        T: AsRef<[u8]>,
    {
        Self {
            data: data.as_ref().to_vec(),
            flags,
        }
    }

    pub fn new_with_serializable<T>(data: &T, flags: u8) -> Result<Self>
    where
        T: ?Sized + Serialize,
    {
        let string = serde_json::to_string(data)?;
        Ok(Self::new(string, flags))
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut result = vec![self.flags];
        result.extend_from_slice(&(self.data.len() as u32).to_be_bytes());
        result.extend_from_slice(&self.data);
        result
    }

    pub fn decode(data: Vec<u8>) -> Result<Self> {
        let length = data.len();
        if length < 5 {
            return Err(anyhow!("invalid data length"));
        }
        let flags = data[0];
        let data_length = u32::from_be_bytes(data[1..5].try_into()?) as usize;
        if length != data_length + 5 {
            return Err(anyhow!("invalid data length"));
        }
        Ok(Self {
            data: data[5..].to_vec(),
            flags,
        })
    }

    pub fn utf8_string(&self) -> Result<String> {
        String::from_utf8(self.data.clone()).map_err(Into::into)
    }
}

impl EnvelopedMessage {
    pub fn end() -> Self {
        Self::new([], 2)
    }

    pub fn is_end(&self) -> bool {
        self.flags == 2
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct MessageContent {
    pub text: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FilledPrompt {
    #[serde(rename = "filledPrompt")]
    pub text: String,
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::*;

    #[test]
    fn test_encode_decode() -> Result<()> {
        let bytes = [1u8, 2, 3, 4];
        let result = EnvelopedMessage::new(bytes, 1).encode();
        assert_eq!(result, vec![1, 0, 0, 0, 4, 1, 2, 3, 4]);

        let data = EnvelopedMessage::decode(result)?;
        assert_eq!(data.flags, 1);
        assert_eq!(data.data, bytes);
        Ok(())
    }
}

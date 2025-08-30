use super::FailedDecode;
use super::StorageEncoder;

use serde::Serialize;
use serde::de::DeserializeOwned;

/// Default [StorageEncoder].
///
/// Uses a non-human readable format.
/// Format uses Serde, and is compressed and then encoded into a utf8 compatible string.
pub struct DefaultEncoder;

impl<T: Serialize + DeserializeOwned> StorageEncoder<T> for DefaultEncoder {
    type EncodedValue = String;
    type DecodeError = FailedDecode<String>;

    fn deserialize(loaded: &Self::EncodedValue) -> Result<T, Self::DecodeError> {
        try_serde_from_string::<T>(loaded)
    }

    fn serialize(value: &T) -> Self::EncodedValue {
        serde_to_string(value)
    }
}

// Helper functions

/// Serializes a value to a string and compresses it.
fn serde_to_string<T: Serialize>(value: &T) -> String {
    let mut serialized = Vec::new();
    ciborium::into_writer(value, &mut serialized).unwrap();
    let compressed = yazi::compress(
        &serialized,
        yazi::Format::Zlib,
        yazi::CompressionLevel::BestSize,
    )
    .unwrap();
    let as_str: String = compressed
        .iter()
        .flat_map(|u| {
            [
                char::from_digit(((*u & 0xF0) >> 4).into(), 16).unwrap(),
                char::from_digit((*u & 0x0F).into(), 16).unwrap(),
            ]
            .into_iter()
        })
        .collect();
    as_str
}

/// Deserializes and decompresses a value from a string and returns None if there is an error.
fn try_serde_from_string<T: DeserializeOwned>(value: &str) -> Result<T, FailedDecode<String>> {
    let fail = |description: String| FailedDecode::from(value.to_string(), description);

    let mut bytes: Vec<u8> = Vec::new();
    let mut chars = value.chars();
    while let Some(c) = chars.next() {
        let n1 = c
            .to_digit(16)
            .ok_or_else(|| fail("decode error 1".to_string()))?;
        let c2 = chars
            .next()
            .ok_or_else(|| fail("decode error 2".to_string()))?;
        let n2 = c2
            .to_digit(16)
            .ok_or_else(|| fail("decode error 3".to_string()))?;
        bytes.push((n1 * 16 + n2) as u8);
    }

    match yazi::decompress(&bytes, yazi::Format::Zlib) {
        Ok((decompressed, _)) => ciborium::from_reader(std::io::Cursor::new(decompressed))
            .map_err(|err| fail(format!("ciborium Error: {err}"))),
        Err(err) => Result::Err(fail(format!("yazi Error: {err:?}"))),
    }
}

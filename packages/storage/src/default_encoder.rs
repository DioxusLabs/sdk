use super::FailedDecode;
use super::StorageEncoder;

use serde::Serialize;
use serde::de::DeserializeOwned;

/// Default [StorageEncoder].
///
/// Uses a non-human readable format.
/// Format uses Serde, and is compressed and then encoded into a utf8 compatible string using Hex.
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Debug;

    #[test]
    fn round_trips() {
        round_trip(0);
        round_trip(999);
        round_trip("Text".to_string());
        round_trip((1, 2, 3));
    }

    fn round_trip<T: Serialize + DeserializeOwned + PartialEq + Debug>(value: T) {
        let encoded = DefaultEncoder::serialize(&value);
        let decoded: Result<_, FailedDecode<String>> = DefaultEncoder::deserialize(&encoded);
        assert_eq!(value, decoded.unwrap());
    }

    #[test]
    fn can_decode_existing_data() {
        // This test ensures that data produced at the time of writing of this tests remains decomposable.
        // This will fail if, for example, the compression library drops support for this format, our the custom decode logic here changes in an incompatible way (like using base64 encoding).

        // Note that it would be possible to change the encode logic without breaking this (for example use base 64, but prefix data with an escape while keeping the old decode logic).
        // In the event of such a change, the test below (stable_encoding) will fail.
        // In such a case, the test cases here should NOT be modified.
        // These cases should be kept, and the new format should be added to these cases to ensure that the new format remains supported long term.

        assert_eq!(
            try_serde_from_string::<i32>("78da63000000010001").unwrap(),
            0i32
        );

        assert_eq!(
            try_serde_from_string::<String>("78dacb0e492d2e51082e29cacc4b07001da504a3").unwrap(),
            "Test String"
        );
    }

    #[test]
    fn stable_encoding() {
        // This tests that the encoder behavior has not changed.
        // The encoding changing isn't really breaking for users unless it also breaks decode of existing data or round trips (see other tests for those).
        // If this test does need to be updated, see note in `can_decode_existing_data` about adding additional test cases.

        assert_eq!(DefaultEncoder::serialize(&0), "78da63000000010001");
        assert_eq!(
            DefaultEncoder::serialize(&"Test String".to_string()),
            "78dacb0e492d2e51082e29cacc4b07001da504a3"
        );
    }
}

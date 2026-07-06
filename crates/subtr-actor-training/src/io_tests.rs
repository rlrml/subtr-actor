use super::{Reader, UeString, Writer};

fn write_string(string: &UeString) -> Vec<u8> {
    let mut writer = Writer::new();
    writer.ue_string(string).unwrap();
    writer.into_bytes()
}

fn read_string(bytes: &[u8]) -> UeString {
    let mut reader = Reader::new(bytes);
    let string = reader.ue_string().unwrap();
    assert!(reader.is_empty(), "string read must consume all bytes");
    string
}

#[test]
fn ansi_roundtrip() {
    let bytes = write_string(&UeString::Ansi("None".to_string()));
    assert_eq!(bytes, b"\x05\x00\x00\x00None\x00");
    assert_eq!(read_string(&bytes), UeString::Ansi("None".to_string()));
}

#[test]
fn null_and_empty_are_distinct() {
    assert_eq!(write_string(&UeString::Null), b"\x00\x00\x00\x00");
    assert_eq!(read_string(b"\x00\x00\x00\x00"), UeString::Null);

    let empty = UeString::Ansi(String::new());
    let bytes = write_string(&empty);
    assert_eq!(bytes, b"\x01\x00\x00\x00\x00");
    assert_eq!(read_string(&bytes), empty);
}

#[test]
fn utf16_roundtrip() {
    let value = "Привет 🚀";
    let string = UeString::new(value);
    assert_eq!(string, UeString::Utf16(value.to_string()));
    let bytes = write_string(&string);
    // length prefix is negative code-unit count including the terminator:
    // 7 chars + 2 surrogate units + 1 NUL = 10.
    assert_eq!(&bytes[..4], (-10i32).to_le_bytes());
    assert_eq!(bytes.len(), 4 + 10 * 2);
    assert_eq!(read_string(&bytes), string);
}

#[test]
fn encoding_choice_follows_the_game() {
    // Chars <= 0xFF stay ANSI even outside ASCII.
    assert!(matches!(UeString::new("héllo ÿ"), UeString::Ansi(_)));
    // The euro sign is windows-1252 encodable but its code point is > 0xFF,
    // so fresh strings pick UTF-16 exactly like the game's writer.
    assert!(matches!(UeString::new("€5"), UeString::Utf16(_)));
}

#[test]
fn cp1252_high_bytes_preserve_encoding() {
    // Byte 0x80 decodes to U+20AC (€) — a char > 0xFF — but the string was
    // stored as ANSI and must be re-serialized as ANSI byte-for-byte.
    let bytes = b"\x03\x00\x00\x00\x80\x99\x00";
    let string = read_string(bytes);
    assert_eq!(string, UeString::Ansi("€™".to_string()));
    assert_eq!(write_string(&string), bytes);
}

#[test]
fn missing_nul_terminator_is_rejected() {
    let mut reader = Reader::new(b"\x02\x00\x00\x00ab");
    assert!(reader.ue_string().is_err());
}

#[test]
fn serde_representation() {
    let ansi = serde_json::to_string(&UeString::Ansi("hi".to_string())).unwrap();
    assert_eq!(ansi, "\"hi\"");
    let utf16 = serde_json::to_string(&UeString::Utf16("hî".to_string())).unwrap();
    assert_eq!(utf16, "{\"utf16\":\"hî\"}");
    let null = serde_json::to_string(&UeString::Null).unwrap();
    assert_eq!(null, "null");

    for original in [
        UeString::Null,
        UeString::Ansi("plain".to_string()),
        UeString::Ansi("€ stored as ansi".to_string()),
        UeString::Utf16("Ünï€ödé 🚀".to_string()),
        UeString::Utf16("ascii but utf16".to_string()),
    ] {
        let json = serde_json::to_string(&original).unwrap();
        let back: UeString = serde_json::from_str(&json).unwrap();
        assert_eq!(back, original, "via {json}");
    }
}

#[test]
fn reader_primitives_and_eof() {
    let mut writer = Writer::new();
    writer.u8(7);
    writer.i32(-5);
    writer.u32(0xDEAD_BEEF);
    writer.u64(1 << 40);
    writer.f32(1.5);
    let bytes = writer.into_bytes();

    let mut reader = Reader::new(&bytes);
    assert_eq!(reader.u8().unwrap(), 7);
    assert_eq!(reader.i32().unwrap(), -5);
    assert_eq!(reader.u32().unwrap(), 0xDEAD_BEEF);
    assert_eq!(reader.u64().unwrap(), 1 << 40);
    assert_eq!(reader.f32().unwrap(), 1.5);
    assert!(reader.is_empty());
    assert!(reader.u8().is_err());
}

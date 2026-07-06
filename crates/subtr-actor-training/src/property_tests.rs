use super::{
    ArrayObject, ArrayValue, ByteValue, PropertyList, PropertyValue, StructBody, StructValue,
    UnknownValue,
};
use crate::io::{Reader, UeString, Writer};

fn ansi(writer: &mut Writer, value: &str) {
    writer.str(value).unwrap();
}

fn header(writer: &mut Writer, name: &str, type_tag: &str, length: i32, index: i32) {
    ansi(writer, name);
    ansi(writer, type_tag);
    writer.i32(length);
    writer.i32(index);
}

fn parse_exact(bytes: &[u8]) -> PropertyList {
    let mut reader = Reader::new(bytes);
    let list = PropertyList::parse(&mut reader).unwrap();
    assert!(reader.is_empty(), "parse must consume every byte");
    list
}

fn serialize(list: &PropertyList) -> Vec<u8> {
    let mut writer = Writer::new();
    list.serialize(&mut writer).unwrap();
    writer.into_bytes()
}

fn assert_roundtrip(bytes: &[u8]) -> PropertyList {
    let list = parse_exact(bytes);
    assert_eq!(serialize(&list), bytes, "byte-faithful round trip");
    // JSON round trip must be lossless too.
    let json = serde_json::to_string(&list).unwrap();
    let back: PropertyList = serde_json::from_str(&json).unwrap();
    assert_eq!(back, list, "JSON round trip via {json}");
    list
}

fn value_of<'a>(list: &'a PropertyList, name: &str) -> &'a PropertyValue {
    &list.get(name).unwrap().value
}

#[test]
fn scalar_property_types() {
    let mut writer = Writer::new();
    // BoolProperty writes one byte but declares length 0.
    header(&mut writer, "bFlag", "BoolProperty", 0, 0);
    writer.u8(1);
    header(&mut writer, "Count", "IntProperty", 4, 0);
    writer.i32(-42);
    header(&mut writer, "Stamp", "QWordProperty", 8, 0);
    writer.u64(1_650_000_000_123);
    header(&mut writer, "Ratio", "FloatProperty", 4, 0);
    writer.f32(2.75);
    // StrProperty: declared length covers the whole string blob.
    let text = UeString::new("hello");
    header(
        &mut writer,
        "Label",
        "StrProperty",
        i32::try_from(text.serialized_len()).unwrap(),
        0,
    );
    writer.ue_string(&text).unwrap();
    let name_value = UeString::new("Park_P");
    header(
        &mut writer,
        "MapName",
        "NameProperty",
        i32::try_from(name_value.serialized_len()).unwrap(),
        0,
    );
    writer.ue_string(&name_value).unwrap();
    header(&mut writer, "Ref", "ObjectProperty", 4, 0);
    writer.i32(3);
    ansi(&mut writer, "None");

    let list = assert_roundtrip(&writer.into_bytes());
    assert_eq!(value_of(&list, "bFlag"), &PropertyValue::Bool(1));
    assert_eq!(value_of(&list, "Count"), &PropertyValue::Int(-42));
    assert_eq!(
        value_of(&list, "Stamp"),
        &PropertyValue::QWord(1_650_000_000_123)
    );
    assert_eq!(value_of(&list, "Ratio"), &PropertyValue::Float(2.75));
    assert_eq!(value_of(&list, "Label"), &PropertyValue::Str(text));
    assert_eq!(value_of(&list, "MapName"), &PropertyValue::Name(name_value));
    assert_eq!(value_of(&list, "Ref"), &PropertyValue::Object(3));
}

#[test]
fn utf16_str_property() {
    let text = UeString::new("Тренировка");
    assert!(matches!(text, UeString::Utf16(_)));
    let mut writer = Writer::new();
    header(
        &mut writer,
        "TM_Name",
        "StrProperty",
        i32::try_from(text.serialized_len()).unwrap(),
        0,
    );
    writer.ue_string(&text).unwrap();
    ansi(&mut writer, "None");
    let list = assert_roundtrip(&writer.into_bytes());
    assert_eq!(value_of(&list, "TM_Name"), &PropertyValue::Str(text));
}

#[test]
fn str_property_declared_length_mismatch_is_preserved() {
    // RocketRP documents that StrProperty lengths can mismatch (UTF-16); a
    // divergent declared length must be written back verbatim.
    let text = UeString::new("abc");
    let real = i32::try_from(text.serialized_len()).unwrap();
    let bogus = real + 3;
    let mut writer = Writer::new();
    header(&mut writer, "Label", "StrProperty", bogus, 0);
    writer.ue_string(&text).unwrap();
    ansi(&mut writer, "None");
    let bytes = writer.into_bytes();

    let list = assert_roundtrip(&bytes);
    assert_eq!(list.get("Label").unwrap().declared_length, Some(bogus));
}

#[test]
fn byte_property_raw_and_enum() {
    let mut writer = Writer::new();
    // Raw byte: "None" enum marker + byte, declared length 1.
    header(&mut writer, "SplitscreenID", "ByteProperty", 1, 0);
    ansi(&mut writer, "None");
    writer.u8(2);
    // Enum: type name + value name; declared length covers only the value
    // name string (4 + len + 1), not the enum type name.
    let value = UeString::new("D_Medium");
    header(
        &mut writer,
        "Difficulty",
        "ByteProperty",
        i32::try_from(value.serialized_len()).unwrap(),
        0,
    );
    ansi(&mut writer, "EDifficulty");
    writer.ue_string(&value).unwrap();
    ansi(&mut writer, "None");

    let list = assert_roundtrip(&writer.into_bytes());
    assert_eq!(
        value_of(&list, "SplitscreenID"),
        &PropertyValue::Byte(ByteValue::Raw(2))
    );
    assert_eq!(
        value_of(&list, "Difficulty"),
        &PropertyValue::Byte(ByteValue::Enum {
            enum_type: UeString::new("EDifficulty"),
            value,
        })
    );
    // No mismatch should be recorded: the computed lengths use the quirky
    // rules already.
    assert!(list.get("SplitscreenID").unwrap().declared_length.is_none());
    assert!(list.get("Difficulty").unwrap().declared_length.is_none());
}

#[test]
fn enum_with_u32_underlying_type_is_an_int_property() {
    // C# serializes uint-backed enums as IntProperty; generically that is
    // indistinguishable from a plain int, which is exactly how it must
    // round-trip.
    let mut writer = Writer::new();
    header(&mut writer, "SomeEnum", "IntProperty", 4, 0);
    writer.i32(7);
    ansi(&mut writer, "None");
    let list = assert_roundtrip(&writer.into_bytes());
    assert_eq!(value_of(&list, "SomeEnum"), &PropertyValue::Int(7));
}

#[test]
fn special_struct_is_raw_and_tagged_struct_is_nested() {
    let guid_bytes: Vec<u8> = (1..=16u8).collect();
    let mut writer = Writer::new();
    // Guid: declared length is the 16-byte body, excluding the type name.
    header(&mut writer, "TM_Guid", "StructProperty", 16, 0);
    ansi(&mut writer, "Guid");
    writer.bytes(&guid_bytes);
    // UniqueNetId: nested tagged properties; declared length is the body
    // (nested props + "None") excluding the struct type name string.
    let mut body = Writer::new();
    header(&mut body, "Uid", "QWordProperty", 8, 0);
    body.u64(76_561_198_000_000_000);
    ansi(&mut body, "None");
    let body_bytes = body.into_bytes();
    header(
        &mut writer,
        "CreatorPlayerID",
        "StructProperty",
        i32::try_from(body_bytes.len()).unwrap(),
        0,
    );
    ansi(&mut writer, "UniqueNetId");
    writer.bytes(&body_bytes);
    ansi(&mut writer, "None");

    let list = assert_roundtrip(&writer.into_bytes());
    match value_of(&list, "TM_Guid") {
        PropertyValue::Struct(StructValue {
            struct_type,
            body: StructBody::Raw(raw),
        }) => {
            assert!(struct_type.is("Guid"));
            assert_eq!(raw, &guid_bytes);
        }
        other => panic!("expected raw Guid struct, got {other:?}"),
    }
    match value_of(&list, "CreatorPlayerID") {
        PropertyValue::Struct(StructValue {
            struct_type,
            body: StructBody::Properties(fields),
        }) => {
            assert!(struct_type.is("UniqueNetId"));
            assert_eq!(
                &fields.get("Uid").unwrap().value,
                &PropertyValue::QWord(76_561_198_000_000_000)
            );
        }
        other => panic!("expected tagged UniqueNetId struct, got {other:?}"),
    }
}

#[test]
fn fixed_size_array_entries_repeat_the_name_with_indexes() {
    let mut writer = Writer::new();
    header(&mut writer, "Slots", "IntProperty", 4, 0);
    writer.i32(10);
    header(&mut writer, "Slots", "IntProperty", 4, 1);
    writer.i32(20);
    header(&mut writer, "Slots", "IntProperty", 4, 3);
    writer.i32(40);
    ansi(&mut writer, "None");

    let list = assert_roundtrip(&writer.into_bytes());
    let entries: Vec<_> = list
        .properties
        .iter()
        .filter(|property| property.name.is("Slots"))
        .collect();
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[1].index, 1);
    assert_eq!(entries[2].index, 3);
    assert_eq!(entries[2].value, PropertyValue::Int(40));
}

#[test]
fn dynamic_arrays_of_ints_and_strings() {
    let mut tags_body = Writer::new();
    tags_body.i32(3);
    for tag in [5, 6, 7] {
        tags_body.i32(tag);
    }
    let tags_bytes = tags_body.into_bytes();

    let mut strings_body = Writer::new();
    strings_body.i32(2);
    strings_body.str("alpha").unwrap();
    strings_body.ue_string(&UeString::new("βeta")).unwrap();
    let strings_bytes = strings_body.into_bytes();

    let mut writer = Writer::new();
    header(
        &mut writer,
        "Tags",
        "ArrayProperty",
        i32::try_from(tags_bytes.len()).unwrap(),
        0,
    );
    writer.bytes(&tags_bytes);
    header(
        &mut writer,
        "SerializedArchetypes",
        "ArrayProperty",
        i32::try_from(strings_bytes.len()).unwrap(),
        0,
    );
    writer.bytes(&strings_bytes);
    ansi(&mut writer, "None");

    let list = assert_roundtrip(&writer.into_bytes());
    assert_eq!(
        value_of(&list, "Tags"),
        &PropertyValue::Array(ArrayValue::Ints(vec![5, 6, 7]))
    );
    assert_eq!(
        value_of(&list, "SerializedArchetypes"),
        &PropertyValue::Array(ArrayValue::Strings(vec![
            UeString::new("alpha"),
            UeString::new("βeta"),
        ]))
    );
}

#[test]
fn dynamic_array_of_value_type_structs() {
    // Value-type struct elements are consecutive tagged property lists with
    // no per-element header (Object.cs SerializePropertyValue, isArrayProp
    // branch for value types).
    let mut body = Writer::new();
    body.i32(2);
    for time_limit in [8.0f32, 12.5] {
        header(&mut body, "TimeLimit", "FloatProperty", 4, 0);
        body.f32(time_limit);
        ansi(&mut body, "None");
    }
    let body_bytes = body.into_bytes();

    let mut writer = Writer::new();
    header(
        &mut writer,
        "Rounds",
        "ArrayProperty",
        i32::try_from(body_bytes.len()).unwrap(),
        0,
    );
    writer.bytes(&body_bytes);
    ansi(&mut writer, "None");

    let list = assert_roundtrip(&writer.into_bytes());
    match value_of(&list, "Rounds") {
        PropertyValue::Array(ArrayValue::Structs(rounds)) => {
            assert_eq!(rounds.len(), 2);
            assert_eq!(
                rounds[1].get("TimeLimit").unwrap().value,
                PropertyValue::Float(12.5)
            );
        }
        other => panic!("expected struct array, got {other:?}"),
    }
}

#[test]
fn dynamic_array_of_class_elements_has_full_name_and_obj_header() {
    // Non-value-type elements are prefixed with their full class name and a
    // 0xFFFFFFFF object header (Object.cs lines ~168-183 / ~346-357).
    let mut body = Writer::new();
    body.i32(2);
    for count in [1, 2] {
        ansi(&mut body, "TAGame.SomeNestedObject_TA");
        body.u32(0xFFFF_FFFF);
        header(&mut body, "Value", "IntProperty", 4, 0);
        body.i32(count);
        ansi(&mut body, "None");
    }
    let body_bytes = body.into_bytes();

    let mut writer = Writer::new();
    header(
        &mut writer,
        "Children",
        "ArrayProperty",
        i32::try_from(body_bytes.len()).unwrap(),
        0,
    );
    writer.bytes(&body_bytes);
    ansi(&mut writer, "None");

    let list = assert_roundtrip(&writer.into_bytes());
    match value_of(&list, "Children") {
        PropertyValue::Array(ArrayValue::Objects(items)) => {
            assert_eq!(items.len(), 2);
            assert_eq!(
                items[0],
                ArrayObject {
                    class: UeString::new("TAGame.SomeNestedObject_TA"),
                    properties: {
                        let mut properties = PropertyList::default();
                        properties.set("Value", PropertyValue::Int(1));
                        properties
                    }
                }
            );
        }
        other => panic!("expected object array, got {other:?}"),
    }
}

#[test]
fn empty_dynamic_array_roundtrips() {
    let mut writer = Writer::new();
    header(&mut writer, "Rounds", "ArrayProperty", 4, 0);
    writer.i32(0);
    ansi(&mut writer, "None");
    let list = assert_roundtrip(&writer.into_bytes());
    assert!(matches!(
        value_of(&list, "Rounds"),
        PropertyValue::Array(ArrayValue::Raw { count: 0, data }) if data.is_empty()
    ));
}

#[test]
fn unknown_property_type_is_preserved_verbatim() {
    let payload = [0xDE, 0xAD, 0xBE, 0xEF, 0x01];
    let mut writer = Writer::new();
    header(
        &mut writer,
        "Mystery",
        "FrobProperty",
        i32::try_from(payload.len()).unwrap(),
        0,
    );
    writer.bytes(&payload);
    ansi(&mut writer, "None");

    let list = assert_roundtrip(&writer.into_bytes());
    match value_of(&list, "Mystery") {
        PropertyValue::Unknown(UnknownValue { type_name, data }) => {
            assert!(type_name.is("FrobProperty"));
            assert_eq!(data.as_slice(), payload);
        }
        other => panic!("expected unknown property, got {other:?}"),
    }
}

#[test]
fn unparseable_array_payload_falls_back_to_raw() {
    // A declared count that cannot be satisfied by any element
    // interpretation must be kept as raw bytes.
    let mut body = Writer::new();
    body.i32(3);
    body.bytes(&[0xFF, 0x01]);
    let body_bytes = body.into_bytes();

    let mut writer = Writer::new();
    header(
        &mut writer,
        "Weird",
        "ArrayProperty",
        i32::try_from(body_bytes.len()).unwrap(),
        0,
    );
    writer.bytes(&body_bytes);
    ansi(&mut writer, "None");

    let list = assert_roundtrip(&writer.into_bytes());
    assert!(matches!(
        value_of(&list, "Weird"),
        PropertyValue::Array(ArrayValue::Raw { count: 3, .. })
    ));
}

#[test]
fn set_replaces_in_place_and_appends_when_missing() {
    let mut list = PropertyList::default();
    list.set("A", PropertyValue::Int(1));
    list.set("B", PropertyValue::Int(2));
    list.set("a", PropertyValue::Int(10)); // case-insensitive replace
    assert_eq!(list.properties.len(), 2);
    assert_eq!(value_of(&list, "A"), &PropertyValue::Int(10));
    assert!(list.properties[0].name.is("A"), "position preserved");
    list.remove("b");
    assert!(list.get("B").is_none());
}

use std::str::FromStr as _;

use eon::{FormatOptions, Value};

const FIXTURE: &str = include_str!("fixtures/large_nested.eon");

fn root_map(value: &Value) -> &eon::Map {
    value.as_map().expect("fixture root should be a map")
}

fn deep_layer_count(value: &Value) -> usize {
    let mut current = root_map(value)
        .get_str("deep_tree")
        .and_then(Value::as_map)
        .expect("deep_tree should be a map");
    let mut count = 0;

    loop {
        let layer_name = format!("layer_{count:02}");
        let layer = current
            .get_str(&layer_name)
            .and_then(Value::as_map)
            .unwrap_or_else(|| panic!("missing {layer_name}"));
        count += 1;

        let payload = layer
            .get_str("payload")
            .and_then(Value::as_variant)
            .expect("payload should be a variant");
        assert_eq!(payload.name, "Layer");

        let payload_map = payload
            .values
            .first()
            .and_then(Value::as_map)
            .expect("Layer payload should be a map");

        if let Some(next) = payload_map.get_str("next") {
            current = next.as_map().expect("next should be a map");
        } else {
            let terminal = payload_map
                .get_str("terminal")
                .and_then(Value::as_map)
                .expect("terminal should be a map");
            let codes = terminal
                .get_str("codes")
                .and_then(Value::as_list)
                .expect("terminal codes should be a list");
            assert_eq!(codes.len(), 3);
            return count;
        }
    }
}

#[test]
fn test_large_nested_fixture_end_to_end() {
    let legacy = Value::from_str(FIXTURE).expect("legacy parser should parse the fixture");
    let core =
        Value::from_str_with_core(FIXTURE).expect("core parser should parse the fixture exactly");
    assert_eq!(legacy, core);

    let root = root_map(&legacy);
    assert_eq!(
        root.get_str("fixture_name").and_then(Value::as_string),
        Some("large_nested_fixture")
    );
    assert_eq!(
        root.get_str("services")
            .and_then(Value::as_list)
            .map(|values| values.len()),
        Some(6)
    );
    assert_eq!(
        root.get_str("matrix")
            .and_then(Value::as_list)
            .map(|values| values.len()),
        Some(8)
    );
    assert_eq!(deep_layer_count(&legacy), 15);

    let legacy_reformatted =
        eon::reformat(FIXTURE, &FormatOptions::default()).expect("legacy reformat should work");
    let legacy_roundtrip: Value = legacy_reformatted
        .parse()
        .expect("legacy reformat output should parse");
    assert_eq!(legacy_roundtrip, legacy);

    let formatter_core_reformatted =
        eon_formatter_core::reformat(FIXTURE, &eon_formatter_core::FormatOptions::default())
            .expect("formatter-core reformat should work");
    let formatter_core_roundtrip =
        Value::from_str_with_core(&formatter_core_reformatted).expect("formatter-core output");
    assert_eq!(formatter_core_roundtrip, legacy);

    let compact = legacy.to_string_with_core();
    let compact_roundtrip =
        Value::from_str_with_core(&compact).expect("compact core output should parse");
    assert_eq!(compact_roundtrip, legacy);
}

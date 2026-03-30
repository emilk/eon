use eon_formatter_core::{FormatOptions, reformat};

fn assert_reformat_is_idempotent(input: &str) {
    let once = reformat(input, &FormatOptions::default()).unwrap();
    let twice = reformat(&once, &FormatOptions::default()).unwrap();
    assert_eq!(twice, once, "input:\n{input}");
}

#[test]
fn idempotent_for_identifier_variants() {
    let input = r#"
        mode: EnumValue
        next: Another({
            value: 1
            items: [1, 2, 3]
        })
    "#;

    assert_reformat_is_idempotent(input);
}

#[test]
fn idempotent_for_root_variant_payload() {
    assert_reformat_is_idempotent("EnumValue({ foo: [1, 2, 3] })");
}

#[test]
fn idempotent_for_root_map_canonicalization() {
    let input = r#"
        {
            alpha: 1
            beta: {
                nested: true
            }
        }
    "#;

    assert_reformat_is_idempotent(input);
}

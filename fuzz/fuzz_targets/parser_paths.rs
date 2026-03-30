#![no_main]

use std::str::FromStr as _;

use eon::Value;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(input) = std::str::from_utf8(data) else {
        return;
    };

    let syntax = eon_syntax::TokenTree::parse_str(input);
    let _ = eon::reformat(input, &Default::default());
    let core = Value::from_str_with_core(input);
    let legacy = Value::from_str(input);

    if let (Ok(core), Ok(legacy)) = (&core, &legacy) {
        assert_eq!(core, legacy);
    }

    if let (Ok(_tree), Ok(core), Ok(legacy)) = (&syntax, &core, &legacy) {
        let core_from_legacy = legacy.to_string_with_core();
        let reparsed_core = Value::from_str_with_core(&core_from_legacy)
            .expect("core serializer output from legacy value should parse");
        assert_eq!(reparsed_core, *legacy);

        let legacy_from_core = core.format(&Default::default());
        let reparsed_legacy: Value = legacy_from_core
            .parse()
            .expect("legacy formatter output from core value should parse");
        assert_eq!(reparsed_legacy, *core);
    }

    if let Ok(value) = legacy {
        let formatted = value.format(&Default::default());
        let _ = Value::from_str(&formatted).expect("formatted legacy value should parse");
    }

    if let Ok(value) = core {
        let formatted = value.to_string_with_core();
        let reparsed = Value::from_str_with_core(&formatted)
            .expect("formatted core value should parse");
        assert_eq!(reparsed, value);
    }
});

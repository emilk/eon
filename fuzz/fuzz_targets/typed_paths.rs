#![no_main]

use std::collections::BTreeMap;

use arbitrary::Unstructured;
use eon::{FormatOptions, experimental};
use libfuzzer_sys::fuzz_target;
use serde::{Deserialize, Serialize};

const MAX_TAGS: usize = 4;
const MAX_MAP_ITEMS: usize = 4;
const MAX_TEXT_LEN: usize = 6;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
enum Mode {
    EnumValue,
    Other,
    #[serde(rename = "kebab-case")]
    KebabCase,
    #[serde(rename = "true")]
    TrueKeyword,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
enum Color {
    Rgb { r: u8, g: u8, b: u8 },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct Config {
    mode: Mode,
    label: String,
    retries: u8,
    enabled: bool,
    tags: Vec<String>,
    color: Color,
    mapping: BTreeMap<Mode, Mode>,
}

fuzz_target!(|data: &[u8]| {
    let Ok(config) = arbitrary_config(&mut Unstructured::new(data)) else {
        return;
    };

    let legacy_text = eon::to_string(&config, &FormatOptions::default())
        .expect("legacy typed serializer should succeed");
    let parsed_legacy: Config =
        eon::from_str(&legacy_text).expect("legacy typed output should parse with legacy path");
    assert_eq!(parsed_legacy, config);

    let parsed_core_from_legacy: Config = experimental::from_str_with_core(&legacy_text)
        .expect("legacy typed output should parse with core path");
    assert_eq!(parsed_core_from_legacy, config);

    let core_text =
        experimental::to_string_with_core(&config).expect("core typed serializer should succeed");
    let parsed_core: Config = experimental::from_str_with_core(&core_text)
        .expect("core typed output should parse with core path");
    assert_eq!(parsed_core, config);
});

fn arbitrary_config(u: &mut Unstructured<'_>) -> arbitrary::Result<Config> {
    let tag_len = u.int_in_range(0..=MAX_TAGS)?;
    let mut tags = Vec::with_capacity(tag_len);
    for _ in 0..tag_len {
        tags.push(arbitrary_string(u)?);
    }

    let map_len = u.int_in_range(0..=MAX_MAP_ITEMS)?;
    let mut mapping = BTreeMap::new();
    for _ in 0..map_len {
        mapping.insert(arbitrary_mode(u), arbitrary_mode(u));
    }

    Ok(Config {
        mode: arbitrary_mode(u),
        label: arbitrary_string(u)?,
        retries: u.arbitrary()?,
        enabled: u.arbitrary()?,
        tags,
        color: Color::Rgb {
            r: u.arbitrary()?,
            g: u.arbitrary()?,
            b: u.arbitrary()?,
        },
        mapping,
    })
}

fn arbitrary_mode(u: &mut Unstructured<'_>) -> Mode {
    match u.int_in_range(0..=3).unwrap_or(0) {
        0 => Mode::EnumValue,
        1 => Mode::Other,
        2 => Mode::KebabCase,
        _ => Mode::TrueKeyword,
    }
}

fn arbitrary_string(u: &mut Unstructured<'_>) -> arbitrary::Result<String> {
    let len = u.int_in_range(0..=MAX_TEXT_LEN)?;
    let mut out = String::new();
    for _ in 0..len {
        out.push(u.arbitrary::<char>()?);
    }
    Ok(out)
}

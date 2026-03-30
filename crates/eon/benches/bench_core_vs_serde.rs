use std::{collections::BTreeMap, str::FromStr as _, sync::OnceLock};

use divan::counter::BytesCount;
use serde::{Deserialize, Serialize};

fn main() {
    divan::main();
}

const TARGET_SIZE: usize = 1_000_000;

#[derive(Debug)]
struct Fixture {
    source_serde: String,
    source_core: String,
    typed: Document,
    value: eon::Value,
}

fn fixture() -> &'static Fixture {
    static FIXTURE: OnceLock<Fixture> = OnceLock::new();
    FIXTURE.get_or_init(|| build_fixture(TARGET_SIZE))
}

fn build_fixture(target_size: usize) -> Fixture {
    let mut document = Document {
        version: 1,
        title: "Benchmark dataset".to_owned(),
        defaults: Defaults {
            mode: Mode::Auto,
            color: Color::Rgb {
                r: 32,
                g: 64,
                b: 128,
            },
            retries: 3,
        },
        items: Vec::new(),
        lookup: BTreeMap::new(),
    };

    let format = eon::FormatOptions::default();

    let source_serde = loop {
        let index = document.items.len() as u32;
        let item = make_item(index);
        document.lookup.insert(item.name.clone(), index);
        document.items.push(item);

        if document.items.len() % 32 == 0 {
            let source = eon::to_string(&document, &format).expect("fixture serialization failed");
            if source.len() >= target_size {
                break source;
            }
        }
    };

    let value = eon::to_value(&document).expect("fixture value conversion failed");
    let source_core = value.to_string_with_core();

    Fixture {
        source_serde,
        source_core,
        typed: document,
        value,
    }
}

fn make_item(index: u32) -> Item {
    Item {
        id: index,
        name: format!("item_{index:05}"),
        enabled: index % 3 != 0,
        tags: vec![
            format!("group_{}", index % 7),
            format!("kind_{}", index % 11),
            "bench".to_owned(),
        ],
        mode: match index % 4 {
            0 => Mode::Auto,
            1 => Mode::Named(format!("mode_{index}")),
            2 => Mode::Range(index, index + 100),
            _ => Mode::Advanced {
                level: (index % 8) as u8,
                label: format!("L{}", index % 16),
            },
        },
        nested: Nested {
            threshold: (index % 100) as f32 / 10.0,
            path: format!("/srv/data/{index:05}"),
            color: match index % 3 {
                0 => Color::Black,
                1 => Color::Gray((index % 255) as u8),
                _ => Color::Rgb {
                    r: (index % 255) as u8,
                    g: ((index * 3) % 255) as u8,
                    b: ((index * 7) % 255) as u8,
                },
            },
        },
        metrics: vec![index as f32 * 0.5, index as f32 * 1.5, index as f32 * 2.5],
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Document {
    version: u32,
    title: String,
    defaults: Defaults,
    items: Vec<Item>,
    lookup: BTreeMap<String, u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Defaults {
    mode: Mode,
    color: Color,
    retries: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Item {
    id: u32,
    name: String,
    enabled: bool,
    tags: Vec<String>,
    mode: Mode,
    nested: Nested,
    metrics: Vec<f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Nested {
    threshold: f32,
    path: String,
    color: Color,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
enum Mode {
    Auto,
    Named(String),
    Range(u32, u32),
    Advanced { level: u8, label: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
enum Color {
    Black,
    Gray(u8),
    Rgb { r: u8, g: u8, b: u8 },
}

#[divan::bench]
fn parse_typed_serde(bencher: divan::Bencher<'_, '_>) {
    let source = fixture().source_serde.as_str();
    bencher
        .counter(BytesCount::of_str(&source))
        .bench_local(move || {
            let parsed: Document = eon::from_str(source).expect("typed serde parse failed");
            divan::black_box(parsed);
        });
}

#[divan::bench]
fn parse_typed_core_on_serde_syntax(bencher: divan::Bencher<'_, '_>) {
    let source = fixture().source_serde.as_str();
    bencher
        .counter(BytesCount::of_str(&source))
        .bench_local(move || {
            let parsed: Document =
                eon::experimental::from_str_with_core(source).expect("typed core parse failed");
            divan::black_box(parsed);
        });
}

#[divan::bench]
fn parse_typed_core_on_core_syntax(bencher: divan::Bencher<'_, '_>) {
    let source = fixture().source_core.as_str();
    bencher
        .counter(BytesCount::of_str(&source))
        .bench_local(move || {
            let parsed: Document = eon::experimental::from_str_with_core(source)
                .expect("typed core syntax parse failed");
            divan::black_box(parsed);
        });
}

#[divan::bench]
fn parse_value_default(bencher: divan::Bencher<'_, '_>) {
    let source = fixture().source_serde.as_str();
    bencher
        .counter(BytesCount::of_str(&source))
        .bench_local(move || {
            let parsed = eon::Value::from_str(source).expect("default value parse failed");
            divan::black_box(parsed);
        });
}

#[divan::bench]
fn parse_value_core_on_serde_syntax(bencher: divan::Bencher<'_, '_>) {
    let source = fixture().source_serde.as_str();
    bencher
        .counter(BytesCount::of_str(&source))
        .bench_local(move || {
            let parsed = eon::Value::from_str_with_core(source).expect("core value parse failed");
            divan::black_box(parsed);
        });
}

#[divan::bench]
fn parse_value_core_on_core_syntax(bencher: divan::Bencher<'_, '_>) {
    let source = fixture().source_core.as_str();
    bencher
        .counter(BytesCount::of_str(&source))
        .bench_local(move || {
            let parsed = eon::Value::from_str_with_core(source).expect("core syntax parse failed");
            divan::black_box(parsed);
        });
}

#[divan::bench]
fn stringify_typed_serde(bencher: divan::Bencher<'_, '_>) {
    let typed = &fixture().typed;
    let format = eon::FormatOptions::default();
    bencher
        .counter(BytesCount::of_str(&fixture().source_serde))
        .bench_local(move || {
            let string = eon::to_string(typed, &format).expect("typed serde stringify failed");
            divan::black_box(string);
        });
}

#[divan::bench]
fn stringify_value_default(bencher: divan::Bencher<'_, '_>) {
    let value = &fixture().value;
    bencher
        .counter(BytesCount::of_str(&fixture().source_serde))
        .bench_local(move || {
            let string = value.to_string();
            divan::black_box(string);
        });
}

#[divan::bench]
fn stringify_value_core(bencher: divan::Bencher<'_, '_>) {
    let value = &fixture().value;
    bencher
        .counter(BytesCount::of_str(&fixture().source_core))
        .bench_local(move || {
            let string = value.to_string_with_core();
            divan::black_box(string);
        });
}

#[divan::bench]
fn stringify_typed_via_core_format(bencher: divan::Bencher<'_, '_>) {
    let typed = &fixture().typed;
    bencher
        .counter(BytesCount::of_str(&fixture().source_core))
        .bench_local(move || {
            let value = eon::to_value(typed).expect("typed to value conversion failed");
            let string = value.to_string_with_core();
            divan::black_box(string);
        });
}

#[divan::bench]
fn stringify_typed_core_direct(bencher: divan::Bencher<'_, '_>) {
    let typed = &fixture().typed;
    bencher
        .counter(BytesCount::of_str(&fixture().source_core))
        .bench_local(move || {
            let string =
                eon::experimental::to_string_with_core(typed).expect("typed core stringify failed");
            divan::black_box(string);
        });
}

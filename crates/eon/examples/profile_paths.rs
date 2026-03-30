use std::{
    collections::BTreeMap,
    env, process,
    str::FromStr as _,
    sync::OnceLock,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};

const TARGET_SIZE: usize = 1_000_000;

fn main() {
    let mode = env::args().nth(1).unwrap_or_else(|| {
        panic!("usage: cargo run --release --example profile_paths -- <mode> [seconds]")
    });
    let seconds = env::args()
        .nth(2)
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(30);

    let fixture = fixture();
    eprintln!(
        "profile_paths pid={} mode={} seconds={}",
        process::id(),
        mode,
        seconds
    );

    let deadline = Instant::now() + Duration::from_secs(seconds);
    let mut iterations = 0_u64;

    while Instant::now() < deadline {
        match mode.as_str() {
            "parse_default" => {
                let value =
                    eon::Value::from_str(&fixture.source_serde).expect("parse_default failed");
                std::hint::black_box(value);
            }
            "parse_core" => {
                let value = eon::Value::from_str_with_core(&fixture.source_serde)
                    .expect("parse_core failed");
                std::hint::black_box(value);
            }
            "parse_typed_serde" => {
                let typed: Document =
                    eon::from_str(&fixture.source_serde).expect("parse_typed_serde failed");
                std::hint::black_box(typed);
            }
            "parse_typed_core_on_serde_syntax" => {
                let typed: Document = eon::experimental::from_str_with_core(&fixture.source_serde)
                    .expect("parse_typed_core_on_serde_syntax failed");
                std::hint::black_box(typed);
            }
            "parse_typed_core_on_core_syntax" => {
                let typed: Document = eon::experimental::from_str_with_core(&fixture.source_core)
                    .expect("parse_typed_core_on_core_syntax failed");
                std::hint::black_box(typed);
            }
            "stringify_value_default" => {
                let string = fixture.value.to_string();
                std::hint::black_box(string);
            }
            "stringify_value_core" => {
                let string = fixture.value.to_string_with_core();
                std::hint::black_box(string);
            }
            "stringify_typed_via_core" => {
                let value = eon::to_value(&fixture.typed).expect("typed->value failed");
                let string = value.to_string_with_core();
                std::hint::black_box(string);
            }
            "stringify_typed_core_direct" => {
                let string = eon::experimental::to_string_with_core(&fixture.typed)
                    .expect("stringify_typed_core_direct failed");
                std::hint::black_box(string);
            }
            "stringify_typed_serde" => {
                let string = eon::to_string(&fixture.typed, &eon::FormatOptions::default())
                    .expect("stringify_typed_serde failed");
                std::hint::black_box(string);
            }
            _ => panic!(
                "unknown mode {mode:?}; expected one of parse_default, parse_core, parse_typed_serde, parse_typed_core_on_serde_syntax, parse_typed_core_on_core_syntax, stringify_value_default, stringify_value_core, stringify_typed_via_core, stringify_typed_core_direct, stringify_typed_serde"
            ),
        }

        iterations += 1;
    }

    eprintln!("profile_paths iterations={iterations}");
}

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

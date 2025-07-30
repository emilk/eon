use std::str::FromStr as _;

fn main() {
    divan::main();
}

fn generate_big_file(target_size: usize) -> String {
    let mut eon_source = String::new();

    while eon_source.len() < target_size {
        eon_source += r#"
{
    type: "Feature",
    properties: {
        "MAPBLKLOT": 1001,
        "BLKLOT": 1001,
        "BLOCK_NUM": 1,
        "LOT_NUM": 1,
        "FROM_ST": 0,
        "TO_ST": 0,
        "STREET": "UNKNOWN",
        "ST_TYPE": null,
        "ODD_EVEN": "E"
    },
    geometry: {
        type: "Polygon",
        coordinates: [
            [
                [
                    -122.422003528252475,
                    37.808480096967251,
                    0.0
                ],
                [
                    -122.422076013325281,
                    37.808835019815085,
                    0.0
                ],
                [
                    -122.421102174348633,
                    37.808803534992904,
                    0.0
                ],
                [
                    -122.421062569067274,
                    37.808601056818148,
                    0.0
                ],
                [
                    -122.422003528252475,
                    37.808480096967251,
                    0.0
                ]
            ]
        ]
    }
}
    "#;
    }

    eon_source
}

#[divan::bench]
fn bench_tokenizer(bencher: divan::Bencher<'_, '_>) {
    let eon_source = generate_big_file(1_000_000);
    bencher.bench_local(move || {
        eon_syntax::TokenTree::parse_str(&eon_source).expect("Failed to parse Eon source");
    });
}

#[divan::bench]
fn bench_full_parse(bencher: divan::Bencher<'_, '_>) {
    let eon_source = generate_big_file(1_000_000);
    bencher.bench_local(move || {
        eon::Value::from_str(&eon_source).expect("Failed to parse Eon source");
    });
}

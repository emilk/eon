use eon_formatter_core::{FormatOptions, reformat};

const SAMPLE: &str = r#"
    // Sample config used to retain formatter-core code in the wasm artifact.
    app: {
        name: "demo"
        enabled: true
        retries: [1, 2, 3]
        variant: "Mode"({
            nested: true
            values: [1, 2, 3]
        })
    }
"#;

fn main() {
    let len = reformat(SAMPLE, &FormatOptions::default())
        .map(|formatted| formatted.len())
        .unwrap_or(0);

    core::hint::black_box(len);
}

use eon_schema::{EonSchema as _, SchemaNode, VariantPayload};

/// Main application config.
#[expect(
    dead_code,
    reason = "schema derive test types are inspected through generated schemas"
)]
#[derive(eon_schema::EonSchema)]
struct Config {
    /// Human-facing app name.
    name: String,
    /// Server port.
    #[serde(default)]
    port: u16,
    /// Runtime mode.
    #[serde(rename = "mode")]
    app_mode: Mode,
    /// Optional labels.
    labels: Option<Vec<String>>,
    #[serde(skip)]
    ignored: bool,
}

/// Runtime mode.
#[expect(
    dead_code,
    reason = "schema derive test types are inspected through generated schemas"
)]
#[derive(eon_schema::EonSchema)]
enum Mode {
    /// Debug mode.
    Debug,
    /// Release mode.
    Release,
    /// RGB color payload.
    Rgb(u8, u8, u8),
    /// Custom mode.
    Custom {
        /// Custom mode name.
        name: String,
    },
}

#[test]
fn derives_named_struct_schema() {
    let schema = Config::schema();
    let SchemaNode::Object(object) = schema else {
        panic!("expected object schema");
    };

    assert_eq!(object.name, "Config");
    assert_eq!(object.docs, "Main application config.");
    assert_eq!(object.fields.len(), 4);
    assert_eq!(object.fields[0].name, "name");
    assert_eq!(object.fields[0].docs, "Human-facing app name.");
    assert!(object.fields[0].required);
    assert_eq!(object.fields[1].name, "port");
    assert!(!object.fields[1].required);
    assert!(object.fields[1].default);
    assert_eq!(object.fields[2].name, "mode");
    assert_eq!(object.fields[3].name, "labels");
    assert!(!object.fields[3].required);
}

#[test]
fn derives_enum_schema() {
    let schema = Mode::schema();
    let SchemaNode::Enum(schema) = schema else {
        panic!("expected enum schema");
    };

    assert_eq!(schema.name, "Mode");
    assert_eq!(schema.docs, "Runtime mode.");
    assert_eq!(schema.variants.len(), 4);
    assert_eq!(schema.variants[0].name, "Debug");
    assert_eq!(schema.variants[0].payload, VariantPayload::Unit);

    let VariantPayload::Tuple(values) = &schema.variants[2].payload else {
        panic!("expected tuple payload");
    };
    assert_eq!(values.len(), 3);

    let VariantPayload::Struct(fields) = &schema.variants[3].payload else {
        panic!("expected struct payload");
    };
    assert_eq!(fields[0].name, "name");
}

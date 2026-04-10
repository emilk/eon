//! Eon language server.

use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    str::FromStr as _,
    sync::Arc,
};

use eon_schema::{EnumSchema, ObjectSchema, SchemaNode, VariantPayload};
use serde_json::Value as JsonValue;
use tokio::sync::RwLock;
use tower_lsp::{
    Client, LanguageServer, LspService, Server,
    jsonrpc::Result as JsonResult,
    lsp_types::{
        CompletionItem, CompletionItemKind, CompletionOptions, CompletionParams,
        CompletionResponse, Diagnostic, DiagnosticSeverity, DidChangeTextDocumentParams,
        DidCloseTextDocumentParams, DidOpenTextDocumentParams, DidSaveTextDocumentParams,
        DocumentFormattingParams, DocumentSymbol, DocumentSymbolParams, DocumentSymbolResponse,
        Documentation, InitializeParams, InitializeResult, InitializedParams, InsertTextFormat,
        MarkupContent, MarkupKind, MessageType, OneOf, Position, Range, ServerCapabilities,
        SymbolKind, TextDocumentSyncCapability, TextDocumentSyncKind, TextEdit, Url,
    },
};

#[derive(Default)]
struct ServerState {
    documents: HashMap<Url, String>,
    schema: Option<SchemaNode>,
    schema_path: Option<PathBuf>,
}

struct Backend {
    client: Client,
    state: Arc<RwLock<ServerState>>,
}

impl Backend {
    async fn set_document(&self, uri: Url, text: String) {
        self.state.write().await.documents.insert(uri, text);
    }

    async fn remove_document(&self, uri: &Url) {
        self.state.write().await.documents.remove(uri);
    }

    async fn get_document(&self, uri: &Url) -> Option<String> {
        self.state.read().await.documents.get(uri).cloned()
    }

    async fn get_schema(&self) -> Option<SchemaNode> {
        self.state.read().await.schema.clone()
    }

    async fn set_schema(&self, loaded: Option<LoadedSchema>) {
        let mut state = self.state.write().await;
        if let Some(loaded) = loaded {
            state.schema = Some(loaded.schema);
            state.schema_path = Some(loaded.path);
        } else {
            state.schema = None;
            state.schema_path = None;
        }
    }

    async fn publish_diagnostics(&self, uri: &Url) {
        let diagnostics = self
            .get_document(uri)
            .await
            .map_or_else(Vec::new, |text| diagnostics_for_document(&text, Some(uri)));
        self.client
            .publish_diagnostics(uri.clone(), diagnostics, None)
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> JsonResult<InitializeResult> {
        match load_schema_for_initialize_params(&params) {
            Ok(loaded) => self.set_schema(loaded).await,
            Err(message) => {
                self.set_schema(None).await;
                self.client
                    .log_message(
                        MessageType::WARNING,
                        format!("eon schema load failed: {message}"),
                    )
                    .await;
            }
        }

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                document_formatting_provider: Some(OneOf::Left(true)),
                completion_provider: Some(CompletionOptions::default()),
                document_symbol_provider: Some(OneOf::Left(true)),
                ..ServerCapabilities::default()
            },
            ..InitializeResult::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "eon-lsp initialized")
            .await;
    }

    async fn shutdown(&self) -> JsonResult<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.set_document(params.text_document.uri.clone(), params.text_document.text)
            .await;
        self.publish_diagnostics(&params.text_document.uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let DidChangeTextDocumentParams {
            text_document,
            content_changes,
        } = params;
        if let Some(change) = content_changes.last() {
            self.set_document(text_document.uri.clone(), change.text.clone())
                .await;
            self.publish_diagnostics(&text_document.uri).await;
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        self.publish_diagnostics(&params.text_document.uri).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.remove_document(&params.text_document.uri).await;
        self.client
            .publish_diagnostics(params.text_document.uri, Vec::new(), None)
            .await;
    }

    async fn formatting(
        &self,
        params: DocumentFormattingParams,
    ) -> JsonResult<Option<Vec<TextEdit>>> {
        let Some(source) = self.get_document(&params.text_document.uri).await else {
            return Ok(None);
        };

        let Some(formatted) = format_source(
            &source,
            params.options.insert_spaces,
            params.options.tab_size as usize,
        ) else {
            return Ok(None);
        };

        Ok(Some(vec![TextEdit {
            range: full_document_range(&source),
            new_text: formatted,
        }]))
    }

    async fn completion(&self, params: CompletionParams) -> JsonResult<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;
        let source = self.get_document(uri).await.unwrap_or_default();
        let schema = self.get_schema().await;
        let items = completion_items_for_source_with_schema(&source, schema.as_ref());
        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> JsonResult<Option<DocumentSymbolResponse>> {
        let Some(source) = self.get_document(&params.text_document.uri).await else {
            return Ok(None);
        };

        let Ok(tree) = eon_syntax::TokenTree::parse_str(&source) else {
            return Ok(None);
        };
        let symbols = symbols_from_token_tree(&tree, &source);
        if symbols.is_empty() {
            Ok(None)
        } else {
            Ok(Some(DocumentSymbolResponse::Nested(symbols)))
        }
    }
}

const DEFAULT_SCHEMA_FILE: &str = ".eon-schema.eon";

#[derive(Clone, Debug)]
struct LoadedSchema {
    schema: SchemaNode,
    path: PathBuf,
}

fn load_schema_for_initialize_params(
    params: &InitializeParams,
) -> std::result::Result<Option<LoadedSchema>, String> {
    let root_dir = root_dir_for_initialize_params(params);
    let explicit_path =
        schema_path_from_initialization_options(params.initialization_options.as_ref());
    let schema_path = explicit_path
        .map(|path| resolve_schema_path(root_dir.as_deref(), path))
        .or_else(|| discover_schema_path(root_dir.as_deref()));

    let Some(schema_path) = schema_path else {
        return Ok(None);
    };

    load_schema_from_path(schema_path).map(Some)
}

fn load_schema_from_path(path: PathBuf) -> std::result::Result<LoadedSchema, String> {
    let source = fs::read_to_string(&path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    let schema = SchemaNode::from_eon_str(&source)
        .map_err(|err| format!("failed to parse {}: {err}", path.display()))?;
    Ok(LoadedSchema { schema, path })
}

fn root_dir_for_initialize_params(params: &InitializeParams) -> Option<PathBuf> {
    params
        .workspace_folders
        .as_ref()
        .and_then(|folders| folders.first())
        .and_then(|folder| folder.uri.to_file_path().ok())
        .or_else(|| {
            params
                .root_uri
                .as_ref()
                .and_then(|uri| uri.to_file_path().ok())
        })
}

fn schema_path_from_initialization_options(options: Option<&JsonValue>) -> Option<PathBuf> {
    let options = options?;
    options
        .get("schemaPath")
        .or_else(|| options.get("eonSchemaPath"))
        .or_else(|| options.get("eon").and_then(|eon| eon.get("schemaPath")))
        .and_then(JsonValue::as_str)
        .map(PathBuf::from)
}

fn resolve_schema_path(root_dir: Option<&Path>, path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        root_dir.unwrap_or_else(|| Path::new(".")).join(path)
    }
}

fn discover_schema_path(root_dir: Option<&Path>) -> Option<PathBuf> {
    let root_dir = root_dir?;
    let path = root_dir.join(DEFAULT_SCHEMA_FILE);
    path.is_file().then_some(path)
}

fn diagnostics_for_document(source: &str, uri: Option<&Url>) -> Vec<Diagnostic> {
    if has_composition_syntax(source) {
        return diagnostics_for_composed_document(source, uri);
    }

    match eon::Value::from_str(source) {
        Ok(_) => Vec::new(),
        Err(err) => vec![diagnostic_for_parse_error(source, err)],
    }
}

fn has_composition_syntax(source: &str) -> bool {
    source.contains('$')
}

fn diagnostics_for_composed_document(source: &str, uri: Option<&Url>) -> Vec<Diagnostic> {
    let root_dir = root_dir_for_uri(uri);
    match eon_compose::Resolver::new(root_dir).resolve_str(source) {
        Ok(_) => Vec::new(),
        Err(err) => vec![Diagnostic {
            range: Range::new(Position::new(0, 0), Position::new(0, 0)),
            severity: Some(DiagnosticSeverity::ERROR),
            source: Some("eon-compose".to_owned()),
            message: err.to_string(),
            ..Diagnostic::default()
        }],
    }
}

fn root_dir_for_uri(uri: Option<&Url>) -> PathBuf {
    uri.and_then(|uri| uri.to_file_path().ok())
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn diagnostic_for_parse_error(source: &str, err: eon_syntax::Error) -> Diagnostic {
    match err {
        eon_syntax::Error::Custom { msg } => Diagnostic {
            range: Range::new(Position::new(0, 0), Position::new(0, 0)),
            severity: Some(DiagnosticSeverity::ERROR),
            source: Some("eon-lsp".to_owned()),
            message: msg,
            ..Diagnostic::default()
        },
        eon_syntax::Error::At { span, message, .. } => Diagnostic {
            range: range_from_span(source, span),
            severity: Some(DiagnosticSeverity::ERROR),
            source: Some("eon-lsp".to_owned()),
            message,
            ..Diagnostic::default()
        },
    }
}

fn format_source(source: &str, insert_spaces: bool, tab_size: usize) -> Option<String> {
    let indentation = if insert_spaces {
        " ".repeat(tab_size.max(1))
    } else {
        "\t".to_owned()
    };
    let options = eon_syntax::FormatOptions::default().with_indentation(indentation);
    let formatted = eon_syntax::reformat(source, &options).ok()?;
    if formatted == source {
        None
    } else {
        Some(formatted)
    }
}

fn completion_items_for_source_with_schema(
    source: &str,
    schema: Option<&SchemaNode>,
) -> Vec<CompletionItem> {
    const KEYWORDS: [&str; 6] = ["null", "true", "false", "+nan", "+inf", "-inf"];

    let mut seen = HashSet::<String>::new();
    let mut items = Vec::<CompletionItem>::new();

    for keyword in KEYWORDS {
        let label = keyword.to_owned();
        seen.insert(label.clone());
        items.push(CompletionItem {
            label,
            kind: Some(CompletionItemKind::KEYWORD),
            insert_text: Some(keyword.to_owned()),
            sort_text: Some(format!("0-{keyword}")),
            ..CompletionItem::default()
        });
    }

    if let Ok(tree) = eon_syntax::TokenTree::parse_str(source) {
        for key in collect_map_keys_from_tree(&tree) {
            if seen.insert(key.clone()) {
                items.push(CompletionItem {
                    label: key.clone(),
                    kind: Some(CompletionItemKind::PROPERTY),
                    insert_text: Some(key.clone()),
                    sort_text: Some(format!("1-{key}")),
                    ..CompletionItem::default()
                });
            }
        }
    }

    if let Some(schema) = schema {
        for item in completion_items_for_schema(schema) {
            if seen.insert(item.label.clone()) {
                items.push(item);
            }
        }
    }

    items
}

fn completion_items_for_schema(schema: &SchemaNode) -> Vec<CompletionItem> {
    match schema {
        SchemaNode::Optional(inner) => completion_items_for_schema(inner),
        SchemaNode::Object(object) => completion_items_for_object_schema(object),
        SchemaNode::Enum(schema) => completion_items_for_enum_schema(schema),
        _ => Vec::new(),
    }
}

fn completion_items_for_object_schema(schema: &ObjectSchema) -> Vec<CompletionItem> {
    schema
        .fields
        .iter()
        .map(|field| CompletionItem {
            label: field.name.to_owned(),
            kind: Some(CompletionItemKind::PROPERTY),
            insert_text: Some(field.name.to_owned()),
            detail: Some(if field.required {
                "required field".to_owned()
            } else if field.default {
                "field with default".to_owned()
            } else {
                "optional field".to_owned()
            }),
            documentation: documentation_from_docs(field.docs),
            sort_text: Some(format!("2-field-{}", field.name)),
            ..CompletionItem::default()
        })
        .collect()
}

fn completion_items_for_enum_schema(schema: &EnumSchema) -> Vec<CompletionItem> {
    schema
        .variants
        .iter()
        .map(|variant| {
            let (insert_text, insert_text_format) = match &variant.payload {
                VariantPayload::Unit => (variant.name.to_owned(), None),
                VariantPayload::Tuple(values) => {
                    let placeholders = (1..=values.len())
                        .map(|index| format!("${{{index}:value}}"))
                        .collect::<Vec<_>>()
                        .join(", ");
                    (
                        format!("{}({placeholders})", variant.name),
                        Some(InsertTextFormat::SNIPPET),
                    )
                }
                VariantPayload::Struct(fields) => {
                    let fields = fields
                        .iter()
                        .enumerate()
                        .map(|(index, field)| format!("{}: ${{{}:value}}", field.name, index + 1))
                        .collect::<Vec<_>>()
                        .join(", ");
                    (
                        format!("{}({{{fields}}})", variant.name),
                        Some(InsertTextFormat::SNIPPET),
                    )
                }
            };
            CompletionItem {
                label: variant.name.to_owned(),
                kind: Some(CompletionItemKind::ENUM_MEMBER),
                insert_text: Some(insert_text),
                insert_text_format,
                detail: Some("enum variant".to_owned()),
                documentation: documentation_from_docs(variant.docs),
                sort_text: Some(format!("2-variant-{}", variant.name)),
                ..CompletionItem::default()
            }
        })
        .collect()
}

fn documentation_from_docs(docs: &str) -> Option<Documentation> {
    (!docs.is_empty()).then(|| {
        Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: docs.to_owned(),
        })
    })
}

fn collect_map_keys_from_tree(tree: &eon_syntax::TokenTree<'_>) -> Vec<String> {
    let mut out = Vec::new();
    collect_keys_from_value(&tree.value, &mut out);
    out
}

fn collect_keys_from_value(value: &eon_syntax::TokenValue<'_>, out: &mut Vec<String>) {
    match value {
        eon_syntax::TokenValue::Map(map) => {
            for pair in &map.key_values {
                out.push(token_to_name(&pair.key));
                collect_keys_from_value(&pair.value.value, out);
            }
        }
        eon_syntax::TokenValue::List(list) => {
            for item in &list.values {
                collect_keys_from_value(&item.value, out);
            }
        }
        eon_syntax::TokenValue::Variant(variant) => {
            for item in &variant.values {
                collect_keys_from_value(&item.value, out);
            }
        }
        eon_syntax::TokenValue::Identifier(_)
        | eon_syntax::TokenValue::Number(_)
        | eon_syntax::TokenValue::QuotedString(_) => {}
    }
}

fn symbols_from_token_tree(tree: &eon_syntax::TokenTree<'_>, source: &str) -> Vec<DocumentSymbol> {
    match &tree.value {
        eon_syntax::TokenValue::Map(map) => map
            .key_values
            .iter()
            .map(|pair| symbol_from_pair(pair, source))
            .collect(),
        _ => Vec::new(),
    }
}

fn symbol_from_pair(pair: &eon_syntax::TokenKeyValue<'_>, source: &str) -> DocumentSymbol {
    let name = token_to_name(&pair.key);
    let detail = Some(token_kind_label(&pair.value.value).to_owned());
    let selection_range = pair.key.span.map_or_else(
        || Range::new(Position::new(0, 0), Position::new(0, 0)),
        |span| range_from_span(source, span),
    );

    let range_span = match (pair.key.span, pair.value.span) {
        (Some(key), Some(value)) => key | value,
        (Some(span), None) | (None, Some(span)) => span,
        (None, None) => eon_syntax::Span { start: 0, end: 0 },
    };
    let range = range_from_span(source, range_span);

    let children = match &pair.value.value {
        eon_syntax::TokenValue::Map(map) => {
            let nested = map
                .key_values
                .iter()
                .map(|nested| symbol_from_pair(nested, source))
                .collect::<Vec<_>>();
            if nested.is_empty() {
                None
            } else {
                Some(nested)
            }
        }
        _ => None,
    };

    #[expect(deprecated, reason = "lsp-types still exposes this field")]
    DocumentSymbol {
        name,
        detail,
        kind: symbol_kind_for_value(&pair.value.value),
        tags: None,
        deprecated: None,
        range,
        selection_range,
        children,
    }
}

fn token_to_name(token: &eon_syntax::TokenTree<'_>) -> String {
    match &token.value {
        eon_syntax::TokenValue::Identifier(value)
        | eon_syntax::TokenValue::Number(value)
        | eon_syntax::TokenValue::QuotedString(value) => value.to_string(),
        eon_syntax::TokenValue::List(_) => "[list]".to_owned(),
        eon_syntax::TokenValue::Map(_) => "{map}".to_owned(),
        eon_syntax::TokenValue::Variant(variant) => format!("{}(...)", variant.quoted_name),
    }
}

fn token_kind_label(value: &eon_syntax::TokenValue<'_>) -> &'static str {
    match value {
        eon_syntax::TokenValue::Identifier(_) => "identifier",
        eon_syntax::TokenValue::Number(_) => "number",
        eon_syntax::TokenValue::QuotedString(_) => "string",
        eon_syntax::TokenValue::List(_) => "list",
        eon_syntax::TokenValue::Map(_) => "map",
        eon_syntax::TokenValue::Variant(_) => "variant",
    }
}

fn symbol_kind_for_value(value: &eon_syntax::TokenValue<'_>) -> SymbolKind {
    match value {
        eon_syntax::TokenValue::Map(_) => SymbolKind::OBJECT,
        eon_syntax::TokenValue::List(_) => SymbolKind::ARRAY,
        eon_syntax::TokenValue::Variant(_) => SymbolKind::ENUM_MEMBER,
        _ => SymbolKind::PROPERTY,
    }
}

fn range_from_span(source: &str, span: eon_syntax::Span) -> Range {
    let start = position_at_byte_offset(source, span.start);
    let end = position_at_byte_offset(source, span.end.max(span.start));
    Range::new(start, end)
}

fn full_document_range(source: &str) -> Range {
    let end = position_at_byte_offset(source, source.len());
    Range::new(Position::new(0, 0), end)
}

fn position_at_byte_offset(source: &str, mut byte_offset: usize) -> Position {
    if byte_offset > source.len() {
        byte_offset = source.len();
    }
    while byte_offset > 0 && !source.is_char_boundary(byte_offset) {
        byte_offset -= 1;
    }

    let mut line = 0_u32;
    let mut line_start = 0_usize;
    for (index, ch) in source.char_indices() {
        if index >= byte_offset {
            break;
        }
        if ch == '\n' {
            line = line.saturating_add(1);
            line_start = index + 1;
        }
    }

    let character = source[line_start..byte_offset]
        .encode_utf16()
        .count()
        .try_into()
        .unwrap_or(u32::MAX);

    Position::new(line, character)
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend {
        client,
        state: Arc::new(RwLock::new(ServerState::default())),
    });

    Server::new(stdin, stdout, socket).serve(service).await;
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use eon_schema::{
        EnumSchema, FieldSchema, ObjectSchema, SchemaExtension, SchemaNode, VariantPayload,
        VariantSchema,
    };
    use serde_json::json;
    use tower_lsp::lsp_types::{CompletionItemKind, InitializeParams, InsertTextFormat, Url};

    use super::{
        DEFAULT_SCHEMA_FILE, completion_items_for_source_with_schema, diagnostics_for_document,
        discover_schema_path, format_source, load_schema_for_initialize_params,
        position_at_byte_offset, schema_path_from_initialization_options, symbols_from_token_tree,
    };

    fn test_dir(name: &str) -> PathBuf {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/eon_lsp_tests")
            .join(name);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn maps_utf16_positions() {
        let source = "name: \"😀\"\nkey: true\n";
        let emoji_start = source
            .find('😀')
            .expect("test source must contain emoji byte offset");
        let pos = position_at_byte_offset(source, emoji_start);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.character, 7);
    }

    #[test]
    fn parses_error_to_single_diagnostic() {
        let diagnostics = diagnostics_for_document("key: $value", None);
        assert_eq!(diagnostics.len(), 1);
        assert!(!diagnostics[0].message.is_empty());
    }

    #[test]
    fn composition_diagnostics_resolve_imports_relative_to_uri() {
        let dir = test_dir("composition_diagnostics_resolve_imports_relative_to_uri");
        let root = dir.join("root.eon");
        fs::write(
            dir.join("common.eon"),
            "database: { host: \"localhost\" }\n",
        )
        .unwrap();
        fs::write(&root, "").unwrap();
        let uri = Url::from_file_path(root).expect("file URI should be valid");

        let diagnostics = diagnostics_for_document(
            r#"
use: { common: "common.eon" }
host: $common.database.host
"#,
            Some(&uri),
        );

        assert!(diagnostics.is_empty(), "{diagnostics:?}");
    }

    #[test]
    fn composition_diagnostics_report_reference_errors() {
        let dir = test_dir("composition_diagnostics_report_reference_errors");
        let root = dir.join("root.eon");
        fs::write(dir.join("common.eon"), "database: {}\n").unwrap();
        fs::write(&root, "").unwrap();
        let uri = Url::from_file_path(root).expect("file URI should be valid");

        let diagnostics = diagnostics_for_document(
            r#"
use: { common: "common.eon" }
host: $common.database.host
"#,
            Some(&uri),
        );

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].source.as_deref(), Some("eon-compose"));
        assert!(
            diagnostics[0].message.contains("missing field `host`"),
            "{diagnostics:?}"
        );
    }

    #[test]
    fn reads_schema_path_from_initialization_options() {
        let options = json!({
            "eon": {
                "schemaPath": "schema.eon"
            }
        });

        let path = schema_path_from_initialization_options(Some(&options))
            .expect("schema path should be configured");

        assert_eq!(path, PathBuf::from("schema.eon"));
    }

    #[test]
    fn discovers_default_schema_file() {
        let dir = test_dir("discovers_default_schema_file");
        let schema_path = dir.join(DEFAULT_SCHEMA_FILE);
        fs::write(&schema_path, "kind: \"object\"\n").unwrap();

        let discovered = discover_schema_path(Some(&dir)).expect("schema should be discovered");

        assert_eq!(discovered, schema_path);
    }

    #[test]
    fn loads_schema_from_initialization_options() {
        let dir = test_dir("loads_schema_from_initialization_options");
        fs::write(
            dir.join("schema.eon"),
            r#"
kind: "object"
name: "Config"
fields: [
    { name: "port", type: "integer" }
]
"#,
        )
        .unwrap();

        let params = InitializeParams {
            root_uri: Some(Url::from_file_path(&dir).expect("file URI should be valid")),
            initialization_options: Some(json!({ "schemaPath": "schema.eon" })),
            ..InitializeParams::default()
        };

        let loaded = load_schema_for_initialize_params(&params)
            .expect("schema loading should succeed")
            .expect("schema should be loaded");

        let SchemaNode::Object(object) = loaded.schema else {
            panic!("expected object schema");
        };
        assert_eq!(object.fields[0].name, "port");
    }

    #[test]
    fn extracts_top_level_symbols() {
        let source = "root: { child: 1 }\nflag: true\n";
        let tree = eon_syntax::TokenTree::parse_str(source).expect("source should parse");
        let symbols = symbols_from_token_tree(&tree, source);
        let names = symbols
            .iter()
            .map(|symbol| symbol.name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(names, vec!["root", "flag"]);
    }

    #[test]
    fn completion_contains_keywords_and_map_keys() {
        let items =
            completion_items_for_source_with_schema("name: true\nconfig: { nested: 2 }\n", None);
        let labels = items
            .iter()
            .map(|item| item.label.as_str())
            .collect::<Vec<_>>();
        assert!(labels.contains(&"null"));
        assert!(labels.contains(&"name"));
        assert!(labels.contains(&"nested"));
    }

    #[test]
    fn completion_can_include_schema_fields() {
        let schema = SchemaNode::Object(ObjectSchema {
            name: "Config",
            docs: "",
            fields: vec![FieldSchema {
                name: "port",
                docs: "Server port.",
                ty: SchemaNode::Integer(eon_schema::IntegerSchema {
                    signed: false,
                    bits: 16,
                }),
                required: true,
                default: false,
                deprecated: None,
                extensions: Vec::<SchemaExtension>::new(),
            }],
            open: false,
            extensions: Vec::<SchemaExtension>::new(),
        });

        let items = completion_items_for_source_with_schema("name: true\n", Some(&schema));
        let port = items
            .iter()
            .find(|item| item.label == "port")
            .expect("schema field completion should be present");

        assert_eq!(port.kind, Some(CompletionItemKind::PROPERTY));
        assert_eq!(port.detail.as_deref(), Some("required field"));
        assert!(port.documentation.is_some());
    }

    #[test]
    fn completion_can_include_schema_variant_snippets() {
        let schema = SchemaNode::Enum(EnumSchema {
            name: "Color",
            docs: "",
            variants: vec![
                VariantSchema {
                    name: "Black",
                    docs: "",
                    payload: VariantPayload::Unit,
                    deprecated: None,
                    extensions: Vec::<SchemaExtension>::new(),
                },
                VariantSchema {
                    name: "Rgb",
                    docs: "RGB color.",
                    payload: VariantPayload::Tuple(vec![
                        SchemaNode::Any,
                        SchemaNode::Any,
                        SchemaNode::Any,
                    ]),
                    deprecated: None,
                    extensions: Vec::<SchemaExtension>::new(),
                },
            ],
            extensions: Vec::<SchemaExtension>::new(),
        });

        let items = completion_items_for_source_with_schema("", Some(&schema));
        let rgb = items
            .iter()
            .find(|item| item.label == "Rgb")
            .expect("schema variant completion should be present");

        assert_eq!(rgb.kind, Some(CompletionItemKind::ENUM_MEMBER));
        let mut expected = String::from("Rgb(");
        for index in 1..=3 {
            if index > 1 {
                expected.push_str(", ");
            }
            expected.push('$');
            expected.push('{');
            expected.push_str(&index.to_string());
            expected.push_str(":value");
            expected.push('}');
        }
        expected.push(')');
        assert_eq!(rgb.insert_text.as_deref(), Some(expected.as_str()));
        assert_eq!(rgb.insert_text_format, Some(InsertTextFormat::SNIPPET));
    }

    #[test]
    fn formatting_is_idempotent() {
        let source = "a:{b:1,c:[2,3]}";
        let once = format_source(source, true, 2).expect("source should format on first pass");
        let twice = format_source(&once, true, 2);
        assert!(twice.is_none(), "formatted source should already be stable");
    }
}

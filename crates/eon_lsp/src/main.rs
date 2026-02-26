//! Eon language server.

use std::{
    collections::{HashMap, HashSet},
    str::FromStr as _,
    sync::Arc,
};

use tokio::sync::RwLock;
use tower_lsp::{
    Client, LanguageServer, LspService, Server,
    jsonrpc::Result as JsonResult,
    lsp_types::{
        CompletionItem, CompletionItemKind, CompletionOptions, CompletionParams,
        CompletionResponse, Diagnostic, DiagnosticSeverity, DidChangeTextDocumentParams,
        DidCloseTextDocumentParams, DidOpenTextDocumentParams, DidSaveTextDocumentParams,
        DocumentFormattingParams, DocumentSymbol, DocumentSymbolParams, DocumentSymbolResponse,
        InitializeParams, InitializeResult, InitializedParams, MessageType, OneOf, Position, Range,
        ServerCapabilities, SymbolKind, TextDocumentSyncCapability, TextDocumentSyncKind, TextEdit,
        Url,
    },
};

#[derive(Default)]
struct ServerState {
    documents: HashMap<Url, String>,
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

    async fn publish_diagnostics(&self, uri: &Url) {
        let diagnostics = self
            .get_document(uri)
            .await
            .map_or_else(Vec::new, |text| diagnostics_for_text(&text));
        self.client
            .publish_diagnostics(uri.clone(), diagnostics, None)
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> JsonResult<InitializeResult> {
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
        let items = completion_items_for_source(&source);
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

fn diagnostics_for_text(source: &str) -> Vec<Diagnostic> {
    match eon::Value::from_str(source) {
        Ok(_) => Vec::new(),
        Err(err) => {
            let diagnostic = match err {
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
            };
            vec![diagnostic]
        }
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

fn completion_items_for_source(source: &str) -> Vec<CompletionItem> {
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

    items
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
    use super::{
        completion_items_for_source, diagnostics_for_text, format_source, position_at_byte_offset,
        symbols_from_token_tree,
    };

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
        let diagnostics = diagnostics_for_text("key: value");
        assert_eq!(diagnostics.len(), 1);
        assert!(!diagnostics[0].message.is_empty());
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
        let items = completion_items_for_source("name: true\nconfig: { nested: 2 }\n");
        let labels = items
            .iter()
            .map(|item| item.label.as_str())
            .collect::<Vec<_>>();
        assert!(labels.contains(&"null"));
        assert!(labels.contains(&"name"));
        assert!(labels.contains(&"nested"));
    }

    #[test]
    fn formatting_is_idempotent() {
        let source = "a:{b:1,c:[2,3]}";
        let once = format_source(source, true, 2).expect("source should format on first pass");
        let twice = format_source(&once, true, 2);
        assert!(twice.is_none(), "formatted source should already be stable");
    }
}

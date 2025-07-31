//! Serialize a [`TokenTree`] to an Eon string.

use crate::token_tree::{TokenKeyValue, TokenList, TokenMap, TokenTree, TokenValue, TokenVariant};

/// How to format an Eon document.
///
/// If you mess up the options too much (e.g. set the indentation to something that is not whitespace)
/// you might end up with a document that is not valid Eon syntax.
#[derive(Clone, Debug)]
pub struct FormatOptions {
    /// `"\t"`
    pub indentation: String,

    /// `"\n"`
    pub newline: String,

    /// `" "`
    pub space_before_suffix_comment: String,

    /// `": "`
    pub key_value_separator: String,

    /// Surround the top-level map in { } with an extra level of indentation.
    pub always_include_outer_braces: bool,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            // A tab character allows users to configure their preferred indentation size in their editor.
            // It's the best default.
            indentation: "\t".to_owned(),
            newline: "\n".to_owned(),
            space_before_suffix_comment: " ".to_owned(),
            key_value_separator: ": ".to_owned(),
            always_include_outer_braces: false,
        }
    }
}

impl FormatOptions {
    /// Create a new [`FormatOptions`] with the default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the indentation string.
    pub fn with_indentation(mut self, indentation: String) -> Self {
        self.indentation = indentation;
        self
    }

    /// Set the newline string.
    pub fn with_newline(mut self, newline: String) -> Self {
        self.newline = newline;
        self
    }
}

impl TokenTree<'_> {
    /// Format as an Eon string.
    pub fn format(&self, options: &FormatOptions) -> String {
        let mut f = Formatter::new(options);

        if !f.options.always_include_outer_braces {
            if let TokenValue::Map(map) = &self.value {
                f.indented_comments(&self.prefix_comments);
                f.map_content(map);
                f.suffix_comment(&self.suffix_comment);
                return f.finish();
            }
        }

        f.indented_value(self);
        f.finish()
    }
}

struct Formatter<'o> {
    options: &'o FormatOptions,
    indent: usize,
    out: String,
}

impl<'o> Formatter<'o> {
    fn new(options: &'o FormatOptions) -> Self {
        Self {
            options,
            indent: 0,
            out: String::new(),
        }
    }

    fn finish(self) -> String {
        debug_assert_eq!(
            self.indent, 0,
            "Formatter finished with non-zero indent of {}",
            self.indent
        );
        self.out
    }

    fn add_indent(&mut self) {
        for _ in 0..self.indent {
            self.out.push_str(&self.options.indentation);
        }
    }

    fn indented_comments(&mut self, comments: &[&str]) {
        for &comment in comments {
            self.add_indent();
            self.out.push_str(comment);
            self.out.push('\n');
        }
    }

    fn indented_value(&mut self, value: &TokenTree<'_>) {
        let TokenTree {
            prefix_comments,
            value,
            suffix_comment,
            span: _,
        } = value;
        self.indented_comments(prefix_comments);
        self.add_indent();
        self.value(value);
        self.suffix_comment(suffix_comment);
    }

    #[expect(clippy::ref_option_ref)]
    fn suffix_comment(&mut self, suffix_comment: &Option<&str>) {
        if let Some(suffix_comment) = suffix_comment {
            self.out.push(' ');
            self.out.push_str(suffix_comment);
        }
    }

    fn value(&mut self, value: &TokenValue<'_>) {
        match value {
            TokenValue::Identifier(slice)
            | TokenValue::Number(slice)
            | TokenValue::QuotedString(slice) => {
                self.out.push_str(slice);
            }
            TokenValue::List(list) => {
                self.list(list);
            }
            TokenValue::Map(map) => {
                self.map(map);
            }
            TokenValue::Variant(variant) => {
                self.variant(variant);
            }
        }
    }

    fn list(&mut self, list: &TokenList<'_>) {
        let TokenList {
            values,
            closing_comments,
        } = list;

        if list.values.is_empty() && closing_comments.is_empty() {
            self.out.push_str("[]");
            return;
        }

        if should_format_list_on_one_line(list) {
            self.out.push('[');
            for (i, value) in values.iter().enumerate() {
                self.value(&value.value);
                if i + 1 < values.len() {
                    self.out.push_str(", "); // We use commas for single-line lists, just for extra readability
                }
            }
            self.out.push(']');
        } else {
            self.out.push('[');
            self.indent += 1;
            self.out.push('\n');
            self.list_content(list);
            self.indent -= 1;
            self.add_indent();
            self.out.push(']');
        }
    }

    fn list_content(&mut self, list: &TokenList<'_>) {
        let TokenList {
            values,
            closing_comments,
        } = list;

        let add_blank_lines = values.iter().any(|v| !v.prefix_comments.is_empty());

        for (i, value) in values.iter().enumerate() {
            self.indented_value(value);
            self.out.push('\n');
            if add_blank_lines && i + 1 < values.len() {
                self.out.push('\n');
            }
        }
        if add_blank_lines && !closing_comments.is_empty() {
            self.out.push('\n');
        }
        self.indented_comments(closing_comments);
    }

    fn map(&mut self, map: &TokenMap<'_>) {
        let TokenMap {
            key_values,
            closing_comments,
        } = map;

        if key_values.is_empty() && closing_comments.is_empty() {
            self.out.push_str("{}");
            return;
        }

        self.out.push('{');
        self.indent += 1;
        self.out.push('\n');
        self.map_content(map);
        self.indent -= 1;
        self.add_indent();
        self.out.push('}');
    }

    fn map_content(&mut self, map: &TokenMap<'_>) {
        let TokenMap {
            key_values,
            closing_comments,
        } = map;

        let add_blank_lines = key_values
            .iter()
            .any(|kv| !kv.key.prefix_comments.is_empty());

        for (i, key_value) in key_values.iter().enumerate() {
            self.indented_key_value(key_value);
            self.out.push('\n');
            if add_blank_lines && i + 1 < key_values.len() {
                self.out.push('\n');
            }
        }

        if add_blank_lines && !closing_comments.is_empty() {
            self.out.push('\n');
        }
        self.indented_comments(closing_comments);
    }

    fn indented_key_value(&mut self, key_value: &TokenKeyValue<'_>) {
        let TokenKeyValue { key, value } = key_value;
        self.indented_comments(&key.prefix_comments);
        self.indented_comments(&value.prefix_comments);
        self.add_indent();
        self.value(&key.value);
        self.out.push_str(&self.options.key_value_separator);
        self.value(&value.value);
        self.suffix_comment(&value.suffix_comment);
    }

    fn variant(&mut self, variant: &TokenVariant<'_>) {
        let TokenVariant {
            name_span: _,
            quoted_name,
            values,
            closing_comments,
        } = variant;

        if values.is_empty() && closing_comments.is_empty() {
            self.out.push_str(quoted_name); // Omit parentheses if no values
            return;
        }

        if should_format_variant_on_one_line(variant) {
            self.out.push_str(quoted_name);
            self.out.push('(');
            for (i, value) in values.iter().enumerate() {
                self.value(&value.value);
                if i + 1 < values.len() {
                    self.out.push_str(", "); // We use commas for single-line variants, just for extra readability
                }
            }
            self.out.push(')');
        } else if closing_comments.is_empty()
            && values.len() == 1
            && matches!(values[0].value, TokenValue::Map(_))
        {
            let TokenValue::Map(map) = &values[0].value else {
                unreachable!() // TODO(emilk): replace with if-let chains
            };

            if map.key_values.is_empty() && map.closing_comments.is_empty() {
                self.out.push_str(quoted_name);
                self.out.push_str("({ })");
            } else {
                // A single map variant, like `"VariantName"({ key: value, … })`.
                // Here we avoid double-indenting for nicer/more compact output.
                self.out.push_str(quoted_name);
                self.out.push_str("({");
                self.indent += 1;
                self.out.push('\n');
                self.map_content(map);
                self.indent -= 1;
                self.add_indent();
                self.out.push_str("})");
            }
        } else if closing_comments.is_empty()
            && values.len() == 1
            && matches!(values[0].value, TokenValue::List(_))
        {
            let TokenValue::List(list) = &values[0].value else {
                unreachable!() // TODO(emilk): replace with if-let chains
            };

            if list.values.is_empty() && list.closing_comments.is_empty() {
                self.out.push_str(quoted_name);
                self.out.push_str("([ ])");
            } else {
                // A single list variant, like `"VariantName"({ key: value, … })`.
                // Here we avoid double-indenting for nicer/more compact output.
                self.out.push_str(quoted_name);
                self.out.push_str("([");
                self.indent += 1;
                self.out.push('\n');
                self.list_content(list);
                self.indent -= 1;
                self.add_indent();
                self.out.push_str("])");
            }
        } else {
            let add_blank_lines = values.iter().any(|v| !v.prefix_comments.is_empty());

            self.out.push_str(quoted_name);
            self.out.push('(');
            self.indent += 1;
            self.out.push('\n');
            for (i, value) in values.iter().enumerate() {
                self.indented_value(value);
                self.out.push('\n');
                if add_blank_lines && i + 1 < values.len() {
                    self.out.push('\n');
                }
            }
            if add_blank_lines && !closing_comments.is_empty() {
                self.out.push('\n');
            }
            self.indented_comments(closing_comments);
            self.indent -= 1;
            self.add_indent();
            self.out.push(')');
        }
    }
}

fn should_format_list_on_one_line(list: &TokenList<'_>) -> bool {
    let TokenList {
        values,
        closing_comments,
    } = list;
    closing_comments.is_empty() && should_format_values_on_one_line(values)
}

fn should_format_variant_on_one_line(variant: &TokenVariant<'_>) -> bool {
    let TokenVariant {
        name_span: _,
        quoted_name: _,
        values,
        closing_comments,
    } = variant;
    closing_comments.is_empty() && should_format_values_on_one_line(values)
}

fn should_format_values_on_one_line(values: &[TokenTree<'_>]) -> bool {
    if !values.iter().all(is_simple) {
        return false;
    }

    if values.len() <= 4 && values.iter().all(|tt| tt.value.is_number()) {
        return true; // e.g. [1 2 3 4]
    }

    if values.len() > 4 {
        return false;
    }

    let mut estimated_width = 0;
    for value in values {
        if let TokenValue::QuotedString(string) = &value.value {
            estimated_width += string.len();
        } else {
            estimated_width += 5;
        }
        estimated_width += 2;
    }

    estimated_width < 60
}

fn is_simple(value: &TokenTree<'_>) -> bool {
    if value.prefix_comments.is_empty() && value.suffix_comment.is_none() {
        match &value.value {
            TokenValue::Identifier(_) | TokenValue::Number(_) => true,

            TokenValue::QuotedString(string) => !string.contains('\n'),

            TokenValue::List(list) => {
                let TokenList {
                    values,
                    closing_comments,
                } = list;
                values.is_empty() && closing_comments.is_empty()
            }

            TokenValue::Map(map) => {
                let TokenMap {
                    key_values,
                    closing_comments,
                } = map;
                key_values.is_empty() && closing_comments.is_empty()
            }

            TokenValue::Variant(variant) => {
                let TokenVariant {
                    name_span: _,
                    quoted_name: _,
                    values,
                    closing_comments,
                } = variant;
                values.is_empty() && closing_comments.is_empty()
            }
        }
    } else {
        false
    }
}

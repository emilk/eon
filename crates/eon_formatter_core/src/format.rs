use alloc::{borrow::ToOwned, string::String};

use crate::{
    Document, KeyValue, List, Map, Result, Value, ValueTree, Variant, VariantName, parse_document,
};

/// Formatting options for the minimal formatter path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormatOptions {
    /// Indentation prefix used for each nesting level.
    pub indentation: String,
    /// Newline sequence inserted by the formatter.
    pub newline: String,
    /// Separator placed before an inline suffix comment.
    pub space_before_suffix_comment: String,
    /// Separator placed between map keys and values.
    pub key_value_separator: String,
    /// Whether the formatter should always print outer braces around root maps.
    pub always_include_outer_braces: bool,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            indentation: "\t".to_owned(),
            newline: "\n".to_owned(),
            space_before_suffix_comment: " ".to_owned(),
            key_value_separator: ": ".to_owned(),
            always_include_outer_braces: false,
        }
    }
}

impl FormatOptions {
    /// Create default formatting options.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
}

/// Parse and reformat an Eon document using the minimal formatter path.
pub fn reformat(source: &str, options: &FormatOptions) -> Result<String> {
    parse_document(source).map(|document| document.format(options))
}

impl Document<'_> {
    /// Format this parsed document into canonical Eon text.
    pub fn format(&self, options: &FormatOptions) -> String {
        let mut formatter = Formatter::new(options);

        if !formatter.options.always_include_outer_braces {
            if let Value::Map(map) = &self.root.value {
                if !map.key_values.is_empty() {
                    // Match the legacy formatter's canonical root-map shape
                    // and omit outer braces for non-empty root maps.
                    //
                    // Empty root maps cannot be represented without braces, so
                    // keep them explicit even when outer braces are normally
                    // omitted.
                    formatter.indented_comments(&self.root.prefix_comments);
                    formatter.map_content(map);
                    if self.root.suffix_comment.is_some() || !self.trailing_comments.is_empty() {
                        formatter.newline();
                    }
                    formatter.trailing_comments(
                        self.root.suffix_comment,
                        &self.trailing_comments,
                    );
                    return formatter.finish();
                }
            }
        }

        formatter.indented_value(&self.root);
        if !self.trailing_comments.is_empty() {
            formatter.newline();
            formatter.indented_comments(&self.trailing_comments);
        }
        formatter.finish()
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
        self.out
    }

    fn newline(&mut self) {
        self.out.push_str(&self.options.newline);
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
            self.newline();
        }
    }

    fn indented_value(&mut self, value: &ValueTree<'_>) {
        self.indented_comments(&value.prefix_comments);
        self.add_indent();
        self.value(&value.value);
        self.suffix_comment(value.suffix_comment);
    }

    fn suffix_comment(&mut self, suffix_comment: Option<&str>) {
        if let Some(comment) = suffix_comment {
            self.out.push_str(&self.options.space_before_suffix_comment);
            self.out.push_str(comment);
        }
    }

    fn trailing_comments(&mut self, suffix_comment: Option<&str>, trailing_comments: &[&str]) {
        if let Some(comment) = suffix_comment {
            self.add_indent();
            self.out.push_str(comment);
            self.newline();
        }

        self.indented_comments(trailing_comments);
    }

    fn value(&mut self, value: &Value<'_>) {
        match value {
            Value::Identifier(text) | Value::Number(text) | Value::QuotedString(text) => {
                self.out.push_str(text);
            }
            Value::List(list) => self.list(list),
            Value::Map(map) => self.map(map),
            Value::Variant(variant) => self.variant(variant),
        }
    }

    fn list(&mut self, list: &List<'_>) {
        if list.values.is_empty() && list.closing_comments.is_empty() {
            self.out.push_str("[]");
            return;
        }

        if should_format_list_on_one_line(list) {
            self.out.push('[');
            for (index, value) in list.values.iter().enumerate() {
                self.value(&value.value);
                if index + 1 < list.values.len() {
                    self.out.push_str(", ");
                }
            }
            self.out.push(']');
        } else {
            self.out.push('[');
            self.indent += 1;
            self.newline();
            self.list_content(list);
            self.indent -= 1;
            self.add_indent();
            self.out.push(']');
        }
    }

    fn list_content(&mut self, list: &List<'_>) {
        for (index, value) in list.values.iter().enumerate() {
            if index > 0 && !value.prefix_comments.is_empty() {
                self.newline();
            }
            self.indented_value(value);
            self.newline();
        }

        if !list.closing_comments.is_empty() {
            if !list.values.is_empty() {
                self.newline();
            }
            self.indented_comments(&list.closing_comments);
        }
    }

    fn map(&mut self, map: &Map<'_>) {
        if map.key_values.is_empty() && map.closing_comments.is_empty() {
            self.out.push_str("{}");
            return;
        }

        self.out.push('{');
        self.indent += 1;
        self.newline();
        self.map_content(map);
        self.indent -= 1;
        self.add_indent();
        self.out.push('}');
    }

    fn map_content(&mut self, map: &Map<'_>) {
        for (index, key_value) in map.key_values.iter().enumerate() {
            if index > 0 && !key_value.key.prefix_comments.is_empty() {
                self.newline();
            }
            self.indented_key_value(key_value);
            self.newline();
        }

        if !map.closing_comments.is_empty() {
            if !map.key_values.is_empty() {
                self.newline();
            }
            self.indented_comments(&map.closing_comments);
        }
    }

    fn indented_key_value(&mut self, key_value: &KeyValue<'_>) {
        self.indented_comments(&key_value.key.prefix_comments);
        self.indented_comments(&key_value.value.prefix_comments);
        self.add_indent();
        self.value(&key_value.key.value);
        self.out.push_str(&self.options.key_value_separator);
        self.value(&key_value.value.value);
        self.suffix_comment(key_value.value.suffix_comment);
    }

    fn variant(&mut self, variant: &Variant<'_>) {
        if variant.values.is_empty() && variant.closing_comments.is_empty() {
            self.write_variant_name(variant.name);
            return;
        }

        let single_comment_free_payload = variant.values.len() == 1
            && variant.values[0].prefix_comments.is_empty()
            && variant.values[0].suffix_comment.is_none();

        if should_format_variant_on_one_line(variant) {
            self.write_variant_name(variant.name);
            self.out.push('(');
            for (index, value) in variant.values.iter().enumerate() {
                self.value(&value.value);
                if index + 1 < variant.values.len() {
                    self.out.push_str(", ");
                }
            }
            self.out.push(')');
        } else if variant.closing_comments.is_empty()
            && single_comment_free_payload
            && matches!(variant.values[0].value, Value::Map(_))
        {
            let Value::Map(map) = &variant.values[0].value else {
                unreachable!();
            };

            self.write_variant_name(variant.name);
            if map.key_values.is_empty() && map.closing_comments.is_empty() {
                self.out.push_str("({})");
            } else {
                self.out.push_str("({");
                self.indent += 1;
                self.newline();
                self.map_content(map);
                self.indent -= 1;
                self.add_indent();
                self.out.push_str("})");
            }
        } else if variant.closing_comments.is_empty()
            && single_comment_free_payload
            && matches!(variant.values[0].value, Value::List(_))
        {
            let Value::List(list) = &variant.values[0].value else {
                unreachable!();
            };

            self.write_variant_name(variant.name);
            if list.values.is_empty() && list.closing_comments.is_empty() {
                self.out.push_str("([])");
            } else {
                self.out.push_str("([");
                self.indent += 1;
                self.newline();
                self.list_content(list);
                self.indent -= 1;
                self.add_indent();
                self.out.push_str("])");
            }
        } else {
            self.write_variant_name(variant.name);
            self.out.push('(');
            self.indent += 1;
            self.newline();
            for (index, value) in variant.values.iter().enumerate() {
                if index > 0 && !value.prefix_comments.is_empty() {
                    self.newline();
                }
                self.indented_value(value);
                self.newline();
            }

            if !variant.closing_comments.is_empty() {
                if !variant.values.is_empty() {
                    self.newline();
                }
                self.indented_comments(&variant.closing_comments);
            }

            self.indent -= 1;
            self.add_indent();
            self.out.push(')');
        }
    }

    fn write_variant_name(&mut self, name: VariantName<'_>) {
        match name {
            VariantName::Identifier(text) | VariantName::Quoted(text) => self.out.push_str(text),
        }
    }
}

fn should_format_list_on_one_line(list: &List<'_>) -> bool {
    list.closing_comments.is_empty() && should_format_values_on_one_line(&list.values)
}

fn should_format_variant_on_one_line(variant: &Variant<'_>) -> bool {
    variant.closing_comments.is_empty() && should_format_values_on_one_line(&variant.values)
}

fn should_format_values_on_one_line(values: &[ValueTree<'_>]) -> bool {
    if !values.iter().all(is_simple) {
        return false;
    }

    if values.len() <= 4 && values.iter().all(|value| value.value.is_number()) {
        return true;
    }

    if values.len() > 4 {
        return false;
    }

    let mut estimated_width = 0;
    for value in values {
        if let Value::QuotedString(string) = &value.value {
            estimated_width += string.len();
        } else {
            estimated_width += 5;
        }
        estimated_width += 2;
    }

    estimated_width < 60
}

fn is_simple(value: &ValueTree<'_>) -> bool {
    if !value.prefix_comments.is_empty() || value.suffix_comment.is_some() {
        return false;
    }

    match &value.value {
        Value::Identifier(_) | Value::Number(_) => true,
        Value::QuotedString(string) => !string.contains('\n'),
        Value::List(list) => list.values.is_empty() && list.closing_comments.is_empty(),
        Value::Map(map) => map.key_values.is_empty() && map.closing_comments.is_empty(),
        Value::Variant(variant) => variant.values.is_empty() && variant.closing_comments.is_empty(),
    }
}

#[cfg(test)]
mod tests {
    use super::{FormatOptions, reformat};

    #[test]
    fn reformat_implicit_root_map_and_variants() {
        let input = r#"
            // outside
            key: true// suffix

            variants: [
                EnumValue
                "Quoted"({ foo: 1, bar: 2 })
            ]
        "#;

        let formatted = reformat(input, &FormatOptions::default()).unwrap();
        assert_eq!(
            formatted,
            "// outside\nkey: true // suffix\nvariants: [\n\tEnumValue\n\t\"Quoted\"({\n\t\tfoo: 1\n\t\tbar: 2\n\t})\n]\n"
        );
    }

    #[test]
    fn reformat_root_scalar_with_variant_payload() {
        let formatted =
            reformat("EnumValue({ foo: [1, 2, 3] })", &FormatOptions::default()).unwrap();
        assert_eq!(formatted, "EnumValue({\n\tfoo: [1, 2, 3]\n})");
    }

    #[test]
    fn reformat_keeps_empty_root_map_explicit() {
        let formatted = reformat("{}", &FormatOptions::default()).unwrap();
        assert_eq!(formatted, "{}");
    }

    #[test]
    fn reformat_root_map_suffix_comment_becomes_trailing_comment_line() {
        let formatted = reformat("{ alpha: 1 } // tail", &FormatOptions::default()).unwrap();
        assert_eq!(formatted, "alpha: 1\n\n// tail\n");
    }

    #[test]
    fn reformat_single_root_value_keeps_trailing_comments_without_wrapping() {
        let formatted = reformat("1\n// tail\n", &FormatOptions::default()).unwrap();
        assert_eq!(formatted, "1\n// tail\n");
    }

    #[test]
    fn reformat_matches_complex_legacy_case() {
        let input = r#"
            // This comment is outside the outermost map.
            {
                // This comment proceeds the first key-value pair.
                key: true// Suffix comment


                // Comment about the second key-value pair.
                key:
                // Very weird comment
                null

                empty_map: {}
                empty_list: []
                short_list: [1, 2, 3]

                variants: [
                    "zero_variant"()
                    "one_variant"(true)
                    "three_variant"(1, 2, 3)
                    "map_variant"({
                        "key": "value",
                        "another_key": 42,
                    })
                    "list_variant"([
                        "doc",
                        "grumpy",
                        "happy",
                        "sleepy",
                        "sneezy",
                        "bashful",
                        "dopey",
                    ])
                ]
            }
        "#;

        let formatted = reformat(input, &FormatOptions::default()).unwrap();
        assert_eq!(
            formatted,
            "// This comment is outside the outermost map.\n// This comment proceeds the first key-value pair.\nkey: true // Suffix comment\n\n// Comment about the second key-value pair.\n// Very weird comment\nkey: null\nempty_map: {}\nempty_list: []\nshort_list: [1, 2, 3]\nvariants: [\n\t\"zero_variant\"\n\t\"one_variant\"(true)\n\t\"three_variant\"(1, 2, 3)\n\t\"map_variant\"({\n\t\t\"key\": \"value\"\n\t\t\"another_key\": 42\n\t})\n\t\"list_variant\"([\n\t\t\"doc\"\n\t\t\"grumpy\"\n\t\t\"happy\"\n\t\t\"sleepy\"\n\t\t\"sneezy\"\n\t\t\"bashful\"\n\t\t\"dopey\"\n\t])\n]\n"
        );
    }
}

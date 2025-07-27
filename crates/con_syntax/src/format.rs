//! Serialize a [`TokenTree`] to a Con string.

use crate::token_tree::{TokenChoice, TokenKeyValue, TokenList, TokenMap, TokenTree, TokenValue};

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
            indentation: "\t".to_owned(), // A tab character allows users to configure their preferred indentation size in their editor
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
    /// Format as a Con string.
    pub fn format(&self, options: &FormatOptions) -> String {
        let mut f = Formatter::new(options);

        if !f.options.always_include_outer_braces {
            if let TokenValue::Map(map) = &self.value {
                f.indented_comments(&self.prefix_comments);
                f.map_content(map);
                if let Some(suffix_comment) = self.suffix_comment {
                    f.out.push(' ');
                    f.out.push_str(suffix_comment);
                    f.out.push_str(&f.options.newline);
                }
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
        if let Some(suffix) = suffix_comment {
            self.out.push(' ');
            self.out.push_str(suffix);
            self.out.push('\n');
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
            TokenValue::Choice(choice) => {
                self.choice(choice);
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
            for value in values {
                self.indented_value(value);
                self.out.push('\n'); // TODO: only if the values have prefix comments
            }
            self.indented_comments(closing_comments);
            self.indent -= 1;
            self.add_indent();
            self.out.push(']');
        }
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

        for key_value in key_values {
            self.indented_key_value(key_value);
            self.out.push('\n'); // TODO: only if the keys have prefix comments
        }
        self.indented_comments(closing_comments);
    }

    fn indented_key_value(&mut self, key_value: &TokenKeyValue<'_>) {
        let TokenKeyValue { key, value } = key_value;
        self.indented_comments(&key.prefix_comments);
        self.indented_comments(&value.prefix_comments);
        self.add_indent();
        self.value(&key.value); // TODO: handle optional quotes around keys
        self.out.push_str(&self.options.key_value_separator);
        self.value(&value.value);
        if let Some(suffix) = value.suffix_comment {
            self.out.push(' ');
            self.out.push_str(suffix);
            self.out.push('\n');
        }
    }

    fn choice(&mut self, choice: &TokenChoice<'_>) {
        let TokenChoice {
            name_span: _,
            quoted_name,
            values,
            closing_comments,
        } = choice;

        if values.is_empty() && closing_comments.is_empty() {
            self.out.push_str(quoted_name); // Omit parentheses if no values
            return;
        }

        if should_format_choice_on_one_line(choice) {
            self.out.push_str(quoted_name);
            self.out.push('(');
            for (i, value) in values.iter().enumerate() {
                self.value(&value.value);
                if i + 1 < values.len() {
                    self.out.push_str(", "); // We use commas for single-line choices, just for extra readability
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
                // A single map choice, like `"ChoiceName"({ key: value, … })`.
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
        } else {
            self.out.push_str(quoted_name);
            self.out.push('(');
            self.indent += 1;
            self.out.push('\n');
            for value in values {
                self.indented_value(value);
                self.out.push('\n'); // TODO: only if the values have prefix comments
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

fn should_format_choice_on_one_line(choice: &TokenChoice<'_>) -> bool {
    let TokenChoice {
        name_span: _,
        quoted_name: _,
        values,
        closing_comments,
    } = choice;
    closing_comments.is_empty() && should_format_values_on_one_line(values)
}

fn should_format_values_on_one_line(values: &[TokenTree<'_>]) -> bool {
    if values.len() <= 4 && values.iter().all(|tt| tt.value.is_number()) {
        return true; // e.g. [1 2 3 4]
    }

    if values.len() > 4 {
        return false;
    }

    let mut estimated_width = 0;
    for value in values {
        if !is_simple(value) {
            return false;
        }
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
            TokenValue::Identifier(_) | TokenValue::Number(_) | TokenValue::QuotedString(_) => true,

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

            TokenValue::Choice(choice) => {
                let TokenChoice {
                    name_span: _,
                    quoted_name: _,
                    values,
                    closing_comments,
                } = choice;
                values.is_empty() && closing_comments.is_empty()
            }
        }
    } else {
        false
    }
}

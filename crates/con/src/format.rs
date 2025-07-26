//! Serialize a [`TokenTree`] to a Con string.

use crate::{
    Value,
    token_tree::{
        CommentedChoice, CommentedKeyValue, CommentedList, CommentedMap, TokenTree, TreeValue,
    },
};

#[derive(Clone, Debug)]
pub struct FormatOptions {
    /// `"    "`
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
            indentation: "    ".to_owned(),
            newline: "\n".to_owned(),
            space_before_suffix_comment: " ".to_owned(),
            key_value_separator: ": ".to_owned(),
            always_include_outer_braces: false,
        }
    }
}

impl TokenTree<'_> {
    /// Pretty-print a [`CommentedValue`] to a string.
    pub fn format(&self, options: &FormatOptions) -> String {
        let mut f = Formatter::new(options);

        if !f.options.always_include_outer_braces {
            if let TreeValue::Map(map) = &self.value {
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

        f.indented_commented_value(self);
        f.finish()
    }
}

impl Value {
    /// Pretty-print a [`Value`] to a string.
    pub fn format(&self, options: &FormatOptions) -> String {
        TokenTree::from(self.clone()).format(options)
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

    fn indented_commented_value(&mut self, value: &TokenTree<'_>) {
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

    fn value(&mut self, value: &TreeValue<'_>) {
        match value {
            TreeValue::Identifier(slice)
            | TreeValue::Number(slice)
            | TreeValue::QuotedString(slice) => {
                self.out.push_str(slice);
            }
            TreeValue::List(list) => {
                self.list(list);
            }
            TreeValue::Map(map) => {
                self.map(map);
            }
            TreeValue::Choice(choice) => {
                self.choice(choice);
            }
        }
    }

    fn list(&mut self, list: &CommentedList<'_>) {
        let CommentedList {
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
                self.indented_commented_value(value);
                self.out.push('\n'); // TODO: only if the values have prefix comments
            }
            self.indented_comments(closing_comments);
            self.indent -= 1;
            self.add_indent();
            self.out.push(']');
        }
    }

    fn map(&mut self, map: &CommentedMap<'_>) {
        let CommentedMap {
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

    fn map_content(&mut self, map: &CommentedMap<'_>) {
        let CommentedMap {
            key_values,
            closing_comments,
        } = map;

        for key_value in key_values {
            self.indented_key_value(key_value);
            self.out.push('\n'); // TODO: only if the keys have prefix comments
        }
        self.indented_comments(closing_comments);
    }

    fn indented_key_value(&mut self, key_value: &CommentedKeyValue<'_>) {
        let CommentedKeyValue { key, value } = key_value;
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

    fn choice(&mut self, choice: &CommentedChoice<'_>) {
        let CommentedChoice {
            name_span: _,
            name,
            values,
            closing_comments,
        } = choice;

        if values.is_empty() && closing_comments.is_empty() {
            self.out.push_str(name); // Omit parentheses if no values
            return;
        }

        // TODO: single-line short-form for single value, e.g. `choice(value)`
        // TODO: put {} braces on next to () for variants containing a single map

        self.out.push_str(name);
        self.out.push('(');
        self.indent += 1;
        self.out.push('\n');
        for value in values {
            self.indented_commented_value(value);
            self.out.push('\n'); // TODO: only if the values have prefix comments
        }
        self.indented_comments(closing_comments);
        self.indent -= 1;
        self.add_indent();
        self.out.push(')');
    }
}

fn should_format_list_on_one_line(list: &CommentedList<'_>) -> bool {
    if list.values.len() <= 4 && list.values.iter().all(|tt| tt.value.is_number()) {
        return true; // e.g. [1 2 3 4]
    }

    if list.values.len() > 4 {
        return false;
    }

    let mut estimated_width = 0;
    for value in &list.values {
        if !is_simple(value) {
            return false;
        }
        if let TreeValue::QuotedString(string) = &value.value {
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
            TreeValue::Identifier(_) | TreeValue::Number(_) | TreeValue::QuotedString(_) => true,

            TreeValue::List(list) => {
                let CommentedList {
                    values,
                    closing_comments,
                } = list;
                values.is_empty() && closing_comments.is_empty()
            }

            TreeValue::Map(map) => {
                let CommentedMap {
                    key_values,
                    closing_comments,
                } = map;
                key_values.is_empty() && closing_comments.is_empty()
            }

            TreeValue::Choice(choice) => {
                let CommentedChoice {
                    name_span: _,
                    name: _,
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

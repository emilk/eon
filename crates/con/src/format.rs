use crate::ast::{CommentedValue, KeyValue, List, Object, Value};

#[derive(Clone, Debug)]
pub struct FormatOptions {
    pub indentation: String,
    pub newline: String,
    pub space_before_suffix_comment: String,
    pub key_value_separator: String,
    pub always_include_outer_braces: bool,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            indentation: "    ".to_owned(), // TODO: what should be the default?
            newline: "\n".to_owned(),
            space_before_suffix_comment: " ".to_owned(),
            key_value_separator: ": ".to_owned(),
            always_include_outer_braces: false,
        }
    }
}

/// Pretty-print a [`CommentedValue`].
pub fn format(value: &CommentedValue<'_>, options: &FormatOptions) -> String {
    let mut f = Formatter::new(options);

    if !f.options.always_include_outer_braces {
        if let Value::Object(object) = &value.value {
            f.indented_comments(&value.prefix_comments);
            f.object_content(object);
            if let Some(suffix_comment) = value.suffix_comment {
                f.out.push(' ');
                f.out.push_str(suffix_comment);
                f.out.push_str(&f.options.newline);
            }
            return f.finish();
        }
    }

    f.indented_commented_value(value);
    f.finish()
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

    fn indented_commented_value(&mut self, value: &CommentedValue<'_>) {
        let CommentedValue {
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

    fn value(&mut self, value: &Value<'_>) {
        match value {
            Value::Identifier(slice) | Value::String(slice) => {
                self.out.push_str(slice);
            }
            Value::List(list) => {
                self.list(list);
            }
            Value::Object(object) => {
                self.object(object);
            }
        }
    }

    fn list(&mut self, list: &List<'_>) {
        // TODO: one-line lists when very short, e.g. `[1 2 3]`
        let List {
            values,
            closing_comments,
        } = list;

        if list.values.is_empty() && closing_comments.is_empty() {
            self.out.push_str("[ ]");
            return;
        }

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

    fn object(&mut self, value: &Object<'_>) {
        // TODO: one-line objects when possible
        self.out.push('{');
        self.indent += 1;
        self.out.push('\n');
        self.object_content(value);
        self.indent -= 1;
        self.out.push('}');
    }

    fn object_content(&mut self, value: &Object<'_>) {
        let Object {
            key_values,
            closing_comments,
        } = value;
        for key_value in key_values {
            self.indented_key_value(key_value);
            self.out.push('\n'); // TODO: only if the keys have prefix comments
        }
        self.indented_comments(closing_comments);
    }

    fn indented_key_value(&mut self, key_value: &KeyValue<'_>) {
        let KeyValue { key, value } = key_value;
        let CommentedValue {
            prefix_comments,
            value,
            suffix_comment,
            span: _,
        } = value;
        self.indented_comments(prefix_comments);
        self.add_indent();
        self.out.push_str(key.slice);
        self.out.push_str(&self.options.key_value_separator);
        self.value(value);
        if let Some(suffix) = suffix_comment {
            self.out.push(' ');
            self.out.push_str(suffix);
            self.out.push('\n');
        }
    }
}

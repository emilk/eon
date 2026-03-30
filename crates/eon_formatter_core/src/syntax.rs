use alloc::vec::Vec;

/// `// A comment`.
pub type Comment<'a> = &'a str;

/// Formatter-oriented document tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document<'a> {
    /// Root value of the document.
    pub root: ValueTree<'a>,
    /// Whether the root map was implicit rather than surrounded by `{}`.
    pub implicit_root_map: bool,
}

/// A syntax value plus attached comments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueTree<'a> {
    /// Comments on lines before the value.
    pub prefix_comments: Vec<Comment<'a>>,
    /// The actual syntax value.
    pub value: Value<'a>,
    /// Inline `// comment` on the same line as the value.
    pub suffix_comment: Option<Comment<'a>>,
}

/// Value kinds preserved for formatting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value<'a> {
    /// Identifier token.
    Identifier(&'a str),
    /// Number token.
    Number(&'a str),
    /// Quoted string token.
    QuotedString(&'a str),
    /// List value.
    List(List<'a>),
    /// Map value.
    Map(Map<'a>),
    /// Variant value.
    Variant(Variant<'a>),
}

/// One map entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyValue<'a> {
    /// Key value tree.
    pub key: ValueTree<'a>,
    /// Value tree.
    pub value: ValueTree<'a>,
}

/// Map contents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Map<'a> {
    /// Key/value entries in source order.
    pub key_values: Vec<KeyValue<'a>>,
    /// Comments before the closing brace or at the end of an implicit map.
    pub closing_comments: Vec<Comment<'a>>,
}

/// List contents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct List<'a> {
    /// Values in source order.
    pub values: Vec<ValueTree<'a>>,
    /// Comments before the closing bracket or at the end of a root implicit list.
    pub closing_comments: Vec<Comment<'a>>,
}

/// Variant contents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variant<'a> {
    /// Variant name token.
    pub name: VariantName<'a>,
    /// Payload values.
    pub values: Vec<ValueTree<'a>>,
    /// Comments before the closing parenthesis.
    pub closing_comments: Vec<Comment<'a>>,
}

/// Variant name token family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariantName<'a> {
    /// Identifier variant head.
    Identifier(&'a str),
    /// Quoted-string variant head.
    Quoted(&'a str),
}

impl Value<'_> {
    /// Returns `true` for number values.
    #[inline]
    pub fn is_number(&self) -> bool {
        matches!(self, Self::Number(_))
    }
}

impl<'a> From<Value<'a>> for ValueTree<'a> {
    fn from(value: Value<'a>) -> Self {
        Self {
            prefix_comments: Vec::new(),
            value,
            suffix_comment: None,
        }
    }
}

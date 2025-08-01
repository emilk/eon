# Eon: the human-friendly configuration format
Eon is a configuration format that is designed to be familiar, clean, and powerful.

Example Eon document:

```yaml
// Comment
string: "Hello Eon!"
list: [1, 2, 3]
object: {
    boolean: true
    regex: '\d{3}-\d{3}-\d{4}'
}
map: {
    1: "map keys don't need to be strings"
    2: "they can be any Eon value"
}
special_floats: [+inf, -inf, +nan]
```

Read more about Eon at <https://github.com/emilk/eon>.

This crate provides a parser and formatter for Eon.
It is used to implement the [`eonfmt`](http://crates.io/crates/eonfmt) formatter tool,
but also used by the [`eon`](http://crates.io/crates/eon) crate to parse and format Eon documents.

You can use it to read and write Eon documents, with comments.
This can be useful for e.g. reading "docstrings" from an `.eon` file,
or for automate the editing an `.eon` file while preserving comments and formatting.

# Eon: the human-friendly configuration format
To learn the Eon syntax, see <https://github.com/emilk/eon>.

This crate provides a parser and formatter for Eon.
It is used to implement the [`eonfmt`](http://crates.io/crates/eonfmt) formatter tool,
but also used by the [`eon`](http://crates.io/crates/eon) crate to parse and format Eon documents.

You can use it to read and write Eon documents, with comments.
This can be useful for e.g. reading "docstrings" from an `.eon` file,
or for automate the editing an `.eon` file while preserving comments and formatting.

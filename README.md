# CON - The nice config format
This repository contains the definition of _Con_, a new config format designed for human editing.

It also contains a Rust crate for reading, parsing, and formatting Con files.

## In short
```c
// comments
key: "value"
list: [1 2 3]
special_numbers: [+inf -inf +NaN]
object: {
    key: "value"
}
```

## Why another config format?
There are two things I want from a config file format:
* Hierarchy using {} and [] like JSON. Rules out YAML and TOML.
* No top-level {} wrapping the whole file. Rules out JSON5, RON, and others.

## Roadmap
* [ ] Parser
* [ ] Formatter
* [ ] Serde

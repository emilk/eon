# Con - The nice config format
This repository contains the definition of _Con_, a new config format designed for human editing.

It also contains a Rust crate for reading, parsing, and formatting Con files.

## In short
```c
// comments
key: "value"
list: [1 2 3]
special_floats: [+inf -inf +NaN]
object: {
    key: "value"
}
```

Con is a strict superset of JSON.

The library is rather forgiving with what it accepts as input, but the output is strict.

## Why another config format?
There is no format that has both of the following properties:
* Hierarchy using {} and [] like JSON. Rules out YAML and TOML.
* No top-level {} wrapping the whole file. Rules out JSON5, RON, and others.


### Why not JSON5?
JSON5 is _almost_ great, but requires wrapping the whole file in an extra `{ }` block, and indent that. That's too ugly for me.
It also has a bunch of unnessary commas between values.

RON has the same problem.

### Why not RON?
The goal of RON is to perfectly map the Rust datatypes, which is cool, but it means it leans more towards being verobse and complex while Con wants to be lean and simple.

### Why not TOML?
TOML is a hierarchical format, but unlike almost every other programming language known, it does not use any indentation to visually aid the reader, leading to very confusing hierarchies.

### Why not YAML?
It's just so ugly, and filled with foot-guns. Go away.

## Performance
The Con language and library are designed for human-readable, human-sized config files.

There is nothing in the Con spec that would not make it as fast (or as slow) as JSON,
but the library has not been optimized for performance.

If you plan on having huge files and performance is important to you, I suggest you use a binary format instead.


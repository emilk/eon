# Con - The human-friendly config format
This repository contains the definition of _Con_, a new config format designed for human editing.

Con uses the `.con` file ending.

It also contains a Rust crate for using Con with `serde`, and a binary for formatting Con files.

## In short
```c
// Comment
string: "Hello Con!"
list: [1 2 3]
map: {
    bool: true
    key: 'Use single quotes if your string contains "quotes"'
}
special_floats: [+inf, -inf, +NaN]
```

Con is designed to be
* **Familiar**: strongly resembles JSON
* **Clean**: forgoes unnecessary commas, quotes, and indentation
* **Clear**: lists are enclosed in `[ ]`, maps with `{ }`
* **Powerful**: supports sum types, like Rust enums

Con is a strict superset of JSON, i.e. any JSON file is also valid Con.

## Why another config format?
I wanted a format designed for human eyes with
* Indented hierarchy using `{ }` and `[ ]` (like JSON, C, Rust, …). Rules out YAML and TOML.
* No top-level `{ }` wrapping the whole file. Rules out JSON5, RON, and others.


### Why not JSON5?
JSON5 is _almost_ great, but requires wrapping the whole file in an extra `{ }` block, and indent that. That's too ugly for me.
It also has a bunch of unnecessary commas between values.

RON has the same problem.

### Why not RON?
The goal of RON is to perfectly map the Rust datatypes, which is cool, but it means it leans more towards being verobse and complex while Con wants to be lean and simple.

### Why not TOML?
TOML is a hierarchical format, but unlike almost every other programming language known, it does not use any indentation to visually aid the reader, leading to very confusing hierarchies.

### Why not YAML?
It's just so ugly, and filled with foot-guns. Go away.

## Performance
The Con language and library are designed for human-readable, human-sized config files.

There is nothing in the Con spec that would not make it as fast (or as slow, depending on your perspective) as JSON, but the library has not been optimized for performance (no crazy SIMD stuff etc).

If you plan on having huge files and performance is important to you, I suggest you use a binary format instead, like `MsgPack`.

## Con specification
TODO: fill in

### Sum types
Consider this Rust enum:

```rs
enum Color {
    Black,
    Gray(u8),
    Hsl(u8, u8, u8),
    Rgb { r: u8, g: u8, b: u8 },
}
```

JSON has no one standard for how to encode the different alternatives, and the different alternatives (external tagging, internal tagging, adjaceny tagging, …) all have their own shortcomings.

In Con, enum variants are written as `Variant(data)`.

So different values for the above choice would be written as:

* `Black` (equivalent to `Black()`)
* `Gray(128)`
* `Hsl(0, 100, 200)`
* `Rgb({r: 255, g: 0, b: 0})`

Note that all parenthesis (outside of quotes) MUST be proceeded by an identifier.

We could be tempted to allow `Rgb{r: 255, g: 0, b: 0}`, but that would be inconsistent. Worse, it would be ambiguous when parsing: is it one enum variant, or an enum variant followed by a map? (remember: commas are optional in Con).

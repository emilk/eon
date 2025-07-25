# Con - The nice config format
This repository contains the definition of _Con_, a new config format designed for human editing.

It also contains a Rust crate for reading, parsing, and formatting Con files.

## In short
```c
// comments
key: "value"
list: [1 2 3]
map: {
    key: "value"
}
special_floats: [+inf -inf +NaN]
```

Con is a strict superset of JSON.

The library is rather forgiving with what it accepts as input, but the output is strict.

Con is:
* Familiar: strongly resembles JSON
* Clean: forgoes unnecessary commas and quotes
* Simple: lists are enclosed in [], maps with {}
* Powerful: supports sum types, like Rust enums.

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

## Con specification
* Numbers: +123
* Simple enums: `option: Choice`

### Sum types
Consider this Rust example:

```rs
enum Choice {
    A,
    B(i32),
    C(i32),
    D(i32, i32),
    E{a: i32, b: i32}
}
```

JSON has no one standard for this, and all the popular alternatives have their own drawbacks.

In Con, enum variants are written as `Variant(data)`.

So different values for the above choice would be written as:

* `A` (equivalent to `A()`)
* `B(42)`
* `C(42)`
* `D(42, "hello")
* `E({a: 1, b: 2})`

Note that all paranthesis (outside of quotes) MUST be proceeded by an identifier.

We could be tempted to allow `E{a: 1, b: 2}`, but that would be inconsistent. Worse, it would be ambigious when parsing: is it one enum variant, or an enum variant followed by an object? `E, {a: 1, b: 2}` (remember: commas are optional in Con).

When decoding, we will forgivingly allow writing `"A"` instead of `A`.

That means we can still ommit commas

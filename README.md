# Con - The simple and friendly config format
This repository contains the definition of _Con_, a simple config format designed for human editing.

Con uses the `.con` file ending.

This repository also contains a Rust crate for using Con with `serde`, and a binary for formatting Con files.

## Overview
```yaml
// Comment
string: "Hello Con!"
list: [1 2 3]
object: {
    boolean: true
    key: 'Use single quotes if your string contains "quotes"'
}
map: {
    1: "map keys don't need to be strings"
    2: "they can be any Con value"
}
special_floats: [+inf, -inf, +NaN]
```

Con is designed to be
* **Familiar**: strongly resembles JSON
* **Clean**: forgoes unnecessary commas, quotes, and indentation
* **Clear**: lists are enclosed in `[ ]`, maps in `{ }`
* **Powerful**: supports sum types (e.g. Rust enums) and arbitrary map keys
* **Unambiguous**: no [Norway problem](https://www.bram.us/2022/01/11/yaml-the-norway-problem/)

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
### Supported types
Con files are always encoded as UTF-8.

A Con value is one of

#### `null`
A special `null` value.

#### Booleans
`true` or `false`.

#### Numbers
Numbers in Con can have an optional sign (`+` or `-`) followed by either:
- a decimal (`42`)
- a hexadecimal (`0xbeef`, case insensitive)
- a binary (`0b0101`)
- a float (`3.14`, `6.022e23` etc)
- `nan`: [IEEE 754 `NaN`](https://en.wikipedia.org/wiki/NaN)
- `inf`: infinity

#### Text
Strings are usually `"double-quoted"`, but `'single-quoted'` is allowed,
and useful for text that contain `"`.

- `"double quoted"`
- `'single quoted'`
- Escape special characters: `"\n \t \" \u{211D}"`

#### Lists
Lists are written as `[ … ]`, with _optional_ commas between values.
Usually the commas are omitted for lists that span multiple lines,
and included for lists that are on a single line.

```yaml
long_list: [
    2
    3
    5
    7
    11
]

short_list: [1, 2, 3]

also_fine: [
    1,
    2,
    3,
]

very_terse: [1 2 3]
```

A list can contain any Con value (including other lists).

#### Map
Maps are written as `{ key: value, … }`, again with optional commas between key-value pairs.
Usually the commas are omitted for maps that span multiple lines,
and included for maps that are on a single line.

Maps are used to represent either a record type (like a `struct`) or a table type (e.g. a hash map).

For instance, say you have a hash map for looking up country code based on the name of the country.
This can be serialized to Con as:

```C
country_codes: {
    "United States": 1
    "France": 3
    "United Kingdom": 4
    "Sweden": 4
    "Germany": 4
    "Australia": 6
    "Japan": 8
    "China": 8
    "India": 9
}
```

Both keys and values of a Con map can be any Con value.
This is significantly different from e.g. JSON, where keys can only be strings.
So, a reverse map (from code to country name) can we written as:


```yaml
country_from_code: {
    1:  "United States"
    33: "France"
    44: "United Kingdom"
    46: "Sweden"
    49: "Germany"
    61: "Australia"
    81: "Japan"
    86: "China"
    91: "India"
}
```

You can even use other maps as keys:

```yaml
complex_map: {
    {"foo": 42, "bar": "mushroom"}: "Yes, this is fine"
}
```

You are allowed to omit the quotes around map keys (NOT values!) when the keys are _identifiers_.

An identifier is defined in Con as any string matching the regex `[a-zA-Z_][a-zA-Z0-9_]*`.
This definition matches most programming languages (e.g. what is allowed as a variable name in C).

This means these two are equivalent:

```yaml
explicit: {
    "a": 1
    "b": 2
}
same: {
    a: 1
    b: 2
}
```

The convention is to use quotes for general strings (e.g. when representing a `HashMap<String, …>`
and to use identifiers when representing struct fields.

Examples of what is and isn't allowed:

```yaml
snake_case: true       // OK!
kebab-case: false      // ERROR! 'kebab-case' is not an identifier, and must be quoted
"kebab-case": true     // OK!
string: Hello          // ERROR! "Hello" should be in quotes
key: null              // OK! 'null' is a special value (it's NOT a string)
42: "forty-two"        // OK! Maps the integer 42 to a string
"42": "forty-two"      // OK! Maps the string "42" to a string (which is different from the above!)
true: "confusing"      // Confusing, but OK. Uses a boolean as key (not a string!).
"true": "fine"         // OK! Uses the string "true" as key
```


### Sum types (enum variants)
Let's first consider a simple `enum`, like one you would find in `C`:

```c
enum Side {
    Left,
    Middle,
    Right,
}
```

The variants are encoded as strings in Con, e.g.

```yaml
side: "Middle"
```

In other words, there is no difference between a string and an enum variant, as far as Con is concerned.

Now consider this more complicated Rust `enum`:

```rs
enum Color {
    Black,
    Gray(u8),
    Hsl(u8, u8, u8),
    Rgb { r: u8, g: u8, b: u8 },
}
```

Here a simple string will not sufficr, as some of the enum variants contain data.

There are several competing techniques of encoding this in JSON (external tagging, internal tagging, adjaceny tagging, …) all with their own shortcomings.

In Con, enum variants are written as `"Variant"(data)`.

So different values for the above choice would be written as:

* `"Black"` (equivalent to `"Black"()`)
* `"Gray"(128)`
* `"Hsl"(0, 100, 200)`
* `"Rgb"({r: 255, g: 0, b: 0})`

#### Digression: why this syntax for sum types?

Why the quotes, and not just `Black`, `Gray(128)`, etc?
Consider this hypothetical Con file:

```yaml
color: Black
name: Emil
```

Is `name` really a multiple-choice enum? Maybe. Maybe not.
But the Con parser doesn't know, so must accept it.
And this will leave the door open to weirdness where _some_ strings need quotes and not others (and we'll end up similar to YAML).
So instead we explicitly forbid the above, and always require quotes for strings (…except for map keys).

We could also tempted to also allow `"Rgb"{r: 255, g: 0, b: 0}`, but that would be ambiguous when parsing: is it one enum variant, or a string followed by a map? (remember: commas are optional in Con).

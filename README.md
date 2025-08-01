# Eon - The simple and friendly config format

[![Latest version](https://img.shields.io/crates/v/eon.svg)](https://crates.io/crates/eon)
[![Documentation](https://docs.rs/eon/badge.svg)](https://docs.rs/eon)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)
![Apache](https://img.shields.io/badge/license-Apache-blue.svg)

This repository contains the definition of _Eon_, a simple config format designed for human editing.

Eon uses the `.eon` file ending.

Eon is aimed to be a replacement for [Toml](https://toml.io/en/) and Yaml.

This repository also contains a Rust crate `eon` for using Eon with `serde`, and a `eonfmt` binary for formatting Eon files.


## Overview
```yaml
// Comment
string: "Hello Eon!"
list: [1, 2, 3]
map: {
    boolean: true
    literal_string: 'Can contain \ and "quotes"'
}
any_map: {
    42: "This key is an integer"
	[]: "This key is an empty list"
}
hex: 0xdead_beef
special_numbers: [+inf, -inf, +nan]

// Strings come in four flavors:
basic_strings: [
	"I'm a string."
	"I contain \"quotes\"."
	"Newline:\nUnicode: \u{262E} (☮)"
]
multiline_basic_strings: [
	"""\
		It was the best of strings.
		It was the worst of strings."""

	// The above is equivalent to:
	"It was the best of strings.\n\t\tIt was the worst of strings."
]
literal_strings: {
	// What you see is what you get:
	windows_path: 'C:\System32\foo.dll'
	quotes: 'I use "quotes" in this string'
	regex: '[+\-0-9\.][0-9a-zA-Z\.+\-_]*'
}
multiline_literal_strings: {
	python: '''
    # The first newline is ignored, but everything else is kept
    def main():
        print('Hello world!')
    '''
}
```

### Design goals
- **Familiar**: strongly resembles JSON
- **Clean**: forgoes unnecessary commas and quotes
- **Powerful**: supports arbitrary map keys and sum types (e.g. Rust enums)


### Main differences from JSON
* No need to wrap entire document in `{}`
* Quotes around map keys are optional for identifiers
* Map keys can be any value (not just string)
* Commas are optional
* Eon adds:
    * `// Comments`
    * Special floats: `+inf`, `-inf`, `+nan`
    * Named sum-type variants


## Formatter
You can format any Eon file using the `eonfmt` binary

```sh
cargo install --locked eonfmt
eonfmt *.eon
```


## Why another config format?
I wanted a format designed for human eyes with
- Indented hierarchy using `{ }` and `[ ]` (like JSON, C, Rust, …). Rules out YAML and TOML.
- No top-level `{ }` wrapping the whole file. Rules out JSON5, RON, and others.

### Why not [JSON5](https://json5.org/)?
JSON5 is _almost_ great, but requires wrapping the whole file in an extra `{ }` block, and indent that. That's too ugly for me.
It also has a bunch of unnecessary commas between values.

RON has the same problem.

### Why not [RON](https://github.com/ron-rs/ron)?
The goal of RON is to perfectly map the Rust datatypes, which is cool, but it means it leans more towards being verobse and complex while Eon wants to be lean and simple.

### Why not [Toml](https://toml.io/en/)?
Toml is a very nice and clean language.
However, unlike almost every other programming language known, it does not use any indentation to aid the reader, leading to very confusing hierarchies.
This means that when (visually) scanning a Toml document it's hard to figure out where one section starts and another ends.

It also means Toml is a bad candidate whenever you have deeply nested structures.

The `[[array.of.tables]]` syntax is also quite confusing to newcomers.

### Why not [YAML](https://yaml.org/)?
Yaml is clean, but over-complicated, inconsistent, and filled with foot guns. [It is known](https://ruudvanasseldonk.com/2023/01/11/the-yaml-document-from-hell).


## Performance
The Eon language (and library) are designed for config files that humans edit by hand.

There is nothing in the Eon spec that would not make it as fast (or as slow, depending on your perspective) as JSON, but the library has not been optimized for performance (no crazy SIMD stuff etc).
It still parses a chunky 1MB file in under 10ms on an M3 MacBook.

I would not recommend using Eon as a data transfer format. For that, use a binary format (like MsgPack or protobuffs), or JSON (which has optimized parser/formatters for every programming language).


## Roadmap
Future work, which I may or may not get to (contributions welcome!)

### Additional tools
- VSCode extension for
    - Syntax highlighting
    - Formatting

### Extending the spec
- Add special types?
    - ISO 8601
        - datetimes
        - local times
        - Durations? But ISO 8601 durations are so ugly
    - UUID?
- Hex floats?


## Eon specification
An Eon document is always encoded as UTF-8.

A document can be one of:
- A single value, e.g. `42` or `{foo: 1337, bar: 32}`
- The contents of a Map, e.g. `foo: 42, bar: 32` (this is syntactic sugar so you don't have to wrap the document in `{}`)
- The contents of a List, e.g. `32 46 12` (useful for a stream of values, e.g. like [ndjson](https://docs.mulesoft.com/dataweave/latest/dataweave-formats-ndjson))

Commas are optional in Eon, so `[1, 2, 3]` is the same as `[1  2  3]`.
By convention, commas are included when multiple values are on the same line, but omitted for multi-line maps and lists.

Whitespace is not significant (other than as a token separator).

By convention, we indent everything wrapped in `[]`, `{}`, `()`.

Comments are prefixed with `//`.

### Supported types
An Eon value is one of:

#### `null`
A special `null` value, signifying nothingness.

#### Boolean
`true` or `false`.

#### Number
Numbers in Eon can have an optional sign (`+` or `-`) followed by either:
- a decimal (`42`)
- a hexadecimal (`0xbeef`, case insensitive)
- a binary (`0b0101`)
- a decimal (`3.14`, `6.022e23` etc)

Numbers can use `_` anywhere in them as a visual separator, e.g. `123_456` or `0xdead_beef`.

There is not explicit bounds on the precision and magnitude of supported numbers in an Eon file,
but the `eon` crate is currently only support integers in the `[-2^127, 2^128)` range, and decimals are limited to the precision of `f64`.

Eon also support the special values:
- `+nan`: [IEEE 754 `NaN`](https://en.wikipedia.org/wiki/NaN)
- `+inf`: positive infinity
- `-inf`: negative infinity

Note that these special values MUST be prefixed with a sign (they are not keywords like `true/false/null` are).

#### Strings
Text in Eon comes in four flavors:
- `"basic string"`
- `"""multiline basic string"""`
- `'literal string'`
- `'''multiline literal string'''`

##### `"Basic strings"`
Basic strings uses double-quoted, and can contain escape sequences:

```py
"I'm a string."
"I contain \"quotes\"."
"Newline:\nUnicode: \u{262E} (☮)"
```

#### `"""Multi-line basic strings"""`
When you have long text it can be useful to have a string span multiple lines, using triple-quotes.

Escape sequences still work, and a line ending with `\` removes the newline and any whitespace at the start of the next line.

This means these two strings are equivalent:

```py
str1: """\
    It was the best of strings.
    It was the worst of strings."""

str2: "It was the best of strings.\n\tIt was the worst of strings."
```


#### `'Literal strings'`
Literal strings uses single quotes, and no escaping is performed. What you see is exactly what you get:

```py
windows_path: 'C:\System32\foo.dll'
quotes: 'I use "quotes" in this string'
regex: '[+\-0-9\.][0-9a-zA-Z\.+\-_]*'
```

There is no way to put a single quote inside of a literal strings, but you can in a…

#### `'''Multi-line literal strings'''`
A multi-line literal string uses three single quotes. Within that, everything is taken verbatim, except that the very first newline (if any) is ignored:

```py
quote_re: '''(["'])[^"']*$1'''
python: '''
# The first newline is ignored, but everything else is kept
def main():
    print('Hello world!')
'''
```


#### List
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

very_terse: [1, 2, 3]
```

A list can contain any Eon value (including other lists).

#### Map
Maps are written as `{ key: value, … }`, again with optional commas between key-value pairs.
Usually the commas are omitted for maps that span multiple lines,
and included for maps that are on a single line.

Maps are used to represent either a record type (like a `struct`) or a table type (e.g. a hash map).

For instance, say you have a hash map for looking up country code based on the name of the country.
This can be serialized to Eon as:

```C
country_codes: {
    "United States": 1
    "France": 33
    "United Kingdom": 44
    "Sweden": 46
    "Germany": 49
    "Australia": 61
    "Japan": 81
    "China": 86
    "India": 91
}
```

Both keys and values of an Eon map can be any Eon value.
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

An identifier is defined in Eon as any string matching the regex `[a-zA-Z_][a-zA-Z0-9_]*`.
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
true: "confusing"      // ⚠️ Confusing, but OK. Uses a boolean as key (not a string!).
"true": "fine"         // OK! Uses the string "true" as key
```

#### `null/true/false` as map keys
The last two lines in the above example show that you need to be careful when using key names that matches one of the three keywords (`true/false/null`). This is is a (small) footgun in Eon, but hopefully these key names are rare (they are already forbidden identifiers in many programming languages).

Alternative designs I've considered:
- Always require quotes for map keys (like JSON). Con: very ugly.
- Only allow strings as map keys. Con: can't express general maps (limiting).
- Only allow identifiers as map keys (like a Rust `struct`). Con: can't serialize a `HashMap<String, …>`.
- Change `null/true/false` to something else (e.g. `%null`, `%true`, `%false`). Would be both surprising and ugly.
- Forbid using unquoted `null/true/false` as map keys. Con: can't serialize `HashMap<bool, …>` or a general `HashMap<Value, …>`. Feels arbitrary.

### Named sum-type variants
Let's first consider a simple `enum`, like one you would find in C or Java:

```c
enum Side {
    Left,
    Middle,
    Right,
}
```

The variants are encoded as strings in Eon, e.g.

```yaml
side: "Middle"
```

In other words, there is no difference between a string and an enum variant, as far as Eon is concerned.

Now consider this more complicated Rust `enum`:

```rs
enum Color {
    Black,
    Gray(u8),
    Hsl(u8, u8, u8),
    Rgb { r: u8, g: u8, b: u8 },
}
```

Here a simple string will not suffice, as some of the enum variants contain data.

There are several competing techniques of encoding this in JSON (external tagging, internal tagging, adjaceny tagging, …) all with their own shortcomings.

In Eon, enum variants are written as `"Variant"(data)`.

So different values for the above choice would be written as:

- `"Black"` (equivalent to `"Black"()`)
- `"Gray"(128)`
- `"Hsl"(0, 100, 200)`
- `"Rgb"({r: 255, g: 0, b: 0})`

#### Digression: why this syntax for sum types?

Why the quotes, and not just `Black`, `Gray(128)`, etc?
Consider this hypothetical Eon file:

```yaml
color: Black
name: Emil
```

Is `name` really a multiple-choice enum? Maybe. Maybe not.
But the Eon parser wouldn't know, so must accept it.
And this will leave the door open to weirdness where _some_ strings need quotes and not others (and we'll end up similar to YAML).
So instead we explicitly forbid the above, and always require quotes for strings (…except for map keys that are identifiers).

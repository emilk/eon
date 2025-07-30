
## TODO
* [x] Parser
* [x] Serde
* [x] general maps (keys of any type)
* [x] formatter binary
* [x] Figure out how the difference between a string and a zero-value variant should be encoded in `Value`
* [x] Protect against stack overflow in recursive-decent parser
* [ ] Generate comparison with https://docs.rs/ron/
* [ ] Test against evil JSON files to make sure Eon is robust
* [ ] Remove all TODOs
* [ ] Numbers
    * [x] Allow `_` in numbers
    * [x] Test hexal and binary numbers
    * [x] What strings should we use for infinities
    * [ ] Test perfect round-tripping of floats
    * [ ] Hex floats?
* [ ] Strings
    * [ ] Raw strings, for e.g. regex
    * [ ] Handle JSON-style unicode embeddings??
    * [ ] Allow escaped in single-quoted strings?
* [ ] Write a spec
    * [ ] newline-separated as part of spec (parse multiple values in same file)
* [ ] Publish crates
* [ ] Warn about unused keys (i.e. mistyped keys that was never accessed during deserialization)

## Additional tools
* [ ] VSCode extension for
    * [ ] Syntax highlighting
    * [ ] Formatting

## Extending the spec
* [ ] Add special types?
    * [ ] ISO 8601 datetimes?
    * [ ] ISO 8601 durations?
    * [ ] UUID?

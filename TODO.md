
## TODO
* [x] Parser
* [x] Serde
* [x] general maps (keys of any type)
* [x] formatter binary
* [x] Figure out how the difference between a string and a zero-value variant should be encoded in `Value`
* [x] Protect against stack overflow in recursive-decent parser
* [x] Strings
* [x] Make sure CI workss
* [x] Produce error if the same key is repeated
* [x] Handle Windows newlines (strip all `\r` from input).
* [x] Remove all naked TODOs
* [x] Numbers
    * [x] Allow `_` in numbers
    * [x] Test hexal and binary numbers
    * [x] What strings should we use for infinities
    * [x] Test perfect round-tripping of floats
* [x] Create an `example.eon` file
* [ ] Generate comparison with https://docs.rs/ron/
* [ ] Publish crates
* [ ] Warn about unused keys (i.e. mistyped keys that was never accessed during deserialization)

## Additional tools
* [ ] VSCode extension for
    * [ ] Syntax highlighting
    * [ ] Formatting

## Extending the spec
* [ ] Add special types?
    * [ ] ISO 8601
        * [ ] datetimes
        * [ ] local times
        * [ ] Durations? But ISO 8601 durations are so ugly
    * [ ] UUID?
* [ ] Hex floats?

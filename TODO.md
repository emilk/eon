
## TODO
* [x] Parser
* [x] Serde
* [x] general maps (keys of any type)
* [x] formatter binary
* [x] Figure out how the difference between a string and a zero-value variant should be encoded in `Value`
* [ ] Remove all TODOs
* [ ] Numbers
    * [x] Allow `_` in numbers
    * [x] Test hexal and binary numbers
    * [ ] Test perfect round-tripping of floats
    * [ ] Hex floats?
* [ ] Write a spec
    * [ ] newline-separated as part of spec (parse multiple values in same file)
* [ ] Generate comparison with https://docs.rs/ron/
* [ ] What strings should we use for infinities?
    * [ ] use lowercase `nan` to be more similar to toml
    * [ ] allow `inf` and `nan` without a sign?
* [ ] Protect against stack overflow in recursive-decent parser
* [ ] Test against evil JSON files to make sure Con is robust
* [ ] Publish crates
* [ ] Warn about unused keys (i.e. mistyped keys that was never accessed during deserialization)
* [ ] Special types?
    * [ ] UUID?
    * [ ] Date-time?


## TODO
* [x] Parser
* [x] Serde
* [x] general maps (keys of any type)
* [x] formatter binary
* [ ] Remove all TODOs
* [ ] Hexal and binary numbers
* [ ] Write a spec
* [ ] Protect against stack overflow in recursive-decent parser
* [ ] Test against evil JSON files to make sure Con is robust
* [ ] Publish crates
* [ ] newline-separated as part of spec (parse multiple values in same file)
* [ ] Generate comparison with https://docs.rs/ron/latest/ron/
* [ ] What strings should we use for infinities?
* [ ] Warn about unused keys (i.e. mistyped keys that was never accessed during deserialization)
* [ ] Allow `_` in numbers
* [ ] use lowercase `nan` to be more similar to toml
* [ ] allow `inf` and `nan` without a sign?
* [ ] Special types?
    * [ ] UUID?
    * [ ] Date-time?

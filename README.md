# kle-serial

A Rust library for deserialising [Keyboard Layout Editor] files.
Designed to be used in conjunction with [`serde_json`] to deserialize JSON files exported from KLE.

## Example

![example]

```rust
use kle_serial::Keyboard;

let keyboard: Keyboard = serde_json::from_str(
    r#"[
        {"name": "example"},
        [{"f": 4}, "!\n1\n¹\n¡"]
    ]"#
).unwrap();

assert_eq!(keyboard.metadata.name, "example");
assert_eq!(keyboard.keys.len(), 1);

assert!(keyboard.keys[0].legends[0].is_some());
let legend = keyboard.keys[0].legends[0].as_ref().unwrap();

assert_eq!(legend.text, "!");
assert_eq!(legend.size, 4);

assert!(keyboard.keys[0].legends[1].is_none());
```

[Keyboard Layout Editor]: http://www.keyboard-layout-editor.com/
[`serde_json`]: https://crates.io/crates/serde_json
[example]: doc/example.png

## Licence

Licensed under either of

* Apache License, Version 2.0 ([LICENCE-APACHE](LICENCE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
* MIT license ([LICENCE-MIT](LICENCE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
this crate by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without
any additional terms or conditions.

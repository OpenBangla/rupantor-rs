# rupantor-rs
[![Build Status](https://travis-ci.org/OpenBangla/rupantor-rs.svg?branch=master)](https://travis-ci.org/OpenBangla/rupantor-rs)
[![Rust](https://img.shields.io/badge/rust-1.31.0%2B-blue.svg?maxAge=3600)](https://github.com/OpenBangla/rupantor-rs)
[![crates.io](https://img.shields.io/crates/v/rupantor.svg)](https://crates.io/crates/rupantor)
[![DOCS.rs](https://docs.rs/rupantor/badge.svg)](https://docs.rs/rupantor)

A Bengali Phonetic Parser which is very flexible and converts text into Bengali according to a json formatted grammar.

`rupantor` supports **Avro Phonetic** out of the box, which is a very popular phonetic based transliteration method for writing Bengali.
`rupantor` is very flexible as it is possible to control the conversion by specifying the *grammar*/*conversion rules*. So it is possible
by the user to create a phonetic method by specifying a custom grammar.

## Usage
Add this to your `Cargo.toml`:
```toml
[dependencies]
rupantor = "0.2"
```

## Example
This example shows how to use Avro Phonetic:
```rust
use rupantor::avro::AvroPhonetic;

let avro = AvroPhonetic::new();
let bengali = avro.convert("ami banglay gan gai");
assert_eq!(bengali, "আমি বাংলায় গান গাই");
```

## License
`rupantor` is distributed under the terms of MPL License (Version 2.0).

See [LICENSE](https://github.com/OpenBangla/rupantor-rs/blob/master/LICENSE) for details.
//! **Rupantor** is a very flexible Bengali phonetic parser
//! written in Rust.
//! 
//! `rupantor` converts text into Bengali text by obeying the _rules_
//! given in a _grammar_. A grammar is actually a Json formated data.
//! 
//! **Avro Phonetic** is a very popular phonetic based transliteration method 
//! for writing Bengali. `rupantor` supports Avro Phonetic writing method out of the box.
//! User can use Avro Phonetic by using the [`AvroPhonetic`](avro/struct.AvroPhonetic.html).
//! 
//! # Example: Using Avro Phonetic to convert text into Bengali
//! ```rust
//! extern crate rupantor;
//! use rupantor::avro::AvroPhonetic;
//! 
//! let avro = AvroPhonetic::new();
//! let bengali = avro.convert("ami banglay gan gai");
//! assert_eq!(bengali, "আমি বাংলায় গান গাই");
//! ```
//! 
//! `rupantor` is very flexible as the conversion is driven by a __grammar__ file. Actually
//! the [`AvroPhonetic`](avro/struct.AvroPhonetic.html) struct uses [`PhoneticParser`](parser/struct.PhoneticParser.html)
//! struct and a Avro Phonetic [grammar file](https://github.com/OpenBangla/rupantor-rs/blob/master/src/AvroPhonetic.json)
//! internally to do the conversion.
//! 
//! The phonetic conversion algorithm was actually implemented by
//! [Rifat Nabi](https://github.com/torifat) in [JavaScript](https://github.com/torifat/jsAvroPhonetic)
//! and [ObjectiveC](https://github.com/torifat/iAvro/blob/master/AvroParser.m).
//! This crate is the Rust port of that phonetic conversion algorithm.
extern crate serde_json;
extern crate stringplus;

pub mod parser;
pub mod avro;
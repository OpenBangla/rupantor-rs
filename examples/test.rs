extern crate bengali_phonetic_parser;
extern crate json;
use bengali_phonetic_parser::PhoneticParser;

fn main() {
    // Parse the rule file
    let js = json::parse(include_str!("../src/AvroPhonetic.json")).unwrap();

    let cvt = PhoneticParser::new(&js);
    println!("{}", cvt.convert("ami banglay gan gai".to_owned()));
}
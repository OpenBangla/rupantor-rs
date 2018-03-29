extern crate bengali_phonetic_parser;
extern crate json;
use std::env;
use std::fs::File;
use std::io::Read;
use bengali_phonetic_parser::PhoneticParser;

fn main() {
    // Get the rule file
    let mut p = env::current_dir().unwrap();
    p.push("AvroPhonetic.json");
    let path = p.to_str().unwrap();
        
    let mut grammer = String::new();

    let _ = File::open(path).unwrap().read_to_string(&mut grammer);
    let js = json::parse(&grammer).unwrap();

    let cvt = PhoneticParser::new(&js);
    println!("{}", cvt.convert("ami".to_owned()));
/*
    let _find = &js["patterns"];
    println!("{}", _find.len());*/
}
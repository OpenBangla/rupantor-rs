use rupantor::parser::PhoneticParser;

fn main() {
    // Parse the rule file
    let js = serde_json::from_str(include_str!("../src/AvroPhonetic.json")).unwrap();

    let cvt = PhoneticParser::new(&js);
    println!("{}", cvt.convert("ami banglay gan gai"));
}
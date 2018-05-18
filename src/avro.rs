use json;
use parser::PhoneticParser;

pub struct AvroPhonetic {
    parser: PhoneticParser,
}

impl AvroPhonetic {
    pub fn new() -> AvroPhonetic {
        let rule = json::parse(include_str!("AvroPhonetic.json")).unwrap();
        AvroPhonetic { parser: PhoneticParser::new(&rule) }
    }

    pub fn convert(&self, input: &str) -> String {
        self.parser.convert(input)
    }
}

#[cfg(test)]
mod tests {
    use avro::AvroPhonetic;

    #[test]
    fn test_avro() {
        let parser = AvroPhonetic::new();
        assert_eq!(parser.convert("amader valObasa hoye gel ghas, kheye gel goru ar diye gelo ba^sh"), "আমাদের ভালোবাসা হয়ে গেল ঘাস, খেয়ে গেল গরু আর দিয়ে গেল বাঁশ");
    }
}
#![allow(dead_code)]
extern crate json;

pub struct PhoneticParser<'a> {
    patterns: &'a json::JsonValue,
    vowel: String,
    consonant: String,
    numbers: String,
    case_sensitive: String,
}

impl<'a> PhoneticParser<'a> {
    fn new(rule: &json::JsonValue) -> PhoneticParser {
        PhoneticParser {
            patterns: rule,
            vowel: rule["vowel"].as_str().unwrap().to_string(),
            consonant: rule["consonant"].as_str().unwrap().to_string(),
            numbers: rule["number"].as_str().unwrap().to_string(),
            case_sensitive: rule["casesensitive"].as_str().unwrap().to_string()
        }
    }
    
    fn fix_string(&self, string: String) -> String {
        let mut fixed = String::new();
        for character in string.chars() {
            if self.is_case_sensitive(character) {
                fixed.push(character);
            } else {
                fixed.push_str(&character.to_lowercase().to_string());
            }
        }
        fixed
    }

    fn is_vowel(&self, string: char) -> bool {
        self.vowel.contains(&string.to_lowercase().to_string())
    }

    fn is_consonant(&self, string: char) -> bool {
        self.consonant.contains(&string.to_lowercase().to_string())
    }

    fn is_case_sensitive(&self, character: char) -> bool {
        self.case_sensitive.contains(&character.to_lowercase().to_string())
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs::File;
    use std::io::Read;
    use json;
    use super::PhoneticParser;

    fn get_rule() -> json::JsonValue {
        // Get the rule file
        let mut p = env::current_dir().unwrap();
        p.push("AvroPhonetic.json");
        let path = p.to_str().unwrap();
        
        let mut grammer = String::new();

        let _ = File::open(path).unwrap().read_to_string(&mut grammer);

        json::parse(&grammer).unwrap()
    }
    
    #[test]
    fn test_helpers() {
        let json = get_rule();
        let parser = PhoneticParser::new(&json);

        assert!(parser.is_vowel('A'));
        assert_eq!(parser.is_vowel('b'), false);
        assert!(parser.is_consonant('B'));
        assert_eq!(parser.is_consonant('e'), false);
        assert_eq!(parser.fix_string("ODEr AMAr".to_string()), "ODer amar");
    }
}

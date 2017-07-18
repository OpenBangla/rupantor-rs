extern crate json;

pub struct PhoneticParser {
    patterns: json::JsonValue,
    vowel: String,
    consonant: String,
    numbers: String,
    case_sensitive: String,
}

impl PhoneticParser {
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
        self.vowel.contains(string)
    }

    fn is_consonant(&self, string: char) -> bool {
        self.consonant.contains(string)
    }

    fn is_case_sensitive(&self, character: char) -> bool {
        self.case_sensitive.contains(&character.to_lowercase().to_string())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}

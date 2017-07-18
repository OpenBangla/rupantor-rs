//extern crate json;

pub struct PhoneticParser {
    //patterns: json::Object,
    vowel: String,
    consonant: String,
    numbers: String,
    case_sensitive: String,
}

impl PhoneticParser {
    fn is_vowel(&self, string: &str) -> bool {
        self.vowel.contains(string)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}

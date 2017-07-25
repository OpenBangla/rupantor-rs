#![allow(dead_code)]
extern crate json;

use std::cmp::Ordering;

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

    fn convert(&self, input: String) -> String {
        let fixed = self.fix_string(input);
        let mut output = String::new();

        let _find = &self.patterns[0]["find"];
        let max_pattern_len = _find.len();


        let len = fixed.len();

        for mut cur in 0..len {
            let start = cur;
            let mut end = 0;
            let mut matched = false;

            for chunk_len in (1..max_pattern_len+1).rev() {
                end = start + chunk_len;
                if end <= len {
                    let chunk = &fixed[start..chunk_len+1];
                    
                    // Binary Search
                    let mut left = 0; 
                    let mut right = self.patterns.len() - 1;
                    let mut mid = 0;
                    while right >= left {
                        mid = (right + left) / 2;
                        let pattern = &self.patterns[mid];
                        let find = &pattern["find"];
                        if find == chunk {
                            let rules = &pattern["rules"];
                            if !rules.is_empty() {
                                for rule in rules.members() {
                                    let mut replace = true;
                                    let mut chk = 0;
                                    let matches = &rule["matches"];
                                    for _match in matches.members() {
                                        let value = _match["value"].as_str().unwrap();
                                        let _type = &_match["type"];
                                        let mut scope = _match["scope"].as_str().unwrap();
                                        let mut is_negative = false;

                                        // Handle Negative
                                        if &scope[0..1] == "!" {
                                            is_negative = true;
                                            scope = &scope[1..];
                                        }

                                        if _find == "suffix" {
                                            chk = end;
                                        } else {
                                            chk = start - 1;
                                        }
                                        
                                        // Beginning
                                        if scope == "punctuation" {
                                            if
                                              ! (
                                              (chk < 0 && (_type == "prefix")) ||
                                              (chk >= len && (_type == "suffix")) ||
                                               self.is_punctuation(&fixed[chk..chk+1])
                                              ) ^ is_negative
                                            {
                                                replace = false;
                                                break;
                                            }
                                        } else if scope == "vowel" {
                                            if
                                              !(
                                               (
                                               (chk >= 0 && (_type == "prefix")) ||
                                               (chk < len && (_type == "suffix"))
                                               ) &&
                                               self.is_vowel(&fixed[chk..chk+1])
                                               ) ^ is_negative
                                            {
                                                replace = false;
                                                break;
                                            }
                                        } else if scope == "number" {
                                            if
                                              !(
                                               (
                                               (chk >= 0 && (_type == "prefix")) ||
                                               (chk < len && (_type == "suffix"))
                                               ) &&
                                               self.is_number(&fixed[chk..chk+1])
                                               ) ^ is_negative
                                            {
                                                replace = false;
                                                break;
                                            }
                                        } else if scope == "exact" {
                                            let mut s = 0;
                                            let mut e = 0;
                                            if _type == "suffix" {
                                                s = end;
                                                e = end + value.len();
                                            } else {
                                                // Prefix
                                                s = start - value.len();
                                                e = start;
                                            }
                                            if !self.is_exact(value, &fixed, s as u32, e as u32, is_negative) {
                                                replace = false;
                                                break;
                                            }
                                        }
                                    }

                                    if replace {
                                        let rl = rule["replace"].as_str().unwrap();
                                        output += rl;
                                        cur = end - 1;
                                        matched = true;
                                        break;
                                    }
                                }
                            }

                            if matched { break; }

                            // Default
                            let rl = pattern["replace"].as_str().unwrap();
                            output += rl;
                            cur = end - 1;
                            matched = true;
                            break;
                        } else if find.len() > chunk.len() ||
                                  (find.len() == chunk.len() && find.as_str().unwrap().to_string().cmp(&chunk.to_string()) == Ordering::Less) {
                            left = mid + 1;
                        } else {
                            right = mid - 1;
                        }
                    }
                    if matched { break; }
                }
            }

            if !matched {
                output += &fixed[cur..cur+1];
            }
        }

        output
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

    fn is_vowel(&self, string: &str) -> bool {
        self.vowel.contains(&string.to_lowercase())
    }

    fn is_consonant(&self, string: &str) -> bool {
        self.consonant.contains(&string.to_lowercase())
    }

    fn is_case_sensitive(&self, character: char) -> bool {
        self.case_sensitive.contains(&character.to_lowercase().to_string())
    }

    fn is_number(&self, character: &str) -> bool {
        self.numbers.contains(character)
    }

    fn is_exact(&self, needle: &str, heystack: &str, start: u32, end: u32, not: bool) -> bool {
        let len = end - start;
        //return ((start >= 0 && end < heystack.length() && (heystack.mid(start, len) == needle)) ^ strnot);
        ((start >= 0 && end < heystack.len() as u32 && (&heystack[(start as usize)..(len as usize + 1)] == needle)) ^ not)
    }

    fn is_punctuation(&self, character: &str) -> bool {
        !(self.is_vowel(character) || self.is_consonant(character))
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

        assert!(parser.is_vowel("A"));
        assert_eq!(parser.is_vowel("b"), false);
        assert!(parser.is_consonant("B"));
        assert_eq!(parser.is_consonant("e"), false);
        assert_eq!(parser.fix_string("ODEr AMAr".to_string()), "ODer amar");
        assert!(parser.is_number("1"));
    }
}

#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_assignments)]
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
    pub fn new(rule: &json::JsonValue) -> PhoneticParser {
        PhoneticParser {
            patterns: &rule["patterns"],
            vowel: rule["vowel"].as_str().unwrap().to_string(),
            consonant: rule["consonant"].as_str().unwrap().to_string(),
            numbers: rule["number"].as_str().unwrap().to_string(),
            case_sensitive: rule["casesensitive"].as_str().unwrap().to_string(),
        }
    }

    pub fn convert(&self, input: String) -> String {
        let fixed = self.fix_string(input);
        let mut output = String::new();

        let _find = self.patterns[0]["find"].as_str().unwrap();
        let max_pattern_len = _find.len();

        let len = fixed.len();
        let mut cur = 0;
        while cur < len {
            let start = cur as i32;
            let mut end: i32 = 0;
            let mut matched = false;

            for chunk_len in (1..=max_pattern_len).rev() {
                end = start + chunk_len as i32;
                if end <= len as i32 {
                    let chunk = fixed.substring(start as usize, chunk_len as usize);

                    // Binary Search
                    let mut left: i32 = 0;
                    let mut right = self.patterns.len() as i32 - 1;
                    let mut mid: i32 = 0;
                    while right >= left {
                        mid = (right + left) / 2;
                        let pattern = &self.patterns[mid as usize];
                        let find = pattern["find"].as_str().unwrap();
                        if find == chunk {
                            let rules = &pattern["rules"];
                            if !rules.is_empty() {
                                for rule in rules.members() {
                                    let mut replace = true;
                                    let mut chk = 0;
                                    let matches = &rule["matches"];
                                    for _match in matches.members() {
                                        let value = _match["value"].as_str().unwrap_or("");
                                        let _type = _match["type"].as_str().unwrap();
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
                                        match scope {
                                            "punctuation" => if ((chk < 0 && (_type == "prefix"))
                                                || (chk >= len as i32 && (_type == "suffix"))
                                                || self.is_punctuation(fixed.at(chk as usize)))
                                                == is_negative
                                            {
                                                replace = false;
                                                break;
                                            },
                                            "vowel" => if (((chk >= 0 && (_type == "prefix"))
                                                || (chk < len as i32 && (_type == "suffix")))
                                                && self.is_vowel(fixed.at(chk as usize)))
                                                == is_negative
                                            {
                                                replace = false;
                                                break;
                                            },

                                            "consonant" => if (((chk >= 0 && (_type == "prefix"))
                                                || (chk < len as i32 && (_type == "suffix")))
                                                && self.is_consonant(fixed.at(chk as usize)))
                                                == is_negative
                                            {
                                                replace = false;
                                                break;
                                            },

                                            "number" => if (((chk >= 0 && (_type == "prefix"))
                                                || (chk < len as i32 && (_type == "suffix")))
                                                && self.is_number(fixed.at(chk as usize)))
                                                == is_negative
                                            {
                                                replace = false;
                                                break;
                                            },

                                            "exact" => {
                                                let mut s: i32 = 0;
                                                let mut e: i32 = 0;
                                                if _type == "suffix" {
                                                    s = end;
                                                    e = end + value.len() as i32;
                                                } else {
                                                    // Prefix
                                                    s = start - value.len() as i32;
                                                    e = start;
                                                }
                                                if !self.is_exact(value, &fixed, s, e, is_negative)
                                                {
                                                    replace = false;
                                                    break;
                                                }
                                            }
                                            _ => panic!("Unknown scope"),
                                        };
                                    }

                                    if replace {
                                        output += rule["replace"].as_str().unwrap();
                                        cur = (end - 1) as usize;
                                        matched = true;
                                        break;
                                    }
                                }
                            }

                            if matched {
                                break;
                            }

                            // Default
                            output += pattern["replace"].as_str().unwrap();
                            cur = (end - 1) as usize;
                            matched = true;
                            break;
                        } else if find.len() > chunk.len()
                            || (find.len() == chunk.len() && find.cmp(&chunk) == Ordering::Less)
                        {
                            left = mid + 1;
                        } else {
                            right = mid - 1;
                        }
                    }
                    if matched {
                        break;
                    }
                }
            }

            if !matched {
                output += &fixed[cur..cur + 1];
            }
            cur += 1;
        }

        output
    }

    fn fix_string(&self, string: String) -> String {
        string
            .chars()
            .map(|character| {
                if self.is_case_sensitive(character) {
                    character
                } else {
                    character.to_lowercase().next().unwrap()
                }
            })
            .collect()
    }

    fn is_vowel(&self, string: &str) -> bool {
        self.vowel.contains(&string.to_lowercase())
    }

    fn is_consonant(&self, string: &str) -> bool {
        self.consonant.contains(&string.to_lowercase())
    }

    fn is_case_sensitive(&self, character: char) -> bool {
        self.case_sensitive
            .contains(&character.to_lowercase().to_string())
    }

    fn is_number(&self, character: &str) -> bool {
        self.numbers.contains(character)
    }

    fn is_exact(&self, needle: &str, heystack: &str, start: i32, end: i32, not: bool) -> bool {
        let len = end - start;
        (start >= 0 && end < heystack.len() as i32
            && (heystack.substring(start as usize, len as usize) == needle)) != not
    }

    fn is_punctuation(&self, character: &str) -> bool {
        !(self.is_vowel(character) || self.is_consonant(character))
    }
}

trait Substring {
    fn substring(&self, start: usize, length: usize) -> &str;
    fn at(&self, pos: usize) -> &str;
}

impl Substring for std::string::String {
    fn substring(&self, start: usize, length: usize) -> &str {
        &self[start..(start + length)]
    }

    fn at(&self, pos: usize) -> &str {
        &self[pos..pos + 1]
    }
}

impl Substring for str {
    fn substring(&self, start: usize, length: usize) -> &str {
        &self[start..(start + length)]
    }

    fn at(&self, pos: usize) -> &str {
        &self[pos..pos + 1]
    }
}

#[cfg(test)]
mod tests {
    use json;
    use super::PhoneticParser;

    #[test]
    fn test_helpers() {
        let json = json::parse(include_str!("AvroPhonetic.json")).unwrap();
        let parser = PhoneticParser::new(&json);

        assert!(parser.is_vowel("A"));
        assert_eq!(parser.is_vowel("b"), false);
        assert!(parser.is_consonant("B"));
        assert_eq!(parser.is_consonant("e"), false);
        assert_eq!(parser.fix_string("ODEr AMAr".to_string()), "ODer amar");
        assert!(parser.is_number("1"));
    }
}

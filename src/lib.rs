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
    max_pattern_len: usize,
}

impl<'a> PhoneticParser<'a> {
    pub fn new(rule: &json::JsonValue) -> PhoneticParser {
        PhoneticParser {
            patterns: &rule["patterns"],
            vowel: rule["vowel"].as_str().unwrap().to_string(),
            consonant: rule["consonant"].as_str().unwrap().to_string(),
            numbers: rule["number"].as_str().unwrap().to_string(),
            case_sensitive: rule["casesensitive"].as_str().unwrap().to_string(),
            max_pattern_len: rule["patterns"][0]["find"].as_str().unwrap().len(),
        }
    }

    pub fn convert(&self, input: String) -> String {
        let fixed = self.fix_string(input);
        let mut output = String::new();

        let len = fixed.len();
        let mut cur = 0;
        while cur < len {
            let start = cur as i32;
            let mut end: i32 = 0;
            let mut matched = false;

            for chunk_len in (1..=self.max_pattern_len).rev() {
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

                                        if _type == "suffix" {
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

    #[test]
    fn test_basic() {
        let json = json::parse(include_str!("AvroPhonetic.json")).unwrap();
        let parser = PhoneticParser::new(&json);

        assert_eq!(parser.convert("bhl".to_string()), "ভ্ল");
        assert_eq!(parser.convert("bj".to_string()), "ব্জ");
        assert_eq!(parser.convert("bd".to_string()), "ব্দ");
        assert_eq!(parser.convert("bb".to_string()), "ব্ব");
        assert_eq!(parser.convert("bl".to_string()), "ব্ল");
        assert_eq!(parser.convert("bh".to_string()), "ভ");
        assert_eq!(parser.convert("vl".to_string()), "ভ্ল");
        assert_eq!(parser.convert("b".to_string()), "ব");
        assert_eq!(parser.convert("v".to_string()), "ভ");
        assert_eq!(parser.convert("cNG".to_string()), "চ্ঞ");
        assert_eq!(parser.convert("cch".to_string()), "চ্ছ");
        assert_eq!(parser.convert("cc".to_string()), "চ্চ");
        assert_eq!(parser.convert("ch".to_string()), "ছ");
        assert_eq!(parser.convert("c".to_string()), "চ");
        assert_eq!(parser.convert("dhn".to_string()), "ধ্ন");
        assert_eq!(parser.convert("dhm".to_string()), "ধ্ম");
        assert_eq!(parser.convert("dgh".to_string()), "দ্ঘ");
        assert_eq!(parser.convert("ddh".to_string()), "দ্ধ");
        assert_eq!(parser.convert("dbh".to_string()), "দ্ভ");
        assert_eq!(parser.convert("dv".to_string()), "দ্ভ");
        assert_eq!(parser.convert("dm".to_string()), "দ্ম");
        assert_eq!(parser.convert("DD".to_string()), "ড্ড");
        assert_eq!(parser.convert("Dh".to_string()), "ঢ");
        assert_eq!(parser.convert("dh".to_string()), "ধ");
        assert_eq!(parser.convert("dg".to_string()), "দ্গ");
        assert_eq!(parser.convert("dd".to_string()), "দ্দ");
        assert_eq!(parser.convert("D".to_string()), "ড");
        assert_eq!(parser.convert("d".to_string()), "দ");
        assert_eq!(parser.convert("...".to_string()), "...");
        assert_eq!(parser.convert(".`".to_string()), ".");
        assert_eq!(parser.convert("..".to_string()), "।।");
        assert_eq!(parser.convert(".".to_string()), "।");
        assert_eq!(parser.convert("ghn".to_string()), "ঘ্ন");
        assert_eq!(parser.convert("Ghn".to_string()), "ঘ্ন");
        assert_eq!(parser.convert("gdh".to_string()), "গ্ধ");
        assert_eq!(parser.convert("gN".to_string()), "গ্ণ");
        assert_eq!(parser.convert("GN".to_string()), "গ্ণ");
        assert_eq!(parser.convert("gn".to_string()), "গ্ন");
        assert_eq!(parser.convert("gm".to_string()), "গ্ম");
        assert_eq!(parser.convert("Gm".to_string()), "গ্ম");
        assert_eq!(parser.convert("gl".to_string()), "গ্ল");
        assert_eq!(parser.convert("Gl".to_string()), "গ্ল");
        assert_eq!(parser.convert("gg".to_string()), "জ্ঞ");
        assert_eq!(parser.convert("GG".to_string()), "জ্ঞ");
        assert_eq!(parser.convert("Gg".to_string()), "জ্ঞ");
        assert_eq!(parser.convert("gG".to_string()), "জ্ঞ");
        assert_eq!(parser.convert("gh".to_string()), "ঘ");
        assert_eq!(parser.convert("Gh".to_string()), "ঘ");
        assert_eq!(parser.convert("g".to_string()), "গ");
        assert_eq!(parser.convert("hN".to_string()), "হ্ণ");
        assert_eq!(parser.convert("hn".to_string()), "হ্ন");
        assert_eq!(parser.convert("hm".to_string()), "হ্ম");
        assert_eq!(parser.convert("hl".to_string()), "হ্ল");
        assert_eq!(parser.convert("h".to_string()), "হ");
        assert_eq!(parser.convert("jjh".to_string()), "জ্ঝ");
        assert_eq!(parser.convert("jNG".to_string()), "জ্ঞ");
        assert_eq!(parser.convert("jh".to_string()), "ঝ");
        assert_eq!(parser.convert("jj".to_string()), "জ্জ");
        assert_eq!(parser.convert("j".to_string()), "জ");
        assert_eq!(parser.convert("J".to_string()), "জ");
        assert_eq!(parser.convert("kkhN".to_string()), "ক্ষ্ণ");
        assert_eq!(parser.convert("kShN".to_string()), "ক্ষ্ণ");
        assert_eq!(parser.convert("kkhm".to_string()), "ক্ষ্ম");
        assert_eq!(parser.convert("kShm".to_string()), "ক্ষ্ম");
        assert_eq!(parser.convert("kxN".to_string()), "ক্ষ্ণ");
        assert_eq!(parser.convert("kxm".to_string()), "ক্ষ্ম");
        assert_eq!(parser.convert("kkh".to_string()), "ক্ষ");
        assert_eq!(parser.convert("kSh".to_string()), "ক্ষ");
        assert_eq!(parser.convert("ksh".to_string()), "কশ");
        assert_eq!(parser.convert("kx".to_string()), "ক্ষ");
        assert_eq!(parser.convert("kk".to_string()), "ক্ক");
        assert_eq!(parser.convert("kT".to_string()), "ক্ট");
        assert_eq!(parser.convert("kt".to_string()), "ক্ত");
        assert_eq!(parser.convert("kl".to_string()), "ক্ল");
        assert_eq!(parser.convert("ks".to_string()), "ক্স");
        assert_eq!(parser.convert("kh".to_string()), "খ");
        assert_eq!(parser.convert("k".to_string()), "ক");
        assert_eq!(parser.convert("lbh".to_string()), "ল্ভ");
        assert_eq!(parser.convert("ldh".to_string()), "ল্ধ");
        assert_eq!(parser.convert("lkh".to_string()), "লখ");
        assert_eq!(parser.convert("lgh".to_string()), "লঘ");
        assert_eq!(parser.convert("lph".to_string()), "লফ");
        assert_eq!(parser.convert("lk".to_string()), "ল্ক");
        assert_eq!(parser.convert("lg".to_string()), "ল্গ");
        assert_eq!(parser.convert("lT".to_string()), "ল্ট");
        assert_eq!(parser.convert("lD".to_string()), "ল্ড");
        assert_eq!(parser.convert("lp".to_string()), "ল্প");
        assert_eq!(parser.convert("lv".to_string()), "ল্ভ");
        assert_eq!(parser.convert("lm".to_string()), "ল্ম");
        assert_eq!(parser.convert("ll".to_string()), "ল্ল");
        assert_eq!(parser.convert("lb".to_string()), "ল্ব");
        assert_eq!(parser.convert("l".to_string()), "ল");
        assert_eq!(parser.convert("mth".to_string()), "ম্থ");
        assert_eq!(parser.convert("mph".to_string()), "ম্ফ");
        assert_eq!(parser.convert("mbh".to_string()), "ম্ভ");
        assert_eq!(parser.convert("mpl".to_string()), "মপ্ল");
        assert_eq!(parser.convert("mn".to_string()), "ম্ন");
        assert_eq!(parser.convert("mp".to_string()), "ম্প");
        assert_eq!(parser.convert("mv".to_string()), "ম্ভ");
        assert_eq!(parser.convert("mm".to_string()), "ম্ম");
        assert_eq!(parser.convert("ml".to_string()), "ম্ল");
        assert_eq!(parser.convert("mb".to_string()), "ম্ব");
        assert_eq!(parser.convert("mf".to_string()), "ম্ফ");
        assert_eq!(parser.convert("m".to_string()), "ম");
        assert_eq!(parser.convert("0".to_string()), "০");
        assert_eq!(parser.convert("1".to_string()), "১");
        assert_eq!(parser.convert("2".to_string()), "২");
        assert_eq!(parser.convert("3".to_string()), "৩");
        assert_eq!(parser.convert("4".to_string()), "৪");
        assert_eq!(parser.convert("5".to_string()), "৫");
        assert_eq!(parser.convert("6".to_string()), "৬");
        assert_eq!(parser.convert("7".to_string()), "৭");
        assert_eq!(parser.convert("8".to_string()), "৮");
        assert_eq!(parser.convert("9".to_string()), "৯");
        assert_eq!(parser.convert("NgkSh".to_string()), "ঙ্ক্ষ");
        assert_eq!(parser.convert("Ngkkh".to_string()), "ঙ্ক্ষ");
        assert_eq!(parser.convert("NGch".to_string()), "ঞ্ছ");
        assert_eq!(parser.convert("Nggh".to_string()), "ঙ্ঘ");
        assert_eq!(parser.convert("Ngkh".to_string()), "ঙ্খ");
        assert_eq!(parser.convert("NGjh".to_string()), "ঞ্ঝ");
        assert_eq!(parser.convert("ngOU".to_string()), "ঙ্গৌ");
        assert_eq!(parser.convert("ngOI".to_string()), "ঙ্গৈ");
        assert_eq!(parser.convert("Ngkx".to_string()), "ঙ্ক্ষ");
        assert_eq!(parser.convert("NGc".to_string()), "ঞ্চ");
        assert_eq!(parser.convert("nch".to_string()), "ঞ্ছ");
        assert_eq!(parser.convert("njh".to_string()), "ঞ্ঝ");
        assert_eq!(parser.convert("ngh".to_string()), "ঙ্ঘ");
        assert_eq!(parser.convert("Ngk".to_string()), "ঙ্ক");
        assert_eq!(parser.convert("Ngx".to_string()), "ঙ্ষ");
        assert_eq!(parser.convert("Ngg".to_string()), "ঙ্গ");
        assert_eq!(parser.convert("Ngm".to_string()), "ঙ্ম");
        assert_eq!(parser.convert("NGj".to_string()), "ঞ্জ");
        assert_eq!(parser.convert("ndh".to_string()), "ন্ধ");
        assert_eq!(parser.convert("nTh".to_string()), "ন্ঠ");
        assert_eq!(parser.convert("NTh".to_string()), "ণ্ঠ");
        assert_eq!(parser.convert("nth".to_string()), "ন্থ");
        assert_eq!(parser.convert("nkh".to_string()), "ঙ্খ");
        assert_eq!(parser.convert("ngo".to_string()), "ঙ্গ");
        assert_eq!(parser.convert("nga".to_string()), "ঙ্গা");
        assert_eq!(parser.convert("ngi".to_string()), "ঙ্গি");
        assert_eq!(parser.convert("ngI".to_string()), "ঙ্গী");
        assert_eq!(parser.convert("ngu".to_string()), "ঙ্গু");
        assert_eq!(parser.convert("ngU".to_string()), "ঙ্গূ");
        assert_eq!(parser.convert("nge".to_string()), "ঙ্গে");
        assert_eq!(parser.convert("ngO".to_string()), "ঙ্গো");
        assert_eq!(parser.convert("NDh".to_string()), "ণ্ঢ");
        assert_eq!(parser.convert("nsh".to_string()), "নশ");
        assert_eq!(parser.convert("Ngr".to_string()), "ঙর");
        assert_eq!(parser.convert("NGr".to_string()), "ঞর");
        assert_eq!(parser.convert("ngr".to_string()), "ংর");
        assert_eq!(parser.convert("nj".to_string()), "ঞ্জ");
        assert_eq!(parser.convert("Ng".to_string()), "ঙ");
        assert_eq!(parser.convert("NG".to_string()), "ঞ");
        assert_eq!(parser.convert("nk".to_string()), "ঙ্ক");
        assert_eq!(parser.convert("ng".to_string()), "ং");
        assert_eq!(parser.convert("nn".to_string()), "ন্ন");
        assert_eq!(parser.convert("NN".to_string()), "ণ্ণ");
        assert_eq!(parser.convert("Nn".to_string()), "ণ্ন");
        assert_eq!(parser.convert("nm".to_string()), "ন্ম");
        assert_eq!(parser.convert("Nm".to_string()), "ণ্ম");
        assert_eq!(parser.convert("nd".to_string()), "ন্দ");
        assert_eq!(parser.convert("nT".to_string()), "ন্ট");
        assert_eq!(parser.convert("NT".to_string()), "ণ্ট");
        assert_eq!(parser.convert("nD".to_string()), "ন্ড");
        assert_eq!(parser.convert("ND".to_string()), "ণ্ড");
        assert_eq!(parser.convert("nt".to_string()), "ন্ত");
        assert_eq!(parser.convert("ns".to_string()), "ন্স");
        assert_eq!(parser.convert("nc".to_string()), "ঞ্চ");
        assert_eq!(parser.convert("n".to_string()), "ন");
        assert_eq!(parser.convert("N".to_string()), "ণ");
        assert_eq!(parser.convert("OI`".to_string()), "ৈ");
        assert_eq!(parser.convert("OU`".to_string()), "ৌ");
        assert_eq!(parser.convert("O`".to_string()), "ো");
        assert_eq!(parser.convert("OI".to_string()), "ঐ");
        assert_eq!(parser.convert("kOI".to_string()), "কৈ");
        assert_eq!(parser.convert(" OI".to_string()), " ঐ");
        assert_eq!(parser.convert("(OI".to_string()), "(ঐ");
        assert_eq!(parser.convert(".OI".to_string()), "।ঐ");
        assert_eq!(parser.convert("OU".to_string()), "ঔ");
        assert_eq!(parser.convert("kOU".to_string()), "কৌ");
        assert_eq!(parser.convert(" OU".to_string()), " ঔ");
        assert_eq!(parser.convert("-OU".to_string()), "-ঔ");
        assert_eq!(parser.convert(",,OU".to_string()), "্‌ঔ");
        assert_eq!(parser.convert("O".to_string()), "ও");
        assert_eq!(parser.convert("pO".to_string()), "পো");
        assert_eq!(parser.convert(" O".to_string()), " ও");
        assert_eq!(parser.convert("iO".to_string()), "ইও");
        assert_eq!(parser.convert("`O".to_string()), "ও");
        assert_eq!(parser.convert("phl".to_string()), "ফ্ল");
        assert_eq!(parser.convert("pT".to_string()), "প্ট");
        assert_eq!(parser.convert("pt".to_string()), "প্ত");
        assert_eq!(parser.convert("pn".to_string()), "প্ন");
        assert_eq!(parser.convert("pp".to_string()), "প্প");
        assert_eq!(parser.convert("pl".to_string()), "প্ল");
        assert_eq!(parser.convert("ps".to_string()), "প্স");
        assert_eq!(parser.convert("ph".to_string()), "ফ");
        assert_eq!(parser.convert("fl".to_string()), "ফ্ল");
        assert_eq!(parser.convert("f".to_string()), "ফ");
        assert_eq!(parser.convert("p".to_string()), "প");
        assert_eq!(parser.convert("rri`".to_string()), "ৃ");
        assert_eq!(parser.convert("rri".to_string()), "ঋ");
        assert_eq!(parser.convert("krri".to_string()), "কৃ");
        assert_eq!(parser.convert("Irri".to_string()), "ঈঋ");
        assert_eq!(parser.convert("^rri".to_string()), "ঁঋ");
        assert_eq!(parser.convert(":rri".to_string()), "ঃঋ");
        assert_eq!(parser.convert("rZ".to_string()), "র‍্য");
        assert_eq!(parser.convert("krZ".to_string()), "ক্র্য");
        assert_eq!(parser.convert("rrZ".to_string()), "রর‍্য");
        assert_eq!(parser.convert("yrZ".to_string()), "ইয়র‍্য");
        assert_eq!(parser.convert("wrZ".to_string()), "ওর‍্য");
        assert_eq!(parser.convert("xrZ".to_string()), "এক্সর‍্য");
        assert_eq!(parser.convert("irZ".to_string()), "ইর‍্য");
        assert_eq!(parser.convert("-rZ".to_string()), "-র‍্য");
        assert_eq!(parser.convert("rrrZ".to_string()), "ররর‍্য");
        assert_eq!(parser.convert("ry".to_string()), "র‍্য");
        assert_eq!(parser.convert("qry".to_string()), "ক্র্য");
        assert_eq!(parser.convert("rry".to_string()), "রর‍্য");
        assert_eq!(parser.convert("yry".to_string()), "ইয়র‍্য");
        assert_eq!(parser.convert("wry".to_string()), "ওর‍্য");
        assert_eq!(parser.convert("xry".to_string()), "এক্সর‍্য");
        assert_eq!(parser.convert("0ry".to_string()), "০র‍্য");
        assert_eq!(parser.convert("rrrry".to_string()), "রররর‍্য");
        assert_eq!(parser.convert("Rry".to_string()), "ড়্র্য");
        assert_eq!(parser.convert("rr".to_string()), "রর");
        assert_eq!(parser.convert("arr".to_string()), "আরর");
        assert_eq!(parser.convert("arrk".to_string()), "আর্ক");
        assert_eq!(parser.convert("arra".to_string()), "আররা");
        assert_eq!(parser.convert("arr".to_string()), "আরর");
        assert_eq!(parser.convert("arr!".to_string()), "আরর!");
        assert_eq!(parser.convert("krr".to_string()), "ক্রর");
        assert_eq!(parser.convert("krra".to_string()), "ক্ররা");
        assert_eq!(parser.convert("Rg".to_string()), "ড়্গ");
        assert_eq!(parser.convert("Rh".to_string()), "ঢ়");
        assert_eq!(parser.convert("R".to_string()), "ড়");
        assert_eq!(parser.convert("r".to_string()), "র");
        assert_eq!(parser.convert("or".to_string()), "অর");
        assert_eq!(parser.convert("mr".to_string()), "ম্র");
        assert_eq!(parser.convert("1r".to_string()), "১র");
        assert_eq!(parser.convert("+r".to_string()), "+র");
        assert_eq!(parser.convert("rr".to_string()), "রর");
        assert_eq!(parser.convert("yr".to_string()), "ইয়র");
        assert_eq!(parser.convert("wr".to_string()), "ওর");
        assert_eq!(parser.convert("xr".to_string()), "এক্সর");
        assert_eq!(parser.convert("zr".to_string()), "য্র");
        assert_eq!(parser.convert("mri".to_string()), "ম্রি");
        assert_eq!(parser.convert("shch".to_string()), "শ্ছ");
        assert_eq!(parser.convert("ShTh".to_string()), "ষ্ঠ");
        assert_eq!(parser.convert("Shph".to_string()), "ষ্ফ");
        assert_eq!(parser.convert("Sch".to_string()), "শ্ছ");
        assert_eq!(parser.convert("skl".to_string()), "স্ক্ল");
        assert_eq!(parser.convert("skh".to_string()), "স্খ");
        assert_eq!(parser.convert("sth".to_string()), "স্থ");
        assert_eq!(parser.convert("sph".to_string()), "স্ফ");
        assert_eq!(parser.convert("shc".to_string()), "শ্চ");
        assert_eq!(parser.convert("sht".to_string()), "শ্ত");
        assert_eq!(parser.convert("shn".to_string()), "শ্ন");
        assert_eq!(parser.convert("shm".to_string()), "শ্ম");
        assert_eq!(parser.convert("shl".to_string()), "শ্ল");
        assert_eq!(parser.convert("Shk".to_string()), "ষ্ক");
        assert_eq!(parser.convert("ShT".to_string()), "ষ্ট");
        assert_eq!(parser.convert("ShN".to_string()), "ষ্ণ");
        assert_eq!(parser.convert("Shp".to_string()), "ষ্প");
        assert_eq!(parser.convert("Shf".to_string()), "ষ্ফ");
        assert_eq!(parser.convert("Shm".to_string()), "ষ্ম");
        assert_eq!(parser.convert("spl".to_string()), "স্প্ল");
        assert_eq!(parser.convert("sk".to_string()), "স্ক");
        assert_eq!(parser.convert("Sc".to_string()), "শ্চ");
        assert_eq!(parser.convert("sT".to_string()), "স্ট");
        assert_eq!(parser.convert("st".to_string()), "স্ত");
        assert_eq!(parser.convert("sn".to_string()), "স্ন");
        assert_eq!(parser.convert("sp".to_string()), "স্প");
        assert_eq!(parser.convert("sf".to_string()), "স্ফ");
        assert_eq!(parser.convert("sm".to_string()), "স্ম");
        assert_eq!(parser.convert("sl".to_string()), "স্ল");
        assert_eq!(parser.convert("sh".to_string()), "শ");
        assert_eq!(parser.convert("Sc".to_string()), "শ্চ");
        assert_eq!(parser.convert("St".to_string()), "শ্ত");
        assert_eq!(parser.convert("Sn".to_string()), "শ্ন");
        assert_eq!(parser.convert("Sm".to_string()), "শ্ম");
        assert_eq!(parser.convert("Sl".to_string()), "শ্ল");
        assert_eq!(parser.convert("Sh".to_string()), "ষ");
        assert_eq!(parser.convert("s".to_string()), "স");
        assert_eq!(parser.convert("S".to_string()), "শ");
        assert_eq!(parser.convert("oo".to_string()), "উ");
        assert_eq!(parser.convert("OO".to_string()), "ওও");
        assert_eq!(parser.convert("oo`".to_string()), "ু");
        assert_eq!(parser.convert("koo".to_string()), "কু");
        assert_eq!(parser.convert("ooo".to_string()), "উঅ");
        assert_eq!(parser.convert("!oo".to_string()), "!উ");
        assert_eq!(parser.convert("!ooo".to_string()), "!উঅ");
        assert_eq!(parser.convert("aoo".to_string()), "আউ");
        assert_eq!(parser.convert("oop".to_string()), "উপ");
        assert_eq!(parser.convert("ooo`".to_string()), "উ");
        assert_eq!(parser.convert("o`".to_string()), "");
        assert_eq!(parser.convert("oZ".to_string()), "অ্য");
        assert_eq!(parser.convert("oY".to_string()), "অয়");
        assert_eq!(parser.convert("o".to_string()), "অ");
        assert_eq!(parser.convert("!o".to_string()), "!অ");
        assert_eq!(parser.convert("^o".to_string()), "ঁঅ");
        assert_eq!(parser.convert("*o".to_string()), "*অ");
        assert_eq!(parser.convert("io".to_string()), "ইও");
        assert_eq!(parser.convert("yo".to_string()), "ইয়");
        assert_eq!(parser.convert("no".to_string()), "ন");
        assert_eq!(parser.convert("tth".to_string()), "ত্থ");
        assert_eq!(parser.convert("t``".to_string()), "ৎ");
        assert_eq!(parser.convert("`t``".to_string()), "ৎ");
        assert_eq!(parser.convert("t``t``".to_string()), "ৎৎ");
        assert_eq!(parser.convert("t```".to_string()), "ৎ");
        assert_eq!(parser.convert("TT".to_string()), "ট্ট");
        assert_eq!(parser.convert("Tm".to_string()), "ট্ম");
        assert_eq!(parser.convert("Th".to_string()), "ঠ");
        assert_eq!(parser.convert("tn".to_string()), "ত্ন");
        assert_eq!(parser.convert("tm".to_string()), "ত্ম");
        assert_eq!(parser.convert("th".to_string()), "থ");
        assert_eq!(parser.convert("tt".to_string()), "ত্ত");
        assert_eq!(parser.convert("T".to_string()), "ট");
        assert_eq!(parser.convert("t".to_string()), "ত");
        assert_eq!(parser.convert("aZ".to_string()), "অ্যা");
        assert_eq!(parser.convert("aaZ".to_string()), "আঅ্যা");
        assert_eq!(parser.convert("AZ".to_string()), "অ্যা");
        assert_eq!(parser.convert("a`".to_string()), "া");
        assert_eq!(parser.convert("a``".to_string()), "া");
        assert_eq!(parser.convert("ka`".to_string()), "কা");
        assert_eq!(parser.convert("A`".to_string()), "া");
        assert_eq!(parser.convert("a".to_string()), "আ");
        assert_eq!(parser.convert("`a".to_string()), "আ");
        assert_eq!(parser.convert("k`a".to_string()), "কআ");
        assert_eq!(parser.convert("ia".to_string()), "ইয়া");
        assert_eq!(parser.convert("aaaa`".to_string()), "আআআা");
        assert_eq!(parser.convert("i`".to_string()), "ি");
        assert_eq!(parser.convert("i".to_string()), "ই");
        assert_eq!(parser.convert("`i".to_string()), "ই");
        assert_eq!(parser.convert("hi".to_string()), "হি");
        assert_eq!(parser.convert("ih".to_string()), "ইহ");
        assert_eq!(parser.convert("i`h".to_string()), "িহ");
        assert_eq!(parser.convert("I`".to_string()), "ী");
        assert_eq!(parser.convert("I".to_string()), "ঈ");
        assert_eq!(parser.convert("cI".to_string()), "চী");
        assert_eq!(parser.convert("Ix".to_string()), "ঈক্স");
        assert_eq!(parser.convert("II".to_string()), "ঈঈ");
        assert_eq!(parser.convert("0I".to_string()), "০ঈ");
        assert_eq!(parser.convert("oI".to_string()), "অঈ");
        assert_eq!(parser.convert("u`".to_string()), "ু");
        assert_eq!(parser.convert("u".to_string()), "উ");
        assert_eq!(parser.convert("ku".to_string()), "কু");
        assert_eq!(parser.convert("uk".to_string()), "উক");
        assert_eq!(parser.convert("uu".to_string()), "উউ");
        assert_eq!(parser.convert("iu".to_string()), "ইউ");
        assert_eq!(parser.convert("&u".to_string()), "&উ");
        assert_eq!(parser.convert("u&".to_string()), "উ&");
        assert_eq!(parser.convert("U`".to_string()), "ূ");
        assert_eq!(parser.convert("U".to_string()), "ঊ");
        assert_eq!(parser.convert("yU".to_string()), "ইয়ূ");
        assert_eq!(parser.convert("Uy".to_string()), "ঊয়");
        assert_eq!(parser.convert("^U".to_string()), "ঁঊ");
        assert_eq!(parser.convert("U^".to_string()), "ঊঁ");
        assert_eq!(parser.convert("EE".to_string()), "ঈ");
        assert_eq!(parser.convert("ee".to_string()), "ঈ");
        assert_eq!(parser.convert("Ee".to_string()), "ঈ");
        assert_eq!(parser.convert("eE".to_string()), "ঈ");
        assert_eq!(parser.convert("ee`".to_string()), "ী");
        assert_eq!(parser.convert("kee".to_string()), "কী");
        assert_eq!(parser.convert("eek".to_string()), "ঈক");
        assert_eq!(parser.convert("0ee".to_string()), "০ঈ");
        assert_eq!(parser.convert("ee8".to_string()), "ঈ৮");
        assert_eq!(parser.convert("(ee)".to_string()), "(ঈ)");
        assert_eq!(parser.convert("e`".to_string()), "ে");
        assert_eq!(parser.convert("e".to_string()), "এ");
        assert_eq!(parser.convert("ke".to_string()), "কে");
        assert_eq!(parser.convert("we".to_string()), "ওয়ে");
        assert_eq!(parser.convert("#e#".to_string()), "#এ#");
        assert_eq!(parser.convert("`e`".to_string()), "ে");
        assert_eq!(parser.convert("z".to_string()), "য");
        assert_eq!(parser.convert("Z".to_string()), "্য");
        assert_eq!(parser.convert("rZ".to_string()), "র‍্য");
        assert_eq!(parser.convert("kZS".to_string()), "ক্যশ");
        assert_eq!(parser.convert("y".to_string()), "ইয়");
        assert_eq!(parser.convert("oy".to_string()), "অয়");
        assert_eq!(parser.convert("ky".to_string()), "ক্য");
        assert_eq!(parser.convert("ya".to_string()), "ইয়া");
        assert_eq!(parser.convert("yaa".to_string()), "ইয়াআ");
        assert_eq!(parser.convert("Y".to_string()), "য়");
        assert_eq!(parser.convert("YY".to_string()), "য়য়");
        assert_eq!(parser.convert("iY".to_string()), "ইয়");
        assert_eq!(parser.convert("kY".to_string()), "কয়");
        assert_eq!(parser.convert("q".to_string()), "ক");
        assert_eq!(parser.convert("Q".to_string()), "ক");
        assert_eq!(parser.convert("w".to_string()), "ও");
        assert_eq!(parser.convert("wa".to_string()), "ওয়া");
        assert_eq!(parser.convert("-wa-".to_string()), "-ওয়া-");
        assert_eq!(parser.convert("woo".to_string()), "ওয়ু");
        assert_eq!(parser.convert("wre".to_string()), "ওরে");
        assert_eq!(parser.convert("kw".to_string()), "ক্ব");
        assert_eq!(parser.convert("x".to_string()), "এক্স");
        assert_eq!(parser.convert("ex".to_string()), "এক্স");
        assert_eq!(parser.convert("bx".to_string()), "বক্স");
        assert_eq!(parser.convert(":`".to_string()), ":");
        assert_eq!(parser.convert(":".to_string()), "ঃ");
        assert_eq!(parser.convert("^`".to_string()), "^");
        assert_eq!(parser.convert("^".to_string()), "ঁ");
        assert_eq!(parser.convert("k^".to_string()), "কঁ");
        assert_eq!(parser.convert("k^i".to_string()), "কঁই");
        assert_eq!(parser.convert("ki^".to_string()), "কিঁ");
        assert_eq!(parser.convert(",,".to_string()), "্‌");
        assert_eq!(parser.convert(",,,".to_string()), "্‌,");
        assert_eq!(parser.convert(",,`,".to_string()), "্‌,");
        assert_eq!(parser.convert("`,,".to_string()), "্‌");
        assert_eq!(parser.convert(",`,".to_string()), ",,");
        assert_eq!(parser.convert("$".to_string()), "৳");
        assert_eq!(parser.convert("`".to_string()), "");
        assert_eq!(parser.convert("bdh".to_string()), "ব্ধ");
    }

    #[test]
    fn test_sentence() {
        let json = json::parse(include_str!("AvroPhonetic.json")).unwrap();
        let parser = PhoneticParser::new(&json);

        assert_eq!(parser.convert("ami banglay gan gai".to_string()),  "আমি বাংলায় গান গাই");
        assert_eq!(parser.convert("amader valObasa hoye gel ghas, kheye gel goru ar diye gelo ba^sh".to_string()),  "আমাদের ভালোবাসা হয়ে গেল ঘাস, খেয়ে গেল গরু আর দিয়ে গেল বাঁশ");
    }
}

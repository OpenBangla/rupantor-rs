#![allow(unused_assignments)]
use std::cmp::Ordering;
use serde_json::Value;
use stringplus::StringPlus;

/// Parses and converts text into Bengali according to given grammar.
pub struct PhoneticParser {
    patterns: Vec<Value>,
    vowel: String,
    consonant: String,
    numbers: String,
    case_sensitive: String,
    max_pattern_len: usize,
}

impl PhoneticParser {
    /// Creates a new `PhoneticParser` instance from the given Json
    /// value. The Json value must need to be a Json Object containing
    /// the required values, otherwise a panic would occur.
    pub fn new(rule: &Value) -> PhoneticParser {
        PhoneticParser {
            patterns: rule["patterns"].as_array().unwrap().clone(),
            vowel: rule["vowel"].as_str().unwrap().to_string(),
            consonant: rule["consonant"].as_str().unwrap().to_string(),
            numbers: rule["number"].as_str().unwrap().to_string(),
            case_sensitive: rule["casesensitive"].as_str().unwrap().to_string(),
            max_pattern_len: rule["patterns"][0]["find"].as_str().unwrap().len(),
        }
    }

    /// Converts the given input string into Bengali according to the grammar.
    pub fn convert(&self, input: &str) -> String {
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
                            let rules = pattern["rules"].as_array().unwrap();
                            if !rules.is_empty() {
                                for rule in rules {
                                    let mut replace = true;
                                    let mut chk = 0;
                                    let matches = rule["matches"].as_array().unwrap();
                                    for _match in matches {
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

    fn fix_string(&self, string: &str) -> String {
        string
            .chars()
            .map(|character| {
                if self.is_case_sensitive(character) {
                    character
                } else {
                    character.to_ascii_lowercase()
                }
            })
            .collect()
    }

    fn is_vowel(&self, string: &str) -> bool {
        self.vowel.contains(&string.to_ascii_lowercase())
    }

    fn is_consonant(&self, string: &str) -> bool {
        self.consonant.contains(&string.to_ascii_lowercase())
    }

    fn is_case_sensitive(&self, character: char) -> bool {
        self.case_sensitive
            .contains(character.to_ascii_lowercase())
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

#[cfg(test)]
mod tests {
    use serde_json;
    use parser::PhoneticParser;

    #[test]
    fn test_helpers() {
        let json = serde_json::from_str(include_str!("AvroPhonetic.json")).unwrap();
        let parser = PhoneticParser::new(&json);

        assert!(parser.is_vowel("A"));
        assert_eq!(parser.is_vowel("b"), false);
        assert!(parser.is_consonant("B"));
        assert_eq!(parser.is_consonant("e"), false);
        assert_eq!(parser.fix_string("ODEr AMAr"), "ODer amar");
        assert!(parser.is_number("1"));
    }

    #[test]
    fn test_basic() {
        let json = serde_json::from_str(include_str!("AvroPhonetic.json")).unwrap();
        let parser = PhoneticParser::new(&json);

        assert_eq!(parser.convert("bhl"), "ভ্ল");
        assert_eq!(parser.convert("bj"), "ব্জ");
        assert_eq!(parser.convert("bd"), "ব্দ");
        assert_eq!(parser.convert("bb"), "ব্ব");
        assert_eq!(parser.convert("bl"), "ব্ল");
        assert_eq!(parser.convert("bh"), "ভ");
        assert_eq!(parser.convert("vl"), "ভ্ল");
        assert_eq!(parser.convert("b"), "ব");
        assert_eq!(parser.convert("v"), "ভ");
        assert_eq!(parser.convert("cNG"), "চ্ঞ");
        assert_eq!(parser.convert("cch"), "চ্ছ");
        assert_eq!(parser.convert("cc"), "চ্চ");
        assert_eq!(parser.convert("ch"), "ছ");
        assert_eq!(parser.convert("c"), "চ");
        assert_eq!(parser.convert("dhn"), "ধ্ন");
        assert_eq!(parser.convert("dhm"), "ধ্ম");
        assert_eq!(parser.convert("dgh"), "দ্ঘ");
        assert_eq!(parser.convert("ddh"), "দ্ধ");
        assert_eq!(parser.convert("dbh"), "দ্ভ");
        assert_eq!(parser.convert("dv"), "দ্ভ");
        assert_eq!(parser.convert("dm"), "দ্ম");
        assert_eq!(parser.convert("DD"), "ড্ড");
        assert_eq!(parser.convert("Dh"), "ঢ");
        assert_eq!(parser.convert("dh"), "ধ");
        assert_eq!(parser.convert("dg"), "দ্গ");
        assert_eq!(parser.convert("dd"), "দ্দ");
        assert_eq!(parser.convert("D"), "ড");
        assert_eq!(parser.convert("d"), "দ");
        assert_eq!(parser.convert("..."), "...");
        assert_eq!(parser.convert(".`"), ".");
        assert_eq!(parser.convert(".."), "।।");
        assert_eq!(parser.convert("."), "।");
        assert_eq!(parser.convert("ghn"), "ঘ্ন");
        assert_eq!(parser.convert("Ghn"), "ঘ্ন");
        assert_eq!(parser.convert("gdh"), "গ্ধ");
        assert_eq!(parser.convert("gN"), "গ্ণ");
        assert_eq!(parser.convert("GN"), "গ্ণ");
        assert_eq!(parser.convert("gn"), "গ্ন");
        assert_eq!(parser.convert("gm"), "গ্ম");
        assert_eq!(parser.convert("Gm"), "গ্ম");
        assert_eq!(parser.convert("gl"), "গ্ল");
        assert_eq!(parser.convert("Gl"), "গ্ল");
        assert_eq!(parser.convert("gg"), "জ্ঞ");
        assert_eq!(parser.convert("GG"), "জ্ঞ");
        assert_eq!(parser.convert("Gg"), "জ্ঞ");
        assert_eq!(parser.convert("gG"), "জ্ঞ");
        assert_eq!(parser.convert("gh"), "ঘ");
        assert_eq!(parser.convert("Gh"), "ঘ");
        assert_eq!(parser.convert("g"), "গ");
        assert_eq!(parser.convert("hN"), "হ্ণ");
        assert_eq!(parser.convert("hn"), "হ্ন");
        assert_eq!(parser.convert("hm"), "হ্ম");
        assert_eq!(parser.convert("hl"), "হ্ল");
        assert_eq!(parser.convert("h"), "হ");
        assert_eq!(parser.convert("jjh"), "জ্ঝ");
        assert_eq!(parser.convert("jNG"), "জ্ঞ");
        assert_eq!(parser.convert("jh"), "ঝ");
        assert_eq!(parser.convert("jj"), "জ্জ");
        assert_eq!(parser.convert("j"), "জ");
        assert_eq!(parser.convert("J"), "জ");
        assert_eq!(parser.convert("kkhN"), "ক্ষ্ণ");
        assert_eq!(parser.convert("kShN"), "ক্ষ্ণ");
        assert_eq!(parser.convert("kkhm"), "ক্ষ্ম");
        assert_eq!(parser.convert("kShm"), "ক্ষ্ম");
        assert_eq!(parser.convert("kxN"), "ক্ষ্ণ");
        assert_eq!(parser.convert("kxm"), "ক্ষ্ম");
        assert_eq!(parser.convert("kkh"), "ক্ষ");
        assert_eq!(parser.convert("kSh"), "ক্ষ");
        assert_eq!(parser.convert("ksh"), "কশ");
        assert_eq!(parser.convert("kx"), "ক্ষ");
        assert_eq!(parser.convert("kk"), "ক্ক");
        assert_eq!(parser.convert("kT"), "ক্ট");
        assert_eq!(parser.convert("kt"), "ক্ত");
        assert_eq!(parser.convert("kl"), "ক্ল");
        assert_eq!(parser.convert("ks"), "ক্স");
        assert_eq!(parser.convert("kh"), "খ");
        assert_eq!(parser.convert("k"), "ক");
        assert_eq!(parser.convert("lbh"), "ল্ভ");
        assert_eq!(parser.convert("ldh"), "ল্ধ");
        assert_eq!(parser.convert("lkh"), "লখ");
        assert_eq!(parser.convert("lgh"), "লঘ");
        assert_eq!(parser.convert("lph"), "লফ");
        assert_eq!(parser.convert("lk"), "ল্ক");
        assert_eq!(parser.convert("lg"), "ল্গ");
        assert_eq!(parser.convert("lT"), "ল্ট");
        assert_eq!(parser.convert("lD"), "ল্ড");
        assert_eq!(parser.convert("lp"), "ল্প");
        assert_eq!(parser.convert("lv"), "ল্ভ");
        assert_eq!(parser.convert("lm"), "ল্ম");
        assert_eq!(parser.convert("ll"), "ল্ল");
        assert_eq!(parser.convert("lb"), "ল্ব");
        assert_eq!(parser.convert("l"), "ল");
        assert_eq!(parser.convert("mth"), "ম্থ");
        assert_eq!(parser.convert("mph"), "ম্ফ");
        assert_eq!(parser.convert("mbh"), "ম্ভ");
        assert_eq!(parser.convert("mpl"), "মপ্ল");
        assert_eq!(parser.convert("mn"), "ম্ন");
        assert_eq!(parser.convert("mp"), "ম্প");
        assert_eq!(parser.convert("mv"), "ম্ভ");
        assert_eq!(parser.convert("mm"), "ম্ম");
        assert_eq!(parser.convert("ml"), "ম্ল");
        assert_eq!(parser.convert("mb"), "ম্ব");
        assert_eq!(parser.convert("mf"), "ম্ফ");
        assert_eq!(parser.convert("m"), "ম");
        assert_eq!(parser.convert("0"), "০");
        assert_eq!(parser.convert("1"), "১");
        assert_eq!(parser.convert("2"), "২");
        assert_eq!(parser.convert("3"), "৩");
        assert_eq!(parser.convert("4"), "৪");
        assert_eq!(parser.convert("5"), "৫");
        assert_eq!(parser.convert("6"), "৬");
        assert_eq!(parser.convert("7"), "৭");
        assert_eq!(parser.convert("8"), "৮");
        assert_eq!(parser.convert("9"), "৯");
        assert_eq!(parser.convert("NgkSh"), "ঙ্ক্ষ");
        assert_eq!(parser.convert("Ngkkh"), "ঙ্ক্ষ");
        assert_eq!(parser.convert("NGch"), "ঞ্ছ");
        assert_eq!(parser.convert("Nggh"), "ঙ্ঘ");
        assert_eq!(parser.convert("Ngkh"), "ঙ্খ");
        assert_eq!(parser.convert("NGjh"), "ঞ্ঝ");
        assert_eq!(parser.convert("ngOU"), "ঙ্গৌ");
        assert_eq!(parser.convert("ngOI"), "ঙ্গৈ");
        assert_eq!(parser.convert("Ngkx"), "ঙ্ক্ষ");
        assert_eq!(parser.convert("NGc"), "ঞ্চ");
        assert_eq!(parser.convert("nch"), "ঞ্ছ");
        assert_eq!(parser.convert("njh"), "ঞ্ঝ");
        assert_eq!(parser.convert("ngh"), "ঙ্ঘ");
        assert_eq!(parser.convert("Ngk"), "ঙ্ক");
        assert_eq!(parser.convert("Ngx"), "ঙ্ষ");
        assert_eq!(parser.convert("Ngg"), "ঙ্গ");
        assert_eq!(parser.convert("Ngm"), "ঙ্ম");
        assert_eq!(parser.convert("NGj"), "ঞ্জ");
        assert_eq!(parser.convert("ndh"), "ন্ধ");
        assert_eq!(parser.convert("nTh"), "ন্ঠ");
        assert_eq!(parser.convert("NTh"), "ণ্ঠ");
        assert_eq!(parser.convert("nth"), "ন্থ");
        assert_eq!(parser.convert("nkh"), "ঙ্খ");
        assert_eq!(parser.convert("ngo"), "ঙ্গ");
        assert_eq!(parser.convert("nga"), "ঙ্গা");
        assert_eq!(parser.convert("ngi"), "ঙ্গি");
        assert_eq!(parser.convert("ngI"), "ঙ্গী");
        assert_eq!(parser.convert("ngu"), "ঙ্গু");
        assert_eq!(parser.convert("ngU"), "ঙ্গূ");
        assert_eq!(parser.convert("nge"), "ঙ্গে");
        assert_eq!(parser.convert("ngO"), "ঙ্গো");
        assert_eq!(parser.convert("NDh"), "ণ্ঢ");
        assert_eq!(parser.convert("nsh"), "নশ");
        assert_eq!(parser.convert("Ngr"), "ঙর");
        assert_eq!(parser.convert("NGr"), "ঞর");
        assert_eq!(parser.convert("ngr"), "ংর");
        assert_eq!(parser.convert("nj"), "ঞ্জ");
        assert_eq!(parser.convert("Ng"), "ঙ");
        assert_eq!(parser.convert("NG"), "ঞ");
        assert_eq!(parser.convert("nk"), "ঙ্ক");
        assert_eq!(parser.convert("ng"), "ং");
        assert_eq!(parser.convert("nn"), "ন্ন");
        assert_eq!(parser.convert("NN"), "ণ্ণ");
        assert_eq!(parser.convert("Nn"), "ণ্ন");
        assert_eq!(parser.convert("nm"), "ন্ম");
        assert_eq!(parser.convert("Nm"), "ণ্ম");
        assert_eq!(parser.convert("nd"), "ন্দ");
        assert_eq!(parser.convert("nT"), "ন্ট");
        assert_eq!(parser.convert("NT"), "ণ্ট");
        assert_eq!(parser.convert("nD"), "ন্ড");
        assert_eq!(parser.convert("ND"), "ণ্ড");
        assert_eq!(parser.convert("nt"), "ন্ত");
        assert_eq!(parser.convert("ns"), "ন্স");
        assert_eq!(parser.convert("nc"), "ঞ্চ");
        assert_eq!(parser.convert("n"), "ন");
        assert_eq!(parser.convert("N"), "ণ");
        assert_eq!(parser.convert("OI`"), "ৈ");
        assert_eq!(parser.convert("OU`"), "ৌ");
        assert_eq!(parser.convert("O`"), "ো");
        assert_eq!(parser.convert("OI"), "ঐ");
        assert_eq!(parser.convert("kOI"), "কৈ");
        assert_eq!(parser.convert(" OI"), " ঐ");
        assert_eq!(parser.convert("(OI"), "(ঐ");
        assert_eq!(parser.convert(".OI"), "।ঐ");
        assert_eq!(parser.convert("OU"), "ঔ");
        assert_eq!(parser.convert("kOU"), "কৌ");
        assert_eq!(parser.convert(" OU"), " ঔ");
        assert_eq!(parser.convert("-OU"), "-ঔ");
        assert_eq!(parser.convert(",,OU"), "্‌ঔ");
        assert_eq!(parser.convert("O"), "ও");
        assert_eq!(parser.convert("pO"), "পো");
        assert_eq!(parser.convert(" O"), " ও");
        assert_eq!(parser.convert("iO"), "ইও");
        assert_eq!(parser.convert("`O"), "ও");
        assert_eq!(parser.convert("phl"), "ফ্ল");
        assert_eq!(parser.convert("pT"), "প্ট");
        assert_eq!(parser.convert("pt"), "প্ত");
        assert_eq!(parser.convert("pn"), "প্ন");
        assert_eq!(parser.convert("pp"), "প্প");
        assert_eq!(parser.convert("pl"), "প্ল");
        assert_eq!(parser.convert("ps"), "প্স");
        assert_eq!(parser.convert("ph"), "ফ");
        assert_eq!(parser.convert("fl"), "ফ্ল");
        assert_eq!(parser.convert("f"), "ফ");
        assert_eq!(parser.convert("p"), "প");
        assert_eq!(parser.convert("rri`"), "ৃ");
        assert_eq!(parser.convert("rri"), "ঋ");
        assert_eq!(parser.convert("krri"), "কৃ");
        assert_eq!(parser.convert("Irri"), "ঈঋ");
        assert_eq!(parser.convert("^rri"), "ঁঋ");
        assert_eq!(parser.convert(":rri"), "ঃঋ");
        assert_eq!(parser.convert("rZ"), "র‍্য");
        assert_eq!(parser.convert("krZ"), "ক্র্য");
        assert_eq!(parser.convert("rrZ"), "রর‍্য");
        assert_eq!(parser.convert("yrZ"), "ইয়র‍্য");
        assert_eq!(parser.convert("wrZ"), "ওর‍্য");
        assert_eq!(parser.convert("xrZ"), "এক্সর‍্য");
        assert_eq!(parser.convert("irZ"), "ইর‍্য");
        assert_eq!(parser.convert("-rZ"), "-র‍্য");
        assert_eq!(parser.convert("rrrZ"), "ররর‍্য");
        assert_eq!(parser.convert("ry"), "র‍্য");
        assert_eq!(parser.convert("qry"), "ক্র্য");
        assert_eq!(parser.convert("rry"), "রর‍্য");
        assert_eq!(parser.convert("yry"), "ইয়র‍্য");
        assert_eq!(parser.convert("wry"), "ওর‍্য");
        assert_eq!(parser.convert("xry"), "এক্সর‍্য");
        assert_eq!(parser.convert("0ry"), "০র‍্য");
        assert_eq!(parser.convert("rrrry"), "রররর‍্য");
        assert_eq!(parser.convert("Rry"), "ড়্র্য");
        assert_eq!(parser.convert("rr"), "রর");
        assert_eq!(parser.convert("arr"), "আরর");
        assert_eq!(parser.convert("arrk"), "আর্ক");
        assert_eq!(parser.convert("arra"), "আররা");
        assert_eq!(parser.convert("arr"), "আরর");
        assert_eq!(parser.convert("arr!"), "আরর!");
        assert_eq!(parser.convert("krr"), "ক্রর");
        assert_eq!(parser.convert("krra"), "ক্ররা");
        assert_eq!(parser.convert("Rg"), "ড়্গ");
        assert_eq!(parser.convert("Rh"), "ঢ়");
        assert_eq!(parser.convert("R"), "ড়");
        assert_eq!(parser.convert("r"), "র");
        assert_eq!(parser.convert("or"), "অর");
        assert_eq!(parser.convert("mr"), "ম্র");
        assert_eq!(parser.convert("1r"), "১র");
        assert_eq!(parser.convert("+r"), "+র");
        assert_eq!(parser.convert("rr"), "রর");
        assert_eq!(parser.convert("yr"), "ইয়র");
        assert_eq!(parser.convert("wr"), "ওর");
        assert_eq!(parser.convert("xr"), "এক্সর");
        assert_eq!(parser.convert("zr"), "য্র");
        assert_eq!(parser.convert("mri"), "ম্রি");
        assert_eq!(parser.convert("shch"), "শ্ছ");
        assert_eq!(parser.convert("ShTh"), "ষ্ঠ");
        assert_eq!(parser.convert("Shph"), "ষ্ফ");
        assert_eq!(parser.convert("Sch"), "শ্ছ");
        assert_eq!(parser.convert("skl"), "স্ক্ল");
        assert_eq!(parser.convert("skh"), "স্খ");
        assert_eq!(parser.convert("sth"), "স্থ");
        assert_eq!(parser.convert("sph"), "স্ফ");
        assert_eq!(parser.convert("shc"), "শ্চ");
        assert_eq!(parser.convert("sht"), "শ্ত");
        assert_eq!(parser.convert("shn"), "শ্ন");
        assert_eq!(parser.convert("shm"), "শ্ম");
        assert_eq!(parser.convert("shl"), "শ্ল");
        assert_eq!(parser.convert("Shk"), "ষ্ক");
        assert_eq!(parser.convert("ShT"), "ষ্ট");
        assert_eq!(parser.convert("ShN"), "ষ্ণ");
        assert_eq!(parser.convert("Shp"), "ষ্প");
        assert_eq!(parser.convert("Shf"), "ষ্ফ");
        assert_eq!(parser.convert("Shm"), "ষ্ম");
        assert_eq!(parser.convert("spl"), "স্প্ল");
        assert_eq!(parser.convert("sk"), "স্ক");
        assert_eq!(parser.convert("Sc"), "শ্চ");
        assert_eq!(parser.convert("sT"), "স্ট");
        assert_eq!(parser.convert("st"), "স্ত");
        assert_eq!(parser.convert("sn"), "স্ন");
        assert_eq!(parser.convert("sp"), "স্প");
        assert_eq!(parser.convert("sf"), "স্ফ");
        assert_eq!(parser.convert("sm"), "স্ম");
        assert_eq!(parser.convert("sl"), "স্ল");
        assert_eq!(parser.convert("sh"), "শ");
        assert_eq!(parser.convert("Sc"), "শ্চ");
        assert_eq!(parser.convert("St"), "শ্ত");
        assert_eq!(parser.convert("Sn"), "শ্ন");
        assert_eq!(parser.convert("Sm"), "শ্ম");
        assert_eq!(parser.convert("Sl"), "শ্ল");
        assert_eq!(parser.convert("Sh"), "ষ");
        assert_eq!(parser.convert("s"), "স");
        assert_eq!(parser.convert("S"), "শ");
        assert_eq!(parser.convert("oo"), "উ");
        assert_eq!(parser.convert("OO"), "ওও");
        assert_eq!(parser.convert("oo`"), "ু");
        assert_eq!(parser.convert("koo"), "কু");
        assert_eq!(parser.convert("ooo"), "উঅ");
        assert_eq!(parser.convert("!oo"), "!উ");
        assert_eq!(parser.convert("!ooo"), "!উঅ");
        assert_eq!(parser.convert("aoo"), "আউ");
        assert_eq!(parser.convert("oop"), "উপ");
        assert_eq!(parser.convert("ooo`"), "উ");
        assert_eq!(parser.convert("o`"), "");
        assert_eq!(parser.convert("oZ"), "অ্য");
        assert_eq!(parser.convert("oY"), "অয়");
        assert_eq!(parser.convert("o"), "অ");
        assert_eq!(parser.convert("!o"), "!অ");
        assert_eq!(parser.convert("^o"), "ঁঅ");
        assert_eq!(parser.convert("*o"), "*অ");
        assert_eq!(parser.convert("io"), "ইও");
        assert_eq!(parser.convert("yo"), "ইয়");
        assert_eq!(parser.convert("no"), "ন");
        assert_eq!(parser.convert("tth"), "ত্থ");
        assert_eq!(parser.convert("t``"), "ৎ");
        assert_eq!(parser.convert("`t``"), "ৎ");
        assert_eq!(parser.convert("t``t``"), "ৎৎ");
        assert_eq!(parser.convert("t```"), "ৎ");
        assert_eq!(parser.convert("TT"), "ট্ট");
        assert_eq!(parser.convert("Tm"), "ট্ম");
        assert_eq!(parser.convert("Th"), "ঠ");
        assert_eq!(parser.convert("tn"), "ত্ন");
        assert_eq!(parser.convert("tm"), "ত্ম");
        assert_eq!(parser.convert("th"), "থ");
        assert_eq!(parser.convert("tt"), "ত্ত");
        assert_eq!(parser.convert("T"), "ট");
        assert_eq!(parser.convert("t"), "ত");
        assert_eq!(parser.convert("aZ"), "অ্যা");
        assert_eq!(parser.convert("aaZ"), "আঅ্যা");
        assert_eq!(parser.convert("AZ"), "অ্যা");
        assert_eq!(parser.convert("a`"), "া");
        assert_eq!(parser.convert("a``"), "া");
        assert_eq!(parser.convert("ka`"), "কা");
        assert_eq!(parser.convert("A`"), "া");
        assert_eq!(parser.convert("a"), "আ");
        assert_eq!(parser.convert("`a"), "আ");
        assert_eq!(parser.convert("k`a"), "কআ");
        assert_eq!(parser.convert("ia"), "ইয়া");
        assert_eq!(parser.convert("aaaa`"), "আআআা");
        assert_eq!(parser.convert("i`"), "ি");
        assert_eq!(parser.convert("i"), "ই");
        assert_eq!(parser.convert("`i"), "ই");
        assert_eq!(parser.convert("hi"), "হি");
        assert_eq!(parser.convert("ih"), "ইহ");
        assert_eq!(parser.convert("i`h"), "িহ");
        assert_eq!(parser.convert("I`"), "ী");
        assert_eq!(parser.convert("I"), "ঈ");
        assert_eq!(parser.convert("cI"), "চী");
        assert_eq!(parser.convert("Ix"), "ঈক্স");
        assert_eq!(parser.convert("II"), "ঈঈ");
        assert_eq!(parser.convert("0I"), "০ঈ");
        assert_eq!(parser.convert("oI"), "অঈ");
        assert_eq!(parser.convert("u`"), "ু");
        assert_eq!(parser.convert("u"), "উ");
        assert_eq!(parser.convert("ku"), "কু");
        assert_eq!(parser.convert("uk"), "উক");
        assert_eq!(parser.convert("uu"), "উউ");
        assert_eq!(parser.convert("iu"), "ইউ");
        assert_eq!(parser.convert("&u"), "&উ");
        assert_eq!(parser.convert("u&"), "উ&");
        assert_eq!(parser.convert("U`"), "ূ");
        assert_eq!(parser.convert("U"), "ঊ");
        assert_eq!(parser.convert("yU"), "ইয়ূ");
        assert_eq!(parser.convert("Uy"), "ঊয়");
        assert_eq!(parser.convert("^U"), "ঁঊ");
        assert_eq!(parser.convert("U^"), "ঊঁ");
        assert_eq!(parser.convert("EE"), "ঈ");
        assert_eq!(parser.convert("ee"), "ঈ");
        assert_eq!(parser.convert("Ee"), "ঈ");
        assert_eq!(parser.convert("eE"), "ঈ");
        assert_eq!(parser.convert("ee`"), "ী");
        assert_eq!(parser.convert("kee"), "কী");
        assert_eq!(parser.convert("eek"), "ঈক");
        assert_eq!(parser.convert("0ee"), "০ঈ");
        assert_eq!(parser.convert("ee8"), "ঈ৮");
        assert_eq!(parser.convert("(ee)"), "(ঈ)");
        assert_eq!(parser.convert("e`"), "ে");
        assert_eq!(parser.convert("e"), "এ");
        assert_eq!(parser.convert("ke"), "কে");
        assert_eq!(parser.convert("we"), "ওয়ে");
        assert_eq!(parser.convert("#e#"), "#এ#");
        assert_eq!(parser.convert("`e`"), "ে");
        assert_eq!(parser.convert("z"), "য");
        assert_eq!(parser.convert("Z"), "্য");
        assert_eq!(parser.convert("rZ"), "র‍্য");
        assert_eq!(parser.convert("kZS"), "ক্যশ");
        assert_eq!(parser.convert("y"), "ইয়");
        assert_eq!(parser.convert("oy"), "অয়");
        assert_eq!(parser.convert("ky"), "ক্য");
        assert_eq!(parser.convert("ya"), "ইয়া");
        assert_eq!(parser.convert("yaa"), "ইয়াআ");
        assert_eq!(parser.convert("Y"), "য়");
        assert_eq!(parser.convert("YY"), "য়য়");
        assert_eq!(parser.convert("iY"), "ইয়");
        assert_eq!(parser.convert("kY"), "কয়");
        assert_eq!(parser.convert("q"), "ক");
        assert_eq!(parser.convert("Q"), "ক");
        assert_eq!(parser.convert("w"), "ও");
        assert_eq!(parser.convert("wa"), "ওয়া");
        assert_eq!(parser.convert("-wa-"), "-ওয়া-");
        assert_eq!(parser.convert("woo"), "ওয়ু");
        assert_eq!(parser.convert("wre"), "ওরে");
        assert_eq!(parser.convert("kw"), "ক্ব");
        assert_eq!(parser.convert("x"), "এক্স");
        assert_eq!(parser.convert("ex"), "এক্স");
        assert_eq!(parser.convert("bx"), "বক্স");
        assert_eq!(parser.convert(":`"), ":");
        assert_eq!(parser.convert(":"), "ঃ");
        assert_eq!(parser.convert("^`"), "^");
        assert_eq!(parser.convert("^"), "ঁ");
        assert_eq!(parser.convert("k^"), "কঁ");
        assert_eq!(parser.convert("k^i"), "কঁই");
        assert_eq!(parser.convert("ki^"), "কিঁ");
        assert_eq!(parser.convert(",,"), "্‌");
        assert_eq!(parser.convert(",,,"), "্‌,");
        assert_eq!(parser.convert(",,`,"), "্‌,");
        assert_eq!(parser.convert("`,,"), "্‌");
        assert_eq!(parser.convert(",`,"), ",,");
        assert_eq!(parser.convert("$"), "৳");
        assert_eq!(parser.convert("`"), "");
        assert_eq!(parser.convert("bdh"), "ব্ধ");
    }

    #[test]
    fn test_sentence() {
        let json = serde_json::from_str(include_str!("AvroPhonetic.json")).unwrap();
        let parser = PhoneticParser::new(&json);

        assert_eq!(parser.convert("ami banglay gan gai"),  "আমি বাংলায় গান গাই");
        assert_eq!(parser.convert("amader valObasa hoye gel ghas, kheye gel goru ar diye gelo ba^sh"),  "আমাদের ভালোবাসা হয়ে গেল ঘাস, খেয়ে গেল গরু আর দিয়ে গেল বাঁশ");
    }
}
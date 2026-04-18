pub enum RegExp {
    Start,                        // ^
    End,                          // $
    Any,                          // .
    Chars(String),                // [a-z]
    NotChars(String),             // [^a-z]
    Group(Box<RegExp>),           // (...)
    Or(Box<RegExp>, Box<RegExp>), // a|b
    Count(Box<RegExp>, u32, u32), // a*, b+, c?, d{1,2}
}

#[derive(Debug, PartialEq)]
pub struct Match {
    substring: String,
    captures: Vec<String>,
}

impl RegExp {
    // Parses a usable RegExp object from a raw string
    pub fn parse(expression: &str) -> RegExp {
        return RegExp::Any;
    }

    pub fn apply(self: &RegExp, haystack: &str) -> Option<Match> {
        return None;
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn openjdk_tests() {
        let input = fs::read_to_string("OpenJDK_Regex_TestCases.txt").unwrap();
        let lines: Vec<&str> = input.lines().filter(|l| !l.starts_with("//")).collect();

        for test_case in lines.chunks(4) {
            let needle = test_case[0];
            let haystack = test_case[1];
            let expected = parse_expected(test_case[2]);

            let regex = RegExp::parse(needle);
            let result = regex.apply(haystack);

            assert_eq!(
                result, expected,
                "Needle: {}\nHaystack: {}\nResult: {:?}",
                needle, haystack, expected
            );
        }
    }

    fn parse_expected(expectation: &str) -> Option<Match> {
        if expectation.starts_with("false") {
            return None;
        }

        let regex = RegExp::parse("true (.*) [0-4]( .*)?( .*)?( .*)?( .*)?");
        let Some(result) = regex.apply(expectation) else {
            panic!("Could not parse expectation \"{}\"", expectation);
        };

        return Some(Match {
            substring: result.captures.first().unwrap().to_string(),
            captures: result
                .captures
                .iter()
                .skip(1)
                .map(|m| (&m[1..]).to_string())
                .collect(),
        });
    }
}

#[derive(Debug)]
pub struct RegExp {
    pub atoms: Vec<Atom>,
}

#[derive(Debug, Clone)]
pub enum Atom {
    Start, // ^
    End,   // $
    Any,   // .
    Chars {
        invert_match: bool,
        chars: Vec<Proton>,
    }, // [a-z]
    Group(usize, Vec<Atom>), // (...)
    Or(Vec<Atom>, Vec<Atom>), // a|b
    Count(Box<Atom>, usize, usize), // a*, b+, c?, d{1,2}
}

#[derive(Debug, Copy, Clone)]
pub enum Proton {
    Char(char),
    Range(char, char),
}

struct AtomParseResult {
    atoms: Vec<Atom>,
    taken: usize,
    groups: usize,
}

impl RegExp {
    // Parses a usable RegExp object from a raw string
    pub fn parse(expression: &str) -> Result<RegExp, String> {
        let AtomParseResult { taken, atoms, .. } = Atom::parse(expression, 0)?;

        if taken != expression.len() {
            return Err("Unexpected closing paren".to_string());
        }

        return Ok(RegExp { atoms });
    }
}

impl Atom {
    /// Need a unique index for each group, so pass in starting_group_index
    /// to safely handle recursion.
    fn parse(expression: &str, starting_group_index: usize) -> Result<AtomParseResult, String> {
        let mut atoms = vec![];
        let mut char_indices = expression.char_indices();
        let mut group_index = starting_group_index;

        while let Some((i, c)) = char_indices.next() {
            let atom = match c {
                '^' => Atom::Start,
                '$' => Atom::End,
                '.' => Atom::Any,
                '[' => {
                    let (taken, atom) = Atom::parse_chars(&expression[i + 1..])?;
                    let _ = char_indices.nth(taken);
                    atom
                }
                '(' => {
                    let my_group_index = group_index;

                    let AtomParseResult {
                        atoms,
                        taken,
                        groups,
                    } = Atom::parse(&expression[i + 1..], group_index + 1)?;

                    group_index += groups + 1;

                    // Take closing paren
                    let Some((_, ')')) = char_indices.nth(taken) else {
                        return Err("Missing expected closing paren".to_string());
                    };

                    Atom::Group(my_group_index, atoms)
                }
                ')' => {
                    return Ok(AtomParseResult {
                        atoms,
                        taken: i,
                        groups: group_index - starting_group_index,
                    });
                }
                '|' => {
                    let AtomParseResult {
                        atoms: right_atoms,
                        taken,
                        groups,
                    } = Atom::parse(&expression[i + 1..], group_index)?;

                    return Ok(AtomParseResult {
                        atoms: vec![Atom::Or(atoms, right_atoms)],
                        taken: i + 1 + taken,
                        groups,
                    });
                }

                // Counts
                '*' => {
                    let Some(prev) = atoms.pop() else {
                        return Err("Unexpected * at start of group".to_string());
                    };
                    Atom::Count(Box::new(prev), 0, usize::MAX)
                }
                '+' => {
                    let Some(prev) = atoms.pop() else {
                        return Err("Unexpected * at start of group".to_string());
                    };
                    Atom::Count(Box::new(prev), 1, usize::MAX)
                }
                '?' => {
                    let Some(prev) = atoms.pop() else {
                        return Err("Unexpected * at start of group".to_string());
                    };
                    Atom::Count(Box::new(prev), 0, 1)
                }

                c => Atom::char(c),
            };

            atoms.push(atom);
        }

        Ok(AtomParseResult {
            atoms,
            taken: expression.len(),
            groups: group_index - starting_group_index,
        })
    }

    fn parse_chars(expression: &str) -> Result<(usize, Atom), String> {
        let mut invert_match = false;
        let mut chars = vec![];

        let mut iter = expression.char_indices().peekable();

        if expression.starts_with("^") {
            let _ = iter.next();
            invert_match = true;
        }

        while let Some((i, c)) = iter.next() {
            match c {
                // literal if leading eg []] or [^]xyz]
                ']' if chars.is_empty() => chars.push(Proton::Char(c)),
                ']' => {
                    return Ok((
                        i,
                        Atom::Chars {
                            invert_match,
                            chars,
                        },
                    ));
                }
                // literal if leading or trailing eg [^-123] or [a-b-]
                '-' if chars.is_empty() | iter.peek().is_some_and(|(_, n)| *n == ']') => {
                    chars.push(Proton::Char(c))
                }
                '-' => {
                    let Some(Proton::Char(prev)) = chars.pop() else {
                        return Err("Malformed range in bracket expression.".to_string());
                    };

                    let Some((_, next)) = iter.next() else {
                        return Err("Missing closing ]".to_string());
                    };

                    chars.push(Proton::Range(prev, next));
                }
                _ => chars.push(Proton::Char(c)),
            }
        }

        Err("Missing closing ]".to_string())
    }

    fn char(c: char) -> Atom {
        Atom::Chars {
            invert_match: false,
            chars: vec![Proton::Char(c)],
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::evaluate::Match;

    #[test]
    fn openjdk_tests() {
        let input = fs::read_to_string("OpenJDK_Regex_TestCases.txt").unwrap();
        let lines: Vec<&str> = input.lines().filter(|l| !l.starts_with("//")).collect();

        for (i, test_case) in lines.chunks(4).enumerate() {
            let needle = test_case[0];
            let haystack = test_case[1];
            let expected = parse_expected(test_case[2]);

            let regex = RegExp::parse(needle).unwrap();

            println!("REGEX: {:?}", regex);

            let result = regex.execute(haystack);

            assert_eq!(
                result, expected,
                "\nNeedle: {}\nHaystack: {}",
                needle, haystack
            );

            println!("PASSED #{}! \n\t- {:?}\n\t- {:?}", i, result, expected);
        }
    }

    fn parse_expected(expectation: &str) -> Option<Match> {
        if expectation.starts_with("false") {
            return None;
        }

        // TODO: fix char ranges and revert this
        let regex = RegExp::parse("true (.*) [0-4]( [^ ]*)?( [^ ]*)?( [^ ]*)?( [^ ]*)?").unwrap();

        let Some(parsed) = regex.execute(expectation) else {
            panic!("Could not parse test case {}", expectation);
        };

        println!("PARSED: {:?}", parsed);

        Some(Match {
            substring: parsed.captures[0].clone().unwrap(),
            captures: parsed.captures[1..]
                .iter()
                .map(|c| {
                    if let Some(val) = c
                        && val == ""
                    {
                        None
                    } else if let Some(val) = c {
                        Some(val[1..].to_string())
                    } else {
                        None
                    }
                })
                .collect(),
        })
    }
}

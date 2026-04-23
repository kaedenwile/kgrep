use crate::regex::{Atom, Proton, RegExp};

#[derive(Debug, PartialEq)]
pub struct Match {
    pub substring: String,
    pub captures: Vec<Option<String>>,
}

#[derive(Debug, Clone)]
struct MatchNode {
    text: String,
    tail: Option<Box<MatchNode>>,
    captures: Vec<(usize, String)>,
}

impl RegExp {
    pub fn execute(self: &RegExp, haystack: &str) -> Option<Match> {
        for i in 0..haystack.len() {
            if let Some(tail) = Atom::match_atoms(&self.atoms, &haystack, i) {
                let mut substring = String::new();
                let mut captures = vec![];

                let mut current = Some(Box::new(tail));
                while let Some(next) = current.take() {
                    // println!("NODE: {:?}", next.text);

                    substring.push_str(&next.text);

                    for (cap_idx, cap_val) in next.captures {
                        println!("FOUND CAPTURE: {} {}", cap_idx, cap_val);
                        while cap_idx >= captures.len() {
                            captures.push(None)
                        }
                        captures[cap_idx] = Some(cap_val);
                    }

                    current = next.tail;
                }

                return Some(Match {
                    substring,
                    captures,
                });
            }
        }

        None
    }
}

impl Atom {
    fn match_atoms(atoms: &[Atom], haystack: &str, cursor: usize) -> Option<MatchNode> {
        use Atom::*;

        let Some(atom) = atoms.get(0) else {
            return Some(MatchNode::empty());
        };

        // println!("Atom: {:?}", atom);

        match atom {
            Start => {
                if cursor == 0
                    && let Some(tail) = Atom::match_atoms(&atoms[1..], haystack, cursor)
                {
                    Some(MatchNode {
                        text: "".to_string(),
                        tail: Some(Box::new(tail)),
                        captures: vec![],
                    })
                } else {
                    None
                }
            }
            End => {
                if cursor == haystack.len()
                    && let Some(tail) = Atom::match_atoms(&atoms[1..], haystack, cursor)
                {
                    Some(MatchNode {
                        text: "".to_string(),
                        tail: Some(Box::new(tail)),
                        captures: vec![],
                    })
                } else {
                    None
                }
            }
            Any => {
                if let Some(c) = haystack.chars().nth(cursor)
                    && let Some(tail) = Atom::match_atoms(&atoms[1..], haystack, cursor + 1)
                {
                    Some(MatchNode {
                        text: c.to_string(),
                        tail: Some(Box::new(tail)),
                        captures: vec![],
                    })
                } else {
                    None
                }
            }
            Chars {
                invert_match,
                chars,
            } => {
                if let Some(next) = haystack.chars().nth(cursor) {
                    let mut matches = false;

                    for proton in chars {
                        match *proton {
                            Proton::Char(c) if next == c => {
                                matches = true;
                                break;
                            }
                            Proton::Range(a, b) if a <= next && next <= b => {
                                matches = true;
                                break;
                            }
                            _ => {}
                        }
                    }

                    // println!("Matches={matches} invert_match={invert_match}");

                    if matches != *invert_match
                        && let Some(tail) = Atom::match_atoms(&atoms[1..], haystack, cursor + 1)
                    {
                        Some(MatchNode {
                            text: next.to_string(),
                            tail: Some(Box::new(tail)),
                            captures: vec![],
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            Group(group_index, group_atoms) => {
                let all_atoms = [&group_atoms, &atoms[1..]].concat();

                if let Some(tail) = Atom::match_atoms(&all_atoms, haystack, cursor) {
                    let mut capture_text = String::new();
                    let mut captures = vec![(0, String::new())];

                    let mut current = tail;
                    for _ in 0..group_atoms.len() {
                        capture_text.push_str(&current.text);
                        captures.extend(current.captures);
                        current = *current.tail.unwrap();
                    }

                    captures[0] = (*group_index, capture_text.clone());

                    Some(MatchNode {
                        text: capture_text,
                        tail: Some(Box::new(current)),
                        captures,
                    })
                } else {
                    None
                }
            }
            Or(l, r) => {
                let all_left_atoms = [&l, &atoms[1..]].concat();
                let all_right_atoms = [&r, &atoms[1..]].concat();

                if let Some(tail) = Atom::match_atoms(&all_left_atoms, haystack, cursor) {
                    let mut capture_text = String::new();
                    let mut captures = vec![];

                    let mut current = tail;
                    for _ in 0..all_left_atoms.len() {
                        capture_text.push_str(&current.text);
                        captures.extend(current.captures);
                        current = *current.tail.unwrap();
                    }

                    Some(MatchNode {
                        text: capture_text,
                        tail: Some(Box::new(current)),
                        captures,
                    })
                } else if let Some(tail) = Atom::match_atoms(&all_right_atoms, haystack, cursor) {
                    let mut capture_text = String::new();
                    let mut captures = vec![];

                    let mut current = tail;
                    for _ in 0..all_left_atoms.len() {
                        capture_text.push_str(&current.text);
                        captures.extend(current.captures);
                        current = *current.tail.unwrap();
                    }

                    Some(MatchNode {
                        text: capture_text,
                        tail: Some(Box::new(current)),
                        captures,
                    })
                } else {
                    None
                }
            }
            Count(boxed, min, max) => {
                let atom = boxed.as_ref();
                let mut count_atoms = vec![atom.clone(); *min];

                while count_atoms.len() < *max
                    && let Some(_) = Atom::match_atoms(&count_atoms, haystack, cursor)
                {
                    count_atoms.push(atom.clone());
                }

                while count_atoms.len() >= *min {
                    let all_count_atoms = [&count_atoms, &atoms[1..]].concat();

                    if let Some(tail) = Atom::match_atoms(&all_count_atoms, haystack, cursor) {
                        let mut capture_text = String::new();
                        let mut captures = vec![];

                        let mut current = tail;
                        for _ in 0..count_atoms.len() {
                            capture_text.push_str(&current.text);
                            captures.extend(current.captures);
                            current = *current.tail.unwrap();
                        }

                        return Some(MatchNode {
                            text: capture_text,
                            tail: Some(Box::new(current)),
                            captures,
                        });
                    } else if !count_atoms.is_empty() {
                        count_atoms.pop();
                        // println!("Decrementing to: {}", count_atoms.len());
                    } else {
                        return None;
                    }
                }

                None
            }
        }
    }
}

impl MatchNode {
    fn empty() -> MatchNode {
        MatchNode {
            text: "".to_string(),
            tail: None,
            captures: vec![],
        }
    }
}

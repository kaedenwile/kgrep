use crate::regex::{Atom, Proton, RegExp};

#[derive(Debug, PartialEq)]
pub struct Match {
    pub substring: String,
    pub captures: Vec<String>,
}

struct MatchNode {
    text: String,
    tail: Option<Box<MatchNode>>,
    capture: Option<String>,
}

impl RegExp {
    pub fn execute(self: &RegExp, haystack: &str) -> Option<Match> {
        // println!("REGEX MATCHING HAYSTACK={haystack}");

        if let Some(tail) = Atom::match_atoms(&self.atoms, haystack, 0) {
            let mut substring = tail.text.clone();
            let mut captures = vec![];

            if let Some(capture) = &tail.capture {
                captures.push(capture.clone());
            }

            let mut current = tail;
            while let Some(next) = current.tail {
                substring.push_str(&next.text);

                if let Some(capture) = &next.capture {
                    captures.push(capture.clone());
                }

                current = *next;
            }

            Some(Match {
                substring,
                captures,
            })
        } else {
            None
        }
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
                        capture: None,
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
                        capture: None,
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
                        capture: None,
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
                            Proton::Range(a, b) if a > next && next > b => {
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
                            capture: None,
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            Group(group_atoms) => {
                let all_atoms = [&group_atoms, &atoms[1..]].concat();

                if let Some(tail) = Atom::match_atoms(&all_atoms, haystack, cursor) {
                    let mut capture_text = String::new();

                    let mut current = &tail;
                    for _ in 0..group_atoms.len() {
                        capture_text.push_str(&current.text);
                        current = current.tail.as_ref().unwrap().as_ref();
                    }

                    Some(MatchNode {
                        text: capture_text.clone(),
                        tail: Some(Box::new(tail)),
                        capture: Some(capture_text),
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

                    let mut current = &tail;
                    for _ in 0..all_left_atoms.len() {
                        capture_text.push_str(&current.text);
                        current = current.tail.as_ref().unwrap().as_ref();
                    }

                    Some(MatchNode {
                        text: capture_text,
                        tail: Some(Box::new(tail)),
                        capture: None,
                    })
                } else if let Some(tail) = Atom::match_atoms(&all_right_atoms, haystack, cursor) {
                    let mut capture_text = String::new();

                    let mut current = &tail;
                    for _ in 0..all_left_atoms.len() {
                        capture_text.push_str(&current.text);
                        current = current.tail.as_ref().unwrap().as_ref();
                    }

                    Some(MatchNode {
                        text: capture_text,
                        tail: Some(Box::new(tail)),
                        capture: None,
                    })
                } else {
                    None
                }
            }
            Count(boxed, min, max) => {
                let atom = boxed.as_ref();
                let mut count_atoms = vec![atom.clone(); *min];

                while count_atoms.len() <= *max
                    && let Some(_) = Atom::match_atoms(&count_atoms, haystack, cursor)
                {
                    count_atoms.push(atom.clone());
                }

                // println!("Found: {}", count_atoms.len());

                while count_atoms.len() >= *min {
                    let all_count_atoms = [&count_atoms, &atoms[1..]].concat();

                    if let Some(tail) = Atom::match_atoms(&all_count_atoms, haystack, cursor) {
                        let mut capture_text = String::new();

                        let mut current = &tail;
                        for _ in 0..count_atoms.len() {
                            capture_text.push_str(&current.text);
                            current = current.tail.as_ref().unwrap().as_ref();
                        }

                        return Some(MatchNode {
                            text: capture_text,
                            tail: Some(Box::new(tail)),
                            capture: None,
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
            capture: None,
        }
    }

    // fn test(condition: bool) -> Option<MatchNode> {
    //     if condition {
    //         Some(MatchNode::simple("".to_string()))
    //     } else {
    //         None
    //     }
    // }
    //
    // fn simple(text: String) -> MatchNode {
    //     MatchNode {
    //         // has_less_greedy_match: false,
    //         text,
    //         captures: vec![],
    //         subframes: vec![],
    //     }
    // }
    //
    // fn group(match_nodes: Vec<MatchNode>, is_capture_group: bool) -> MatchNode {
    //     let mut text = String::new();
    //
    //     for node in &match_nodes {
    //         text.push_str(node.text.as_str());
    //     }
    //
    //     MatchNode {
    //         text,
    //         is_capture_group,
    //         alternate: Alternate::None,
    //         sub: Some(match_nodes),
    //     }
    // }
}

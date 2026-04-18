pub struct RegExp {}

pub struct Match {}

impl RegExp {
    // Parses a usable RegExp object from a raw string
    pub fn parse(expression: String) -> RegExp {
        unimplemented!();
    }

    pub fn apply(self: &RegExp, haystack: String) -> Option<Match> {
        unimplemented!();
    }
}

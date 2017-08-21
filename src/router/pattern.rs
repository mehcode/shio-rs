use std::ops::Deref;
use std::str::FromStr;

use hyper::Uri;
use regex::Regex;

// TODO: Implement a formal route parser to handle `:param` and `*` segments.

pub struct Pattern(Regex);

impl Deref for Pattern {
    type Target = Regex;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for Pattern {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut pattern = String::from("^");

        if !s.is_empty() && s != "/" {
            let mut s = String::from(s);

            if !s.starts_with('/') {
                s.insert(0, '/');
            }

            pattern.push_str(Uri::from_str(&s).unwrap().path());
        }

        pattern.push_str("/?$");

        Ok(Pattern(Regex::new(&pattern).unwrap()))
    }
}

impl<'a> From<&'a str> for Pattern {
    fn from(val: &'a str) -> Self {
        val.parse().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::Pattern;

    #[test]
    fn test_parse() {
        assert_eq!(Pattern::from("").as_str(), "^/?$");
        assert_eq!(Pattern::from("/").as_str(), "^/?$");
        assert_eq!(Pattern::from("users").as_str(), "^/users/?$");
        assert_eq!(Pattern::from("/users").as_str(), "^/users/?$");
    }
}

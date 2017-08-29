use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;
use std::collections::HashMap;

use regex::{Error as RegexError, Regex};

use super::Parameters;

pub struct Pattern {
    re: Regex,
    names: Arc<HashMap<String, usize>>,
}

impl Pattern {
    pub(crate) fn new(re: Regex) -> Self {
        let names = re.capture_names()
            .enumerate()
            .filter_map(|(i, name)| name.map(|name| (name.to_owned(), i)))
            .collect();

        Pattern {
            re,
            names: Arc::new(names),
        }
    }

    pub(crate) fn parameters(&self, text: &str) -> Option<Parameters> {
        let captures = match self.re.captures(text) {
            Some(captures) => captures,
            None => return None,
        };

        Some(Parameters::new(self.names.clone(), text, captures))
    }
}

impl Deref for Pattern {
    type Target = Regex;

    fn deref(&self) -> &Self::Target {
        &self.re
    }
}

impl FromStr for Pattern {
    type Err = RegexError;

    fn from_str(pattern: &str) -> Result<Self, Self::Err> {
        Ok(Pattern::new(Regex::new(&parse(pattern))?))
    }
}

impl<'a> From<&'a str> for Pattern {
    fn from(val: &'a str) -> Self {
        // FIXME: What should we do here? I think it is a good idea to
        //        crash on boot if your routes are invalid.. but is `.unwrap` here
        //        the best way to do that?
        val.parse().unwrap()
    }
}

impl From<Regex> for Pattern {
    fn from(val: Regex) -> Self {
        Pattern::new(val)
    }
}

fn parse(pattern: &str) -> String {
    let mut re = String::from("^/");
    let mut in_param = false;
    let mut param_name = String::new();
    let mut params = Vec::new();

    for (index, ch) in pattern.chars().enumerate() {
        // All routes must have a leading slash so its optional to have one
        if index == 0 && ch == '/' {
            continue;
        }

        if in_param {
            // In parameter segment: `{....}`
            if ch == '}' {
                // Exit the parameter segment
                re.push_str(&format!(r"(?P<{}>[^/]+)", &param_name));
                params.push(param_name.clone());
                in_param = false;
            } else {
                param_name.push(ch);
            }
        } else if ch == '{' {
            // Enter a parameter segment
            in_param = true;
            param_name.clear();
        } else {
            re.push(ch);
        }
    }

    re.push('$');
    re
}

#[cfg(test)]
mod tests {
    use regex::Regex;
    use super::parse;

    fn assert_parse(pattern: &str, expected_re: &str) -> Regex {
        let re_str = parse(pattern);
        assert_eq!(&*re_str, expected_re);

        let re = Regex::new(&re_str);
        println!("{:?}", re);
        assert!(re.is_ok());

        re.unwrap()
    }

    #[test]
    fn test_parse_static() {
        let re = assert_parse("/", r"^/$");
        assert!(re.is_match("/"));
        assert!(!re.is_match("/a"));

        let re = assert_parse("/user", r"^/user$");
        assert!(re.is_match("/user"));
        assert!(!re.is_match("/user1"));
        assert!(!re.is_match("/user/"));

        let re = assert_parse("/user/", r"^/user/$");
        assert!(re.is_match("/user/"));
        assert!(!re.is_match("/user"));
        assert!(!re.is_match("/user/gs"));

        let re = assert_parse("/user/profile", r"^/user/profile$");
        assert!(re.is_match("/user/profile"));
        assert!(!re.is_match("/user/profile/profile"));
    }

    #[test]
    fn test_parse_param() {
        let re = assert_parse("/user/{id}", r"^/user/(?P<id>[^/]+)$");
        assert!(re.is_match("/user/profile"));
        assert!(re.is_match("/user/2345"));
        assert!(!re.is_match("/user/2345/"));
        assert!(!re.is_match("/user/2345/sdg"));

        let captures = re.captures("/user/profile").unwrap();
        assert_eq!(captures.get(1).unwrap().as_str(), "profile");
        assert_eq!(captures.name("id").unwrap().as_str(), "profile");

        let captures = re.captures("/user/1245125").unwrap();
        assert_eq!(captures.get(1).unwrap().as_str(), "1245125");
        assert_eq!(captures.name("id").unwrap().as_str(), "1245125");

        let re = assert_parse(
            "/v{version}/resource/{id}",
            r"^/v(?P<version>[^/]+)/resource/(?P<id>[^/]+)$",
        );
        assert!(re.is_match("/v1/resource/320120"));
        assert!(!re.is_match("/v/resource/1"));
        assert!(!re.is_match("/resource"));

        let captures = re.captures("/v151/resource/adahg32").unwrap();
        assert_eq!(captures.get(1).unwrap().as_str(), "151");
        assert_eq!(captures.name("version").unwrap().as_str(), "151");
        assert_eq!(captures.name("id").unwrap().as_str(), "adahg32");
    }
}

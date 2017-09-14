use std::ops::Index;
use std::sync::Arc;
use std::collections::HashMap;

use util::typemap::Key;
use regex::Captures;

pub struct Parameters {
    text: String,
    matches: Vec<Option<(usize, usize)>>,
    names: Arc<HashMap<String, usize>>,
}

impl Parameters {
    pub(crate) fn new(names: Arc<HashMap<String, usize>>, text: &str, captures: Captures) -> Self {
        Self {
            names,
            text: text.into(),
            matches: captures
                .iter()
                .map(|capture| capture.map(|m| (m.start(), m.end())))
                .collect(),
        }
    }

    pub fn get(&self, index: usize) -> Option<&str> {
        self.matches
            // +1 is added as matches start at 1 in regex (with 0 referring to the
            //  whole matched text)
            .get(index + 1)
            .and_then(|m| m.map(|(start, end)| &self.text[start..end]))
    }

    pub fn name(&self, name: &str) -> Option<&str> {
        self.names.get(name).and_then(|&i| self.get(i - 1))
    }
}

impl Key for Parameters {
    type Value = Self;
}

impl Index<usize> for Parameters {
    type Output = str;

    /// Get a parameter by index.
    ///
    /// # Panics
    ///
    /// If there is no parameter at the given index.
    fn index(&self, index: usize) -> &Self::Output {
        self.get(index)
            .unwrap_or_else(|| panic!("no parameter at index '{}'", index))
    }
}

impl<'b> Index<&'b str> for Parameters {
    type Output = str;

    /// Get a parameter by name.
    ///
    /// # Panics
    ///
    /// If there is no parameter named by the given value.
    fn index<'a>(&'a self, name: &str) -> &'a str {
        self.name(name)
            .unwrap_or_else(|| panic!("no parameter named '{}'", name))
    }
}

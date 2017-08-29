use std::sync::Arc;
use std::collections::HashMap;

use typemap::Key;
use regex::Captures;

pub struct Parameters {
    text: String,
    matches: Vec<Option<(usize, usize)>>,
    names: Arc<HashMap<String, usize>>,
}

impl Parameters {
    pub(crate) fn new(names: Arc<HashMap<String, usize>>, text: &str, captures: Captures) -> Self {
        Parameters {
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
            .get(index)
            .and_then(|m| m.map(|(start, end)| &self.text[start..end]))
    }

    pub fn name(&self, name: &str) -> Option<&str> {
        self.names.get(name).and_then(|&i| self.get(i))
    }
}

impl Key for Parameters {
    type Value = Self;
}

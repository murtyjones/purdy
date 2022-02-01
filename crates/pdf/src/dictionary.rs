use crate::{
    error::HandlingError,
    object::{Name, Object},
};
use anyhow::Result;
use linked_hash_map::{Entries, LinkedHashMap};
use std::fmt;

/// Dictionary object.
#[derive(PartialEq, Clone)]
pub struct Dictionary<'a>(LinkedHashMap<Name, Object<'a>>);

impl<'a> Default for Dictionary<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Dictionary<'a> {
    pub fn new() -> Dictionary<'a> {
        Dictionary(LinkedHashMap::new())
    }

    pub fn insert(&mut self, key: Name, value: Object<'a>) -> Option<Object<'a>> {
        self.0.insert(key, value)
    }

    pub fn entries(&mut self) -> Entries<Name, Object<'a>> {
        self.0.entries()
    }

    pub fn get(&self, key: &[u8]) -> Result<&Object<'a>> {
        self.0
            .get(key)
            .ok_or_else(|| HandlingError::ObjectNotFound.into())
    }

    pub fn iter(&self) -> linked_hash_map::Iter<'_, Vec<u8>, Object<'a>> {
        self.0.iter()
    }
}

impl<'a> Dictionary<'a> {
    pub(crate) fn debug_dict(&self) -> String {
        let mut result = String::from('{');
        self.iter().for_each(|(name, obj)| {
            result.push(' ');

            result.push_str(&format!("/{}", std::str::from_utf8(name).unwrap()));
            result.push_str(" => ");
            result.push_str(&obj.debug_object());
            result.push(',');
        });
        result.push_str(" }");
        result
    }

    pub(crate) fn debug_dict_pretty(&self, indent_level: u8) -> String {
        let units = 4;
        let mut indent = String::new();
        for _ in 1..=(indent_level * units) {
            indent.push(' ');
        }
        let mut indent_minus_one = String::new();
        for _ in 1..=(indent_level.saturating_sub(1) * units) {
            indent_minus_one.push(' ');
        }
        let mut result = String::from("{\n");
        self.iter().for_each(|(name, obj)| {
            result.push_str(&indent);
            result.push_str(&format!("/{}", std::str::from_utf8(name).unwrap()));
            result.push_str(" => ");
            result.push_str(&obj.debug_object_pretty(indent_level + 1));
            result.push('\n');
        });
        result.push_str(&indent_minus_one);
        result.push('}');
        result
    }
}

impl<'a> fmt::Debug for Dictionary<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "{}", self.debug_dict_pretty(1))
        } else {
            write!(f, "{}", self.debug_dict())
        }
    }
}

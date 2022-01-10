use anyhow::Result;
use shared::NumberError;
use std::{fmt, str};
use strum_macros::Display;
use num::ToPrimitive;

use linked_hash_map::LinkedHashMap;

mod invariants;

use crate::{dictionary::Dictionary, error::HandlingError, stream::Stream, ObjectId};

pub type Name = Vec<u8>;

/// Basic PDF object types defined in an enum.
#[derive(PartialEq, Clone, Display)]
pub enum Object<'a> {
    Null,
    Boolean(bool),
    Integer(i64),
    Real(f64),
    Name(Name),
    String(Vec<u8>, StringFormat),
    Array(Vec<Object<'a>>),
    Dictionary(Dictionary<'a>),
    Stream(Stream<'a>),
    Reference(ObjectId),
}

/// String objects can be written in two formats.
#[derive(PartialEq, Debug, Clone)]
pub enum StringFormat {
    Literal,
    Hexadecimal,
}

impl<'a> Object<'a> {
    pub fn string_literal<S: Into<Vec<u8>>>(s: S) -> Self {
        Object::String(s.into(), StringFormat::Literal)
    }

    pub fn is_null(&self) -> bool {
        matches!(*self, Object::Null)
    }

    pub fn as_bool(&self) -> Result<bool> {
        match *self {
            Object::Boolean(ref value) => Ok(*value),
            _ => Err(HandlingError::ObjectCast.into()),
        }
    }

    pub fn as_i64(&self) -> Result<i64> {
        match *self {
            Object::Integer(ref value) => Ok(*value),
            _ => Err(HandlingError::ObjectCast.into()),
        }
    }

    pub fn as_f64(&self) -> Result<f64> {
        match *self {
            Object::Real(ref value) => Ok(*value),
            _ => Err(HandlingError::ObjectCast.into()),
        }
    }

    /// Get the object value as a float.
    /// Unlike as_f64() this will also cast an Integer to a Real.
    pub fn as_float(&self) -> Result<f64> {
        match *self {
            Object::Integer(ref value) => value.to_f64().ok_or(NumberError::InvalidNumberConversion.into()),
            Object::Real(ref value) => Ok(*value),
            _ => Err(HandlingError::ObjectCast.into()),
        }
    }

    pub fn as_name(&self) -> Result<&[u8]> {
        match *self {
            Object::Name(ref name) => Ok(name),
            _ => Err(HandlingError::ObjectCast.into()),
        }
    }

    pub fn as_name_str(&self) -> Result<&str> {
        Ok(std::str::from_utf8(self.as_name()?)?)
    }

    pub fn as_str(&self) -> Result<&[u8]> {
        match self {
            Object::String(string, _) => Ok(string),
            _ => Err(HandlingError::ObjectCast.into()),
        }
    }

    pub fn as_str_mut(&mut self) -> Result<&mut Vec<u8>> {
        match self {
            Object::String(string, _) => Ok(string),
            _ => Err(HandlingError::ObjectCast.into()),
        }
    }

    pub fn as_reference(&self) -> Result<ObjectId> {
        match *self {
            Object::Reference(ref id) => Ok(*id),
            _ => Err(HandlingError::ObjectCast.into()),
        }
    }

    pub fn as_array(&self) -> Result<&Vec<Object<'a>>> {
        match *self {
            Object::Array(ref arr) => Ok(arr),
            _ => Err(HandlingError::ObjectCast.into()),
        }
    }

    pub fn as_array_mut(&mut self) -> Result<&mut Vec<Object<'a>>> {
        match *self {
            Object::Array(ref mut arr) => Ok(arr),
            _ => Err(HandlingError::ObjectCast.into()),
        }
    }

    pub fn as_dict(&self) -> Result<&Dictionary<'a>> {
        match *self {
            Object::Dictionary(ref dict) => Ok(dict),
            _ => Err(HandlingError::ObjectCast.into()),
        }
    }

    pub fn as_dict_mut(&mut self) -> Result<&mut Dictionary<'a>> {
        match *self {
            Object::Dictionary(ref mut dict) => Ok(dict),
            _ => Err(HandlingError::ObjectCast.into()),
        }
    }

    pub fn as_stream(&self) -> Result<&Stream<'a>> {
        match *self {
            Object::Stream(ref stream) => Ok(stream),
            _ => Err(HandlingError::ObjectCast.into()),
        }
    }

    pub fn as_stream_mut(&mut self) -> Result<&mut Stream<'a>> {
        match *self {
            Object::Stream(ref mut stream) => Ok(stream),
            _ => Err(HandlingError::ObjectCast.into()),
        }
    }
}

impl<'a> Object<'a> {
    pub(crate) fn debug_object_pretty(&self, indent_level: u8) -> String {
        let units = 4;
        let mut indent = String::new();
        for _ in 1..=(indent_level * units) {
            indent.push_str(" ");
        }
        let mut indent_minus_one = String::new();
        for _ in 1..=(indent_level.checked_sub(1).unwrap_or(0) * units) {
            indent_minus_one.push_str(" ");
        }
        match self {
            Object::Array(s) => {
                let mut result = String::from("[\n");
                result.push_str(
                    &s.iter()
                        .map(|each| {
                            let mut object = String::from(&indent);
                            object.push_str(&each.debug_object_pretty(indent_level + 1));
                            object
                        })
                        .collect::<Vec<String>>()
                        .join(",\n"),
                );
                result.push_str("\n");
                result.push_str(&indent_minus_one);
                result.push_str("]");
                result
            }
            Object::Real(n) => n.to_string(),
            Object::Integer(n) => n.to_string(),
            Object::Null => "<null>".to_string(),
            Object::Name(n) => {
                let mut result = String::from("/");
                result.push_str(str::from_utf8(&n).unwrap());
                result
            }
            Object::Boolean(b) => b.to_string(),
            Object::Reference(r) => format!("({}, {})", r.0, r.1),
            Object::String(s, _f) => {
                let mut result = String::from("\"");
                result.push_str(str::from_utf8(&s).unwrap());
                result.push('"');
                result
            }
            Object::Dictionary(d) => d.debug_dict_pretty(indent_level),
            Object::Stream(s) => {
                let mut result = String::from("Stream => {\n");
                result.push_str(&indent);
                result.push_str("Dict => ");
                result.push_str(&s.dict.debug_dict_pretty(indent_level + 1));
                result.push_str("\n");
                result.push_str(&indent);
                result.push_str("Bytes (first 50) => ");
                result.push_str(
                    &String::from_utf8_lossy(&s.content[0..(std::cmp::min(100, s.content.len()))])
                        .to_string(),
                );
                result.push_str("\n");
                result.push_str(&indent_minus_one);
                result.push_str("}");
                result
            }
        }
    }

    pub(crate) fn debug_object(&self) -> String {
        match self {
            Object::Array(s) => {
                let mut result = String::from("[ ");
                result.push_str(
                    &s.iter()
                        .map(|each| {
                            let mut object = String::new();
                            object.push_str(&each.debug_object());
                            object
                        })
                        .collect::<Vec<String>>()
                        .join(", "),
                );
                result.push_str(" ]");
                result
            }
            Object::Real(n) => n.to_string(),
            Object::Integer(n) => n.to_string(),
            Object::Null => "<null>".to_string(),
            Object::Name(n) => {
                let mut result = String::from("/");
                result.push_str(str::from_utf8(&n).unwrap());
                result
            }
            Object::Boolean(b) => b.to_string(),
            Object::Reference(r) => format!("({}, {})", r.0, r.1),
            Object::String(s, _f) => {
                let mut result = String::from("\"");
                result.push_str(str::from_utf8(&s).unwrap());
                result.push('"');
                result
            }
            Object::Dictionary(d) => format!("{:?}", d),
            Object::Stream(s) => {
                let mut result = String::from("Stream => { Dict => ");
                result.push_str(&s.dict.debug_dict());
                result.push_str(", Bytes (first 50) => ");
                result.push_str(
                    &String::from_utf8_lossy(&s.content[0..(std::cmp::min(100, s.content.len()))])
                        .to_string(),
                );
                result.push_str(" }");
                result
            }
        }
    }
}

impl<'a> fmt::Debug for Object<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if f.alternate() {
            write!(f, "{}", self.debug_object_pretty(1))
        } else {
            write!(f, "{}", self.debug_object())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        array, dict,
        dictionary::Dictionary,
        dictionary_struct, int, name,
        object::{Object, Stream, StringFormat},
        real, reference,
    };
    #[test]
    fn test_debug_pretty() {
        let obj = Object::Array(vec![
            Object::Real(1.23),
            Object::Integer(20),
            Object::Name(b"lime Green".to_vec()),
            Object::Boolean(true),
            Object::Null,
            Object::Reference((78, 40)),
            Object::String(b"Hello".to_vec(), StringFormat::Hexadecimal),
            Object::String(b"There".to_vec(), StringFormat::Literal),
            dict!(dictionary_struct! {
                "Type" => name!("Page"),
                "Parent" => reference!(3, 0),
                "Resources" => dict!(dictionary_struct! {
                    "Font" => dict!(dictionary_struct! {
                        "F1" => reference!(9, 0)
                    }),
                    "ProcSet" => reference!(8, 0),
                }),
                "MediaBox" => array!(
                    int!(0),
                    int!(0),
                    real!(612.0000),
                    real!(792.0000),
                ),
                "Contents" => reference!(5, 0),
            }),
            Object::Stream(Stream {
                dict: dictionary_struct! {
                    "Type" => name!("Stream"),
                },
                content: &[78, 98, 225],
                allows_compression: false,
                start_position: None,
            }),
        ]);
        assert_eq!(
            format!("{:#?}", obj),
            "[
    1.23,
    20,
    /lime Green,
    true,
    <null>,
    (78, 40),
    \"Hello\",
    \"There\",
    {
        /Type => /Page
        /Parent => (3, 0)
        /Resources => {
            /Font => {
                /F1 => (9, 0)
            }
            /ProcSet => (8, 0)
        }
        /MediaBox => [
            0,
            0,
            612,
            792
        ]
        /Contents => (5, 0)
    },
    Stream => {
        Dict => {
            /Type => /Stream
        }
        Bytes (first 50) => Nb�
    }
]"
        );
    }

    #[test]
    fn test_debug() {
        let obj = Object::Array(vec![
            Object::Real(1.23),
            Object::Integer(20),
            Object::Name(b"lime Green".to_vec()),
            Object::Boolean(true),
            Object::Null,
            Object::Reference((78, 40)),
            Object::String(b"Hello".to_vec(), StringFormat::Hexadecimal),
            Object::String(b"There".to_vec(), StringFormat::Literal),
            dict!(dictionary_struct! {
                "Type" => name!("Page"),
                "Parent" => reference!(3, 0),
                "Resources" => dict!(dictionary_struct! {
                    "Font" => dict!(dictionary_struct! {
                        "F1" => reference!(9, 0)
                    }),
                    "ProcSet" => reference!(8, 0),
                }),
                "MediaBox" => array!(
                    int!(0),
                    int!(0),
                    real!(612.0000),
                    real!(792.0000),
                ),
                "Contents" => reference!(5, 0),
            }),
            Object::Stream(Stream {
                dict: dictionary_struct! {
                    "Type" => name!("Stream"),
                },
                content: &[78, 98, 225],
                allows_compression: false,
                start_position: None,
            }),
        ]);
        assert_eq!(format!("{:?}", obj), "[ 1.23, 20, /lime Green, true, <null>, (78, 40), \"Hello\", \"There\", { /Type => /Page, /Parent => (3, 0), /Resources => { /Font => { /F1 => (9, 0), }, /ProcSet => (8, 0), }, /MediaBox => [ 0, 0, 612, 792 ], /Contents => (5, 0), }, Stream => { Dict => { /Type => /Stream, }, Bytes (first 50) => Nb� } ]");
    }
}

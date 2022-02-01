use crate::dictionary::Dictionary;
use crate::object::{Object, StringFormat};
use crate::xref::{Xref, XrefEntry};

#[macro_export]
macro_rules! null {
    ($input:expr) => {
        Object::Null
    };
}

#[macro_export]
macro_rules! bool {
    ($input:expr) => {
        Object::Boolean($input)
    };
}

#[macro_export]
macro_rules! int {
    ($input:expr) => {
        Object::Integer($input)
    };
}

#[macro_export]
macro_rules! real {
    ($input:expr) => {
        Object::Real($input)
    };
}

#[macro_export]
macro_rules! name {
    ($input:tt) => {
        Object::Name($input.as_bytes().to_vec())
    };
}

#[macro_export]
macro_rules! string {
    ($one:expr, $two:expr) => {
        Object::String($one, $two)
    };
}

#[macro_export]
macro_rules! array {
    () => (
        Object::Array(std::vec::Vec::new())
    );
    ($elem:expr; $n:expr) => (
        Object::Array(std::vec::from_elem($elem, $n))
    );
    ($($x:expr),+ $(,)?) => {
		{
			let mut arr = Vec::new();
				$(
					let _ = arr.push($x);
				)*
			Object::Array(arr)
		}
	};
}

#[macro_export]
macro_rules! dict {
    ($input:expr) => {
        Object::Dictionary($input)
    };
}

#[macro_export]
macro_rules! stream {
    ($input:expr) => {
        Object::Stream($input)
    };
}

#[macro_export]
macro_rules! reference {
    ($one:expr, $two:expr) => {
        Object::Reference(($one, $two))
    };
}

#[macro_export]
macro_rules! string_lit {
    ($input:expr) => {
        Object::String($input.to_vec(), StringFormat::Literal)
    };
}

#[macro_export]
macro_rules! string_hex {
    ($input:expr) => {
        Object::String($input.to_vec(), StringFormat::Hexadecimal)
    };
}

#[macro_export]
macro_rules! dictionary_struct {
    // trailing comma case
    ($($key:expr => $value:expr,)+) => (dictionary_struct!($($key => $value),+));

    ( $($key:expr => $value:expr),* ) => {
        {
            let mut _map = Dictionary::default();
            $(
                let _ = _map.insert($key.as_bytes().to_vec(), $value);
            )*
            _map
        }
    };
}

#[macro_export]
macro_rules! xref_n {
    // trailing comma case
    ( $generation:expr, $offset:expr ) => {
        XrefEntry::InUse {
            generation: $generation,
            offset: $offset,
        }
    };
}

use crate::{object::Object, ObjectId};
use anyhow::Result;

impl<'a> Object<'a> {
	/// Returns the value of a /Contents entry. Could be an array of references
	/// a la `[ 4 0 R 6 0 R ]` or a single reference `4 0 R`. For simplicity we
	/// always return an array
	pub fn as_contents_reference(&self) -> Result<Vec<ObjectId>> {
		match self {
			Object::Array(s) => {
				let v = s.iter();
				let mut r = Vec::new();
				for obj in v {
					let reference = obj.as_reference()?;
					r.push(reference);
				}
				Ok(r)
			}
			Object::Reference(r) => Ok(vec![*r]),
			_ => unreachable!(),
		}
	}
}

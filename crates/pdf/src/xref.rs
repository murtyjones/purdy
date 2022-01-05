use std::collections::BTreeMap;

use crate::{GenerationNumber, ObjectNumber};

#[derive(Debug, Clone, PartialEq)]
pub struct Xref {
	/// Entries for indirect object.
	pub entries: BTreeMap<ObjectNumber, XrefEntry>,

	/// Total number of entries (including free entries), equal to the highest
	/// object number plus 1.
	pub size: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum XrefEntry {
	Free,
	InUse {
		offset: usize,
		generation: GenerationNumber,
	},
	Compressed {
		// I think this u32 should be ObjectNumber
		container: u32,
		index: u16,
	},
}

impl Xref {
	pub fn new() -> Self {
		Xref {
			entries: BTreeMap::new(),
			size: 0,
		}
	}

	pub fn insert(&mut self, key: u32, value: XrefEntry) -> Option<XrefEntry> {
		self.entries.insert(key, value)
	}

	pub fn extend<T: IntoIterator<Item = (u32, XrefEntry)>>(&mut self, entries: T) {
		self.entries.extend(entries)
	}
}

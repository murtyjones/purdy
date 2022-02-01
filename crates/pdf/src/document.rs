use anyhow::Result;
use nom::{
    error::{Error as NomError, VerboseError},
    ErrorConvert,
};
use std::collections::BTreeMap;

mod parsers;

use self::parsers::{all_objects, make_xref_table};
use crate::{dictionary::Dictionary, error::HandlingError, object::Object, xref::Xref, ObjectId};
use parsers::version;

#[derive(Debug, PartialEq)]
pub struct Document<'a> {
    pub version: f64,

    pub xref: Xref,

    pub trailer: Dictionary<'a>,

    pub objects: BTreeMap<ObjectId, Object<'a>>,
}

impl<'a> Default for Document<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Document<'a> {
    pub fn new() -> Document<'a> {
        Document {
            version: 1.7,
            xref: Xref::default(),
            trailer: Dictionary::default(),
            objects: BTreeMap::new(),
        }
    }

    // FIXME: Need to think through whether or not the static lifetime is appropriate.
    //        it seems to make sense to me, because the input should be available for
    //        the lifetime of the program. But this could also cause issues with wasm
    //        bindgen so it may actually be infeasible
    pub fn from_bytes(input: &'static [u8]) -> Result<Document> {
        // Version should appear in the first 50 bytes
        let version = version(&input[..50])?.1;
        let (xref, trailer) = make_xref_table(input)?;
        let document = Document {
            version,
            xref,
            trailer,
            ..Document::default()
        };
        let document = all_objects(input, document)?;

        Ok(document)
    }
}

impl<'a> Document<'a> {
    pub fn get_object_ids(&self) -> std::collections::btree_map::Keys<'_, ObjectId, Object<'a>> {
        self.objects.keys()
    }

    pub fn get_object(&self, object_id: ObjectId) -> Result<&Object<'a>> {
        self.objects
            .get(&object_id)
            .ok_or_else(|| HandlingError::ObjectNotFound.into())
    }

    pub fn get_page_ids(&self) -> Result<Vec<ObjectId>> {
        let catalog = self.get_catalog()?;
        let pages_loc = catalog.get(b"Pages").and_then(Object::as_reference)?;
        let pages = self.get_object(pages_loc).and_then(Object::as_dict)?;
        pages
            .get(b"Kids")
            .and_then(Object::as_array)?
            .iter()
            .map(Object::as_reference)
            .collect()
    }

    pub fn get_catalog(&self) -> Result<&Dictionary<'a>> {
        let catalog_loc = self.trailer.get(b"Root").and_then(Object::as_reference)?;

        self.objects
            .get(&catalog_loc)
            .ok_or_else(|| HandlingError::ObjectNotFound.into())
            .and_then(Object::as_dict)
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use crate::known::sample::{sample_pdf_objects, sample_pdf_trailer, sample_pdf_xref};
    use crate::known::sample_no_xref::{sample_no_xref_pdf_objects, sample_no_xref_pdf_trailer};
    use std::collections::BTreeMap;

    use super::Document;
    use crate::{
        bool, dict, dictionary_struct, int, name, null, object::StringFormat, real, reference,
        stream::Stream, string, string_lit, xref_n,
    };
    use crate::{
        dictionary::Dictionary,
        object::Object,
        utils::*,
        xref::{Xref, XrefEntry},
        ObjectId,
    };

    #[test]
    fn test_sample_pdf_metadata() {
        let bytes = read_file_bytes(concat!(
            env!("CARGO_WORKSPACE_DIR"),
            "/pdfs/sample/sample.pdf"
        ));
        let bytes: &[u8] = unsafe { extend_lifetime(&bytes) };
        let pdf = Document::from_bytes(bytes).expect("could not parse sample");
        assert_relative_eq!(pdf.version, 1.3);
        assert_eq!(pdf.xref, sample_pdf_xref());
        assert_eq!(pdf.trailer, sample_pdf_trailer());
    }

    #[test]
    fn test_sample_pdf_objects() {
        let bytes = read_file_bytes(concat!(
            env!("CARGO_WORKSPACE_DIR"),
            "/pdfs/sample/sample.pdf"
        ));
        let bytes: &[u8] = unsafe { extend_lifetime(&bytes) };
        let pdf = Document::from_bytes(bytes).expect("could not parse sample");
        let mut expected_objects = sample_pdf_objects().into_iter();
        for _ in 0..expected_objects.len() {
            let (id, expected_obj) = expected_objects.next().unwrap();
            let actual_obj = pdf.get_object(id).unwrap();
            assert_eq!(*actual_obj, expected_obj);
        }
        assert_eq!(pdf.get_page_ids().unwrap(), vec![(4, 0), (6, 0)]);
    }

    #[test]
    fn test_sample_pdf_no_xref_metadata() {
        let bytes = read_file_bytes(concat!(
            env!("CARGO_WORKSPACE_DIR"),
            "/pdfs/sample-no-xref-entries/sample-no-xref-entries.pdf"
        ));
        let bytes: &[u8] = unsafe { extend_lifetime(&bytes) };
        let pdf = Document::from_bytes(bytes).expect("could not parse sample");
        assert_relative_eq!(pdf.version, 1.3);
        assert_eq!(pdf.trailer, sample_no_xref_pdf_trailer());
    }

    // TODO: Uncomment this whenever done iterating on the sample PDF with no xref
    // #[test]
    // fn test_sample_pdf_no_xref_objects() {
    //     let bytes = read_file_bytes(concat!(
    //         env!("CARGO_WORKSPACE_DIR"),
    //         "/pdfs/sample-no-xref-entries/sample-no-xref-entries.pdf"
    //     ));
    //     let bytes: &[u8] = unsafe { extend_lifetime(&bytes) };
    //     let pdf = Document::from_bytes(bytes).expect("could not parse sample");
    //     let mut expected_objects = sample_no_xref_pdf_objects().into_iter();
    //     for _ in 0..expected_objects.len() {
    //         let (id, expected_obj) = expected_objects.next().unwrap();
    //         let actual_obj = pdf.get_object(id).unwrap();
    //         assert_eq!(*actual_obj, expected_obj);
    //     }
    //     assert_eq!(pdf.get_page_ids().unwrap(), vec![(4, 0), (6, 0)]);
    // }
}

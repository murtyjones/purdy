use crate::document::Document;
use crate::utils::extend_lifetime;
use anyhow::Result;

pub struct Pdf {
    document: Document<'static>,
}

impl Pdf {
    pub fn from_bytes(input: &[u8]) -> Result<Pdf> {
        let input = unsafe { extend_lifetime(input) };
        let document = Document::from_bytes(input)?;
        Ok(Pdf { document })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::*;

    #[test]
    fn sample_pdf() {
        let pdf = read_file_bytes(concat!(
            env!("CARGO_WORKSPACE_DIR"),
            "/pdfs/sample/sample.pdf"
        ));
        assert!(Pdf::from_bytes(&pdf).is_ok());
    }
}

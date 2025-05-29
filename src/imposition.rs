use crate::{args::Method, error::ImpositionError};
use std::path::{Path, PathBuf};

use lopdf::{dictionary, Document, Object, ObjectId, Stream};

#[derive(Debug, Clone)]
struct Imposition {
    pub document: Document,
    pub filepath: PathBuf,
}

impl Imposition {
    pub fn new(filepath: PathBuf) -> Result<Self, ImpositionError> {
        let document = Document::load(&filepath)?;
        Ok(Imposition { document, filepath })
    }

    pub fn impose(&mut self, method : Method) -> Result<(), ImpositionError> {
        match method {
            Method::Booklet => self.impose_booklet()?,
            Method::DoubleSided => self.impose_double_sided()?,
        }
        Ok(())
    }

    fn impose_booklet(&mut self) -> Result<(), ImpositionError> {
        // Placeholder for booklet imposition logic
        // This should rearrange the pages of the PDF to fit a booklet format
        // For now, we will just print a message
        println!("Imposing PDF in Booklet format...");
        Ok(())
    }

    fn impose_double_sided(&mut self) -> Result<(), ImpositionError> {
        // Placeholder for double-sided imposition logic
        // This should rearrange the pages of the PDF for double-sided printing
        // For now, we will just print a message
        println!("Imposing PDF in Double-Sided format...");
        Ok(())
    }

    pub fn save(&mut self, output: Option<PathBuf>) -> Result<(), ImpositionError> {
        let output_path = match output {
            Some(path) => path,
            None => self.filepath.with_extension("imposed.pdf"),
        };
        self.document.save(&output_path)?;
        Ok(())
    }
}
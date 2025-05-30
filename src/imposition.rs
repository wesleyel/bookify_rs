use std::{collections::BTreeMap, path::PathBuf};

use crate::{
    args::{FlipType, LayoutType, OddEven},
    calc::{generate_booklet_imposition, generate_double_sided_order},
    error::BookifyError,
};
use lopdf::{Dictionary, Document, Object, ObjectId, Stream};

/// PDF Document Imposer
pub struct PdfImposer {
    doc: Document,
    page_size: (f32, f32),
}

impl PdfImposer {
    /// Create new PdfImposer instance
    pub fn new(input_path: PathBuf) -> Result<Self, BookifyError> {
        let doc = Document::load(&input_path)?;
        let page_size = Self::get_page_size(&doc)?;
        Ok(Self { doc, page_size })
    }

    /// Get document page size from the first page
    fn get_page_size(doc: &Document) -> Result<(f32, f32), BookifyError> {
        let pages = doc.get_pages();
        if pages.is_empty() {
            return Err(BookifyError::invalid_pdf_format("Document has no pages"));
        }

        let first_page_id = pages.values().next().ok_or_else(|| {
            BookifyError::pdf_processing_failed(
                "Getting page",
                "Failed to get first page reference",
            )
        })?;

        let first_page = doc
            .get_object(*first_page_id)
            .and_then(Object::as_dict)
            .map_err(|_| {
                BookifyError::pdf_processing_failed(
                    "Getting page",
                    "Failed to get first page dictionary",
                )
            })?;

        let page_size = first_page.get(b"MediaBox").map_err(|_| {
            BookifyError::pdf_processing_failed(
                "Getting page size",
                "Failed to get MediaBox property",
            )
        })?;

        let page_size = page_size.as_array().map_err(|_| {
            BookifyError::pdf_processing_failed(
                "Getting page size",
                "MediaBox is not a valid array",
            )
        })?;

        let width = page_size[2].as_float().map_err(|_| {
            BookifyError::pdf_processing_failed("Getting page size", "Failed to get page width")
        })?;
        let height = page_size[3].as_float().map_err(|_| {
            BookifyError::pdf_processing_failed("Getting page size", "Failed to get page height")
        })?;

        Ok((width, height))
    }

    /// Create blank page with page size
    fn create_blank_page(&mut self) -> Result<ObjectId, BookifyError> {
        let mut page_dict = Dictionary::new();
        page_dict.set(b"Type", Object::Name(b"Page".to_vec()));

        let media_box = Object::Array(vec![
            Object::Real(0.0),
            Object::Real(0.0),
            Object::Real(self.page_size.0),
            Object::Real(self.page_size.1),
        ]);
        page_dict.set(b"MediaBox", media_box);

        let content_stream = Stream::new(Dictionary::new(), Vec::new());
        let content_id = self.doc.add_object(Object::Stream(content_stream));
        page_dict.set(b"Contents", Object::Reference(content_id));

        let page_id = self.doc.add_object(Object::Dictionary(page_dict));
        Ok(page_id)
    }

    /// Update document page structure
    fn update_document_pages(
        &mut self,
        new_kids_objects: Vec<Object>,
        page_count: u32,
    ) -> Result<(), BookifyError> {
        let catalog_dict = self.doc.catalog().map_err(|_| {
            BookifyError::pdf_processing_failed(
                "Updating document",
                "Failed to get catalog dictionary",
            )
        })?;

        let pages_dict_id = catalog_dict
            .get(b"Pages")
            .map_err(|_| {
                BookifyError::pdf_processing_failed(
                    "Updating document",
                    "Missing 'Pages' entry in catalog dictionary",
                )
            })?
            .as_reference()
            .map_err(|_| {
                BookifyError::pdf_processing_failed(
                    "Updating document",
                    "'Pages' entry is not a valid reference",
                )
            })?;

        let pages_dict = self
            .doc
            .get_object_mut(pages_dict_id)
            .and_then(Object::as_dict_mut)
            .map_err(|_| {
                BookifyError::pdf_processing_failed(
                    "Updating document",
                    "Failed to get mutable Pages dictionary",
                )
            })?;

        pages_dict.set(b"Kids", Object::Array(new_kids_objects));
        pages_dict.set(b"Count", Object::Integer(page_count as i64));

        Ok(())
    }

    /// Create new page objects array based on page order
    fn create_new_kids_objects(
        &mut self,
        page_order: &[u32],
        pages_map: &BTreeMap<u32, ObjectId>,
    ) -> Result<Vec<Object>, BookifyError> {
        let mut new_kids_objects: Vec<Object> = Vec::with_capacity(page_order.len());
        for &page_num in page_order {
            if page_num == 0 {
                let blank_page_id = self.create_blank_page()?;
                new_kids_objects.push(Object::Reference(blank_page_id));
            } else if let Some(&page_id) = pages_map.get(&page_num) {
                new_kids_objects.push(Object::Reference(page_id));
            } else {
                return Err(BookifyError::pdf_processing_failed(
                    "Creating page objects",
                    format!("Page {} not found in document", page_num),
                ));
            }
        }
        Ok(new_kids_objects)
    }

    /// Export booklet PDF
    pub fn export_booklet(&mut self, layout: LayoutType) -> Result<(), BookifyError> {
        let pages_map: BTreeMap<u32, ObjectId> = self.doc.get_pages();
        let total_pages = pages_map.len() as u32;
        let new_order = generate_booklet_imposition(total_pages, layout);

        let new_kids_objects = self.create_new_kids_objects(&new_order, &pages_map)?;
        self.update_document_pages(new_kids_objects, total_pages)?;
        Ok(())
    }

    /// Export double-sided PDF
    pub fn export_double_sided(
        &mut self,
        flip_type: FlipType,
        odd_even: OddEven,
    ) -> Result<(), BookifyError> {
        let pages_map: BTreeMap<u32, ObjectId> = self.doc.get_pages();
        let total_pages = pages_map.len() as u32;
        let new_order = generate_double_sided_order(total_pages, flip_type, odd_even);

        let new_kids_objects = self.create_new_kids_objects(&new_order, &pages_map)?;
        self.update_document_pages(new_kids_objects, new_order.len() as u32)?;
        Ok(())
    }

    /// Save document to specified path
    pub fn save(&mut self, output_path: PathBuf) -> Result<(), BookifyError> {
        self.doc
            .save(&output_path)
            .map_err(|e| BookifyError::io_error(e, &output_path))?;
        Ok(())
    }
}

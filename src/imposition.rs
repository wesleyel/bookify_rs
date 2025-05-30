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

        // Basic properties
        page_dict.set(b"Type", Object::Name(b"Page".to_vec()));

        // Inherit page properties from original document
        if let Some(first_page_id) = self.doc.get_pages().values().next() {
            if let Ok(first_page) = self
                .doc
                .get_object(*first_page_id)
                .and_then(Object::as_dict)
            {
                // Copy important properties
                if let Ok(resources) = first_page.get(b"Resources") {
                    page_dict.set(b"Resources", resources.clone());
                }
                if let Ok(rotate) = first_page.get(b"Rotate") {
                    page_dict.set(b"Rotate", rotate.clone());
                }
                if let Ok(group) = first_page.get(b"Group") {
                    page_dict.set(b"Group", group.clone());
                }
            }
        }

        // Set page size
        let media_box = Object::Array(vec![
            Object::Real(0.0),
            Object::Real(0.0),
            Object::Real(self.page_size.0),
            Object::Real(self.page_size.1),
        ]);
        page_dict.set(b"MediaBox", media_box);

        // Create empty content stream
        let content_stream = Stream::new(Dictionary::new(), Vec::new());
        let content_id = self.doc.add_object(Object::Stream(content_stream));
        page_dict.set(b"Contents", Object::Reference(content_id));

        // Set parent node reference
        if let Ok(pages_dict_id) = self
            .doc
            .catalog()
            .and_then(|c| c.get(b"Pages"))
            .and_then(|p| p.as_reference())
        {
            page_dict.set(b"Parent", Object::Reference(pages_dict_id));
        }

        let page_id = self.doc.add_object(Object::Dictionary(page_dict));
        Ok(page_id)
    }

    /// Update document page structure
    fn update_document_pages(
        &mut self,
        new_kids_objects: Vec<Object>,
        page_count: u32,
    ) -> Result<(), BookifyError> {
        let catalog_dict = self.doc.catalog()?;
        let pages_dict_id = catalog_dict.get(b"Pages")?.as_reference()?;

        // Get and clone required values first
        let (media_box, resources) = {
            let pages_dict = self.doc.get_object(pages_dict_id)?.as_dict()?;
            (
                pages_dict.get(b"MediaBox").ok().cloned(),
                pages_dict.get(b"Resources").ok().cloned(),
            )
        };

        // Then perform mutable operations
        let pages_dict = self.doc.get_object_mut(pages_dict_id)?.as_dict_mut()?;
        pages_dict.set(b"Kids", Object::Array(new_kids_objects));
        pages_dict.set(b"Count", Object::Integer(page_count as i64));

        if let Some(media_box) = media_box {
            pages_dict.set(b"MediaBox", media_box);
        }
        if let Some(resources) = resources {
            pages_dict.set(b"Resources", resources);
        }

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
        self.validate_page_tree()?;
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
        self.validate_page_tree()?;
        Ok(())
    }

    /// Save document to specified path
    pub fn save(&mut self, output_path: PathBuf) -> Result<(), BookifyError> {
        self.doc
            .save(&output_path)
            .map_err(|e| BookifyError::io_error(e, &output_path))?;
        Ok(())
    }

    fn validate_page_tree(&self) -> Result<(), BookifyError> {
        let catalog_dict = self.doc.catalog()?;
        let pages_dict_id = catalog_dict.get(b"Pages")?.as_reference()?;

        // Validate page tree structure
        let mut stack = vec![pages_dict_id];
        while let Some(node_id) = stack.pop() {
            let node = self.doc.get_object(node_id)?.as_dict()?;

            match node.get(b"Type")?.as_name()? {
                b"Pages" => {
                    // Validate page tree node
                    if let Ok(kids) = node.get(b"Kids")?.as_array() {
                        for kid in kids {
                            if let Ok(kid_id) = kid.as_reference() {
                                stack.push(kid_id);
                            }
                        }
                    }
                }
                b"Page" => {
                    // Validate page node
                    if !node.has(b"MediaBox") {
                        return Err(BookifyError::invalid_pdf_format("Page missing MediaBox"));
                    }
                }
                _ => {
                    return Err(BookifyError::invalid_pdf_format(
                        "Invalid node type in page tree",
                    ))
                }
            }
        }

        Ok(())
    }
}

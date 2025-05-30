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
    page_size: (i64, i64),
}

impl PdfImposer {
    /// Create new PdfImposer instance
    pub fn new(input_path: PathBuf) -> Result<Self, BookifyError> {
        let doc = Document::load(input_path)?;
        let page_size = Self::get_page_size(&doc)?;
        Ok(Self { doc, page_size })
    }

    /// Get document page size from catalog
    fn get_page_size(doc: &Document) -> Result<(i64, i64), BookifyError> {
        let catalog_dict = doc.catalog()?;
        let pages_dict_id = catalog_dict.get(b"Pages").map_err(|_| {
            BookifyError::Other("Missing 'Pages' entry in Catalog dictionary".to_string())
        })?;

        let pages_dict = pages_dict_id.as_dict()?;
        let page_size = pages_dict.get(b"MediaBox")?.as_array()?;
        let width = page_size[2].as_i64()?;
        let height = page_size[3].as_i64()?;
        Ok((width, height))
    }

    /// Create blank page with page size
    fn create_blank_page(&mut self) -> Result<ObjectId, BookifyError> {
        let mut page_dict = Dictionary::new();
        page_dict.set(b"Type", Object::Name(b"Page".to_vec()));

        let media_box = Object::Array(vec![
            Object::Integer(0),
            Object::Integer(0),
            Object::Integer(self.page_size.0),
            Object::Integer(self.page_size.1),
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
        let catalog_dict = self.doc.catalog()?;
        let pages_dict_id = catalog_dict
            .get(b"Pages")
            .map_err(|_| {
                BookifyError::Other("Missing 'Pages' entry in Catalog dictionary".to_string())
            })?
            .as_reference()
            .map_err(|_| {
                BookifyError::Other("'Pages' entry in Catalog is not a reference".to_string())
            })?;

        let pages_dict = self
            .doc
            .get_object_mut(pages_dict_id)
            .and_then(Object::as_dict_mut)
            .map_err(|_| BookifyError::Other("Cannot get mutable Pages dictionary".to_string()))?;

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
                return Err(BookifyError::Other(format!(
                    "Page {} not found in document",
                    page_num
                )));
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
        self.doc.save(output_path)?;
        Ok(())
    }
}

use std::{collections::BTreeMap, path::PathBuf};

use crate::{
    args::{FlipType, LayoutType, OddEven},
    calc::{generate_booklet_imposition, generate_double_sided_order},
    error::ImpositionError,
};
use lopdf::{Dictionary, Document, Object, ObjectId, Stream};

/// Update document page structure
fn update_document_pages(
    doc: &mut Document,
    new_kids_objects: Vec<Object>,
    page_count: u32,
) -> Result<(), ImpositionError> {
    // Get Catalog dictionary
    let catalog_dict = doc.catalog()?;

    // Get reference to Pages dictionary
    let pages_dict_id = catalog_dict
        .get(b"Pages")
        .map_err(|_| {
            ImpositionError::Other("Missing 'Pages' entry in Catalog dictionary".to_string())
        })?
        .as_reference()
        .map_err(|_| {
            ImpositionError::Other("'Pages' entry in Catalog is not a reference".to_string())
        })?;

    // Get and update Pages dictionary
    let pages_dict = doc
        .get_object_mut(pages_dict_id)
        .and_then(Object::as_dict_mut)
        .map_err(|_| ImpositionError::Other("Cannot get mutable Pages dictionary".to_string()))?;

    // Update Kids array and page count
    pages_dict.set(b"Kids", Object::Array(new_kids_objects));
    pages_dict.set(b"Count", Object::Integer(page_count as i64));

    Ok(())
}

/// Get page size from document
fn get_page_size(doc: &Document) -> Result<(i64, i64), ImpositionError> {
    let catalog_dict = doc.catalog()?;
    let pages_dict_id = catalog_dict.get(b"Pages").map_err(|_| {
        ImpositionError::Other("Missing 'Pages' entry in Catalog dictionary".to_string())
    });

    let pages_dict = pages_dict_id?.as_dict()?;

    let page_size = pages_dict.get(b"MediaBox")?.as_array()?;
    let width = page_size[2].as_i64()?;
    let height = page_size[3].as_i64()?;
    Ok((width, height))
}

/// Create blank page
fn create_blank_page(
    doc: &mut Document,
    page_size: (i64, i64),
) -> Result<ObjectId, ImpositionError> {
    // Create a new blank page
    let mut page_dict = Dictionary::new();

    // Set page type
    page_dict.set(b"Type", Object::Name(b"Page".to_vec()));

    // Set page size (A4)
    let media_box = Object::Array(vec![
        Object::Integer(0),
        Object::Integer(0),
        Object::Integer(page_size.0),
        Object::Integer(page_size.1),
    ]);
    page_dict.set(b"MediaBox", media_box);

    // Set blank content stream
    let content_stream = Stream::new(Dictionary::new(), Vec::new());
    let content_id = doc.add_object(Object::Stream(content_stream));
    page_dict.set(b"Contents", Object::Reference(content_id));

    // Add page to document
    let page_id = doc.add_object(Object::Dictionary(page_dict));

    Ok(page_id)
}

/// Rearrange PDF pages in a new order.
///
/// # Parameters
/// * `input_path` - Path to the input PDF file.
/// * `output_path` - Path to the output PDF file.
/// * `layout` - Layout type defining pages per sheet.
///
/// # Returns
/// `Result<()>` - Returns `Ok(())` if the operation is successful, otherwise returns `Err`.
///
/// # Example
/// ```no_run
/// use std::path::PathBuf;
/// use bookify_rs::args::LayoutType;
/// use bookify_rs::imposition::rearrange_pdf_pages;
///
/// let input = PathBuf::from("input.pdf");
/// let output = PathBuf::from("output.pdf");
/// let layout = LayoutType::FourUp;
/// match rearrange_pdf_pages(input, output, layout) {
///     Ok(_) => println!("PDF pages rearranged successfully!"),
///     Err(e) => eprintln!("Error rearranging PDF: {}", e),
/// }
/// ```
pub fn rearrange_pdf_pages(
    input_path: PathBuf,
    output_path: PathBuf,
    layout: LayoutType,
) -> Result<(), ImpositionError> {
    // 1. Load PDF document
    let mut doc = Document::load(input_path)?;
    let page_size = get_page_size(&doc)?;

    // 2. Get ObjectId mapping of all pages in the document
    // `get_pages()` returns a BTreeMap<u32, ObjectId>, where u32 is 1-based page number
    // ObjectId is a reference to the page dictionary.
    let pages_map: BTreeMap<u32, ObjectId> = doc.get_pages();

    // 3. Generate page order
    let original_num_pages = pages_map.len() as u32;
    let new_order = generate_booklet_imposition(original_num_pages, layout);

    // 4. Build new `Kids` array
    let mut new_kids_objects: Vec<Object> = Vec::with_capacity(new_order.len());
    for page_num in new_order {
        // Get ObjectId of the corresponding page from pages_map
        if page_num == 0 {
            // Add blank page
            let blank_page_id = create_blank_page(&mut doc, page_size)?;
            new_kids_objects.push(Object::Reference(blank_page_id));
        } else if let Some(&page_id) = pages_map.get(&page_num) {
            new_kids_objects.push(Object::Reference(page_id));
        } else {
            return Err(ImpositionError::Other(format!(
                "Page {} not found in document",
                page_num
            )));
        }
    }

    // 5. Find root page dictionary (usually in the "Pages" entry of the Catalog object), update Pages dictionary's "Kids" array, and update page count
    update_document_pages(&mut doc, new_kids_objects, original_num_pages)?;

    // 6. Save modified document
    doc.save(output_path)?;

    Ok(())
}

/// Export PDF for manual double-sided printing
///
/// # Parameters
/// * `input_path` - Path to the input PDF file
/// * `output_path` - Path to the output PDF file
/// * `reading_direction` - Page flipping direction
/// * `flip_direction` - Flip direction
/// * `odd_even` - Output odd or even pages
///
/// # Returns
/// `Result<()>` - Returns `Ok(())` if the operation is successful, otherwise returns `Err(ImpositionError)`
///
/// # Example
/// ```no_run
/// use bookify_rs::{
///     args::{FlipType, OddEven},
///     imposition::export_double_sided_pdf,
/// };
/// use std::path::PathBuf;
///
/// let input = PathBuf::from("input.pdf");
/// let output = PathBuf::from("output.pdf");
/// match export_double_sided_pdf(
///     input,
///     output,
///     FlipType::RR,
///     OddEven::Odd,
/// ) {
///     Ok(_) => println!("PDF exported successfully!"),
///     Err(e) => eprintln!("Error exporting PDF: {}", e),
/// }
/// ```
pub fn export_double_sided_pdf(
    input_path: PathBuf,
    output_path: PathBuf,
    flip_type: FlipType,
    odd_even: OddEven,
) -> Result<(), ImpositionError> {
    // 1. Load PDF document
    let mut doc = Document::load(&input_path)?;
    let page_size = get_page_size(&doc)?;

    // 2. Get ObjectId mapping of all pages in the document
    let pages_map: BTreeMap<u32, ObjectId> = doc.get_pages();
    let total_pages = pages_map.len() as u32;

    // 3. Generate page order
    let page_order = generate_double_sided_order(total_pages, flip_type, odd_even);

    // 4. Build new Kids array
    let mut new_kids_objects = Vec::with_capacity(page_order.len());
    for &page_num in &page_order {
        if page_num == 0 {
            // Add blank page
            let blank_page_id = create_blank_page(&mut doc, page_size)?;
            new_kids_objects.push(Object::Reference(blank_page_id));
        } else if let Some(&page_id) = pages_map.get(&page_num) {
            new_kids_objects.push(Object::Reference(page_id));
        } else {
            return Err(ImpositionError::Other(format!(
                "Page {} not found in document",
                page_num
            )));
        }
    }

    // 5. Update document page structure
    update_document_pages(&mut doc, new_kids_objects, page_order.len() as u32)?;

    // 6. Save modified document
    doc.save(&output_path)?;

    Ok(())
}

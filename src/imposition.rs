use std::{collections::BTreeMap, path::PathBuf};

use crate::{
    args::{FlipType, OddEven},
    calc::{generate_booklet_imposition, LayoutType},
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

/// Rearrange PDF pages in a new order.
///
/// # Parameters
/// * `input_path` - Path to the input PDF file.
/// * `output_path` - Path to the output PDF file.
/// * `new_order` - A Vec containing the new page order.
///                 For example, `vec![3, 1, 2]` means:
///                 - Page 3 of the original document becomes page 1 of the new document
///                 - Page 1 of the original document becomes page 2 of the new document
///                 - Page 2 of the original document becomes page 3 of the new document
///                 Page numbers are 1-based.
///
/// # Returns
/// `Result<()>` - Returns `Ok(())` if the operation is successful, otherwise returns `Err`.
///
/// # Example
/// ```no_run
/// use std::path::PathBuf;
/// use bookify_rs::calc::LayoutType;
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

    // 2. Get ObjectId mapping of all pages in the document
    // `get_pages()` returns a BTreeMap<u32, ObjectId>, where u32 is 1-based page number
    // ObjectId is a reference to the page dictionary.
    let pages_map: BTreeMap<u32, ObjectId> = doc.get_pages();

    // 3. Verify that the page numbers in the new order are valid
    let original_num_pages = pages_map.len() as u32;
    let new_order = generate_booklet_imposition(original_num_pages, layout);
    if new_order.is_empty() {
        return Err(ImpositionError::Other(
            "New order list cannot be empty.".to_string(),
        ));
    }
    if new_order.len() != original_num_pages as usize {
        return Err(ImpositionError::Other(format!(
            "New order list length ({}) must match original page count ({}).",
            new_order.len(),
            original_num_pages
        )));
    }
    for &page_num in &new_order {
        if page_num == 0 || page_num > original_num_pages {
            return Err(ImpositionError::Other(format!(
                "Invalid page number {} in new order. Must be between 1 and {}.",
                page_num, original_num_pages
            )));
        }
    }

    // 4. Build new `Kids` array
    let mut new_kids_objects: Vec<Object> = Vec::with_capacity(new_order.len());
    for page_num in new_order {
        // Get ObjectId of the corresponding page from pages_map
        if let Some(&page_id) = pages_map.get(&page_num) {
            new_kids_objects.push(Object::Reference(page_id));
        } else {
            // Theoretically, if new_order has been validated, this should not happen
            return Err(ImpositionError::Other(format!(
                "Page {} not found in document, despite validation",
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

    // 2. Get ObjectId mapping of all pages in the document
    let pages_map: BTreeMap<u32, ObjectId> = doc.get_pages();
    let total_pages = pages_map.len() as u32;

    // 3. Generate page order
    let page_order = generate_double_sided_order(total_pages, flip_type, odd_even)?;

    // 4. Build new Kids array
    let mut new_kids_objects = Vec::with_capacity(page_order.len());
    for &page_num in &page_order {
        if page_num == 0 {
            // Add blank page
            let blank_page_id = create_blank_page(&mut doc)?;
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

/// Generate page order for double-sided printing
fn generate_double_sided_order(
    total_pages: u32,
    flip_type: FlipType,
    odd_even: OddEven,
) -> Result<Vec<u32>, ImpositionError> {
    // Determine if reverse order is needed
    let should_reverse = match (flip_type, odd_even) {
        (FlipType::RR, OddEven::Odd) => true,
        (FlipType::RR, OddEven::Even) => true,
        (FlipType::NN, OddEven::Odd) => false,
        (FlipType::NN, OddEven::Even) => false,
        (FlipType::RN, OddEven::Odd) => true,
        (FlipType::RN, OddEven::Even) => false,
        (FlipType::NR, OddEven::Odd) => false,
        (FlipType::NR, OddEven::Even) => true,
    };

    // Generate page sequence
    let mut pages = match odd_even {
        OddEven::Odd => {
            // Generate odd page sequence: 1, 3, 5, ...
            (1..=total_pages).step_by(2).collect::<Vec<u32>>()
        }
        OddEven::Even => {
            // Generate even page sequence: 2, 4, 6, ...
            let mut even_pages: Vec<u32> = (2..=total_pages).step_by(2).collect();

            // If total pages is odd, add a blank page (represented by 0)
            if total_pages % 2 == 1 {
                even_pages.push(0);
            }
            even_pages
        }
    };

    // If reverse order is needed, reverse page sequence
    if should_reverse {
        pages.reverse();
    }

    Ok(pages)
}

/// Create blank page
fn create_blank_page(doc: &mut Document) -> Result<ObjectId, ImpositionError> {
    // Create a new blank page
    let mut page_dict = Dictionary::new();

    // Set page type
    page_dict.set(b"Type", Object::Name(b"Page".to_vec()));

    // Set page size (A4)
    let media_box = Object::Array(vec![
        Object::Integer(0),
        Object::Integer(0),
        Object::Integer(595), // A4 width (points)
        Object::Integer(842), // A4 height (points)
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

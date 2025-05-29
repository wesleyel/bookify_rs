use std::{collections::BTreeMap, path::PathBuf};

use crate::{
    calc::{generate_booklet_imposition, LayoutType},
    error::ImpositionError,
};
use lopdf::{Document, Object, ObjectId};

/// 重新调整 PDF 页面的顺序。
///
/// # 参数
/// * `input_path` - 输入 PDF 文件的路径。
/// * `output_path` - 输出 PDF 文件的路径。
/// * `new_order` - 一个 Vec，包含新的页面顺序。
///                 例如，`vec![3, 1, 2]` 表示将原始文档的第3页作为新文档的第1页，
///                 第1页作为新文档的第2页，第2页作为新文档的第3页。
///                 页面编号是 1-based。
///
/// # 返回
/// `Result<()>` - 如果操作成功则返回 `Ok(())`，否则返回 `Err`。
///
/// # 示例
/// ```no_run
/// // 假设你有一个名为 "input.pdf" 的文件
/// // 将原始文档的第3页放在第1位，第1页放在第2位，第2页放在第3位
/// let new_order = vec![3, 1, 2];
/// match rearrange_pdf_pages("input.pdf", "output.pdf", new_order) {
///     Ok(_) => println!("PDF pages rearranged successfully!"),
///     Err(e) => eprintln!("Error rearranging PDF: {}", e),
/// }
/// ```
pub fn rearrange_pdf_pages(
    input_path: PathBuf,
    output_path: PathBuf,
    layout: LayoutType,
) -> Result<(), ImpositionError> {
    // 1. 加载 PDF 文档
    let mut doc = Document::load(input_path)?;

    // 2. 获取文档中所有页面的 ObjectId 映射
    // `get_pages()` 返回一个 BTreeMap<u32, ObjectId>，其中 u32 是 1-based 的页面编号
    // ObjectId 是该页面字典的引用。
    let pages_map: BTreeMap<u32, ObjectId> = doc.get_pages();

    // 3. 验证新顺序中的页面号是否有效
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

    // 4. 构建新的 `Kids` 数组
    let mut new_kids_objects: Vec<Object> = Vec::with_capacity(new_order.len());
    for page_num in new_order {
        // 从 pages_map 中获取对应页面的 ObjectId
        if let Some(&page_id) = pages_map.get(&page_num) {
            new_kids_objects.push(Object::Reference(page_id));
        } else {
            // 理论上，如果 new_order 经过了验证，这里不应该发生
            return Err(ImpositionError::Other(format!(
                "Page {} not found in document, despite validation.",
                page_num
            )));
        }
    }

    // 5. 找到根页面字典 (通常在 Catalog 对象的 "Pages" 条目中)
    // 首先获取 Catalog 字典
    let catalog_dict = doc.catalog()?;

    // 获取 Pages 字典的引用
    let pages_dict_id_obj = catalog_dict.get(b"Pages").map_err(|_| {
        ImpositionError::Other("Catalog dictionary missing 'Pages' entry.".to_string())
    })?;
    let pages_dict_id = pages_dict_id_obj.as_reference().map_err(|_| {
        ImpositionError::Other("'Pages' entry in Catalog is not a reference.".to_string())
    })?;

    let pages_dict = doc
        .get_object_mut(pages_dict_id)
        .and_then(Object::as_dict_mut)
        .map_err(|_| {
            ImpositionError::Other("Failed to get mutable Pages dictionary.".to_string())
        })?;

    // 6. 更新 Pages 字典的 "Kids" 数组
    // 将新的 `Kids` 数组作为 Object::Array 放入 Pages 字典
    pages_dict.set(b"Kids", Object::Array(new_kids_objects));

    // 7. 更新文档的页面计数 (Optional but good practice)
    // Although lopdf usually updates this correctly on save, explicitly setting it can prevent issues.
    pages_dict.set(b"Count", Object::Integer(original_num_pages as i64));

    // 8. 保存修改后的文档
    doc.save(output_path)?;

    Ok(())
}

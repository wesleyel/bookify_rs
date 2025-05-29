use std::{collections::BTreeMap, path::PathBuf};

use crate::{
    args::{FlipType, OddEven},
    calc::{generate_booklet_imposition, LayoutType},
    error::ImpositionError,
};
use lopdf::{Dictionary, Document, Object, ObjectId, Stream};

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

/// 导出用于手动双面打印的 PDF
///
/// # 参数
/// * `input_path` - 输入 PDF 文件的路径
/// * `output_path` - 输出 PDF 文件的路径
/// * `reading_direction` - 翻页方向
/// * `flip_direction` - 翻转方向
/// * `odd_even` - 输出奇数页还是偶数页
///
/// # 返回
/// `Result<()>` - 如果操作成功则返回 `Ok(())`，否则返回 `Err(ImpositionError)`
///
/// # 示例
/// ```no_run
/// use bookify_rs::{
///     args::{FlipDirection, OddEven, ReadingDirection},
///     imposition::export_double_sided_pdf,
/// };
/// use std::path::PathBuf;
///
/// let input = PathBuf::from("input.pdf");
/// let output = PathBuf::from("output.pdf");
/// export_double_sided_pdf(
///     input,
///     output,
///     ReadingDirection::LeftToRight,
///     FlipDirection::ShortEdge,
///     OddEven::Odd,
/// )?;
/// ```
pub fn export_double_sided_pdf(
    input_path: PathBuf,
    output_path: PathBuf,
    flip_type: FlipType,
    odd_even: OddEven,
) -> Result<(), ImpositionError> {
    // 1. 加载 PDF 文档
    let mut doc = Document::load(&input_path)?;

    // 2. 获取文档中所有页面的 ObjectId 映射
    let pages_map: BTreeMap<u32, ObjectId> = doc.get_pages();
    let total_pages = pages_map.len() as u32;

    // 3. 生成页面顺序
    let page_order = generate_double_sided_order(total_pages, flip_type, odd_even)?;

    // 4. 构建新的 Kids 数组
    let mut new_kids_objects = Vec::with_capacity(page_order.len());
    for &page_num in &page_order {
        if page_num == 0 {
            // 添加空白页
            let blank_page_id = create_blank_page(&mut doc)?;
            new_kids_objects.push(Object::Reference(blank_page_id));
        } else if let Some(&page_id) = pages_map.get(&page_num) {
            new_kids_objects.push(Object::Reference(page_id));
        } else {
            return Err(ImpositionError::Other(format!(
                "页面 {} 在文档中未找到",
                page_num
            )));
        }
    }

    // 5. 更新文档的页面结构
    update_document_pages(&mut doc, new_kids_objects, page_order.len() as u32)?;

    // 6. 保存修改后的文档
    doc.save(&output_path)?;

    Ok(())
}

/// 生成双面打印的页面顺序
fn generate_double_sided_order(
    total_pages: u32,
    flip_type: FlipType,
    odd_even: OddEven,
) -> Result<Vec<u32>, ImpositionError> {
    // 确定是否需要倒序
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

    // 生成页面序列
    let mut pages = match odd_even {
        OddEven::Odd => {
            // 生成奇数页序列：1, 3, 5, ...
            (1..=total_pages).step_by(2).collect::<Vec<u32>>()
        }
        OddEven::Even => {
            // 生成偶数页序列：2, 4, 6, ...
            let mut even_pages: Vec<u32> = (2..=total_pages).step_by(2).collect();

            // 如果总页数为奇数，添加一个空白页（用 0 表示）
            if total_pages % 2 == 1 {
                even_pages.push(0);
            }
            even_pages
        }
    };

    // 如果需要倒序，则反转页面序列
    if should_reverse {
        pages.reverse();
    }

    Ok(pages)
}

/// 创建空白页
fn create_blank_page(doc: &mut Document) -> Result<ObjectId, ImpositionError> {
    // 创建一个新的空白页面
    let mut page_dict = Dictionary::new();

    // 设置页面类型
    page_dict.set(b"Type", Object::Name(b"Page".to_vec()));

    // 设置页面大小（A4）
    let media_box = Object::Array(vec![
        Object::Integer(0),
        Object::Integer(0),
        Object::Integer(595), // A4 宽度（点）
        Object::Integer(842), // A4 高度（点）
    ]);
    page_dict.set(b"MediaBox", media_box);

    // 设置空白内容流
    let content_stream = Stream::new(Dictionary::new(), Vec::new());
    let content_id = doc.add_object(Object::Stream(content_stream));
    page_dict.set(b"Contents", Object::Reference(content_id));

    // 添加页面到文档
    let page_id = doc.add_object(Object::Dictionary(page_dict));

    Ok(page_id)
}

/// 更新文档的页面结构
fn update_document_pages(
    doc: &mut Document,
    new_kids_objects: Vec<Object>,
    page_count: u32,
) -> Result<(), ImpositionError> {
    // 获取 Catalog 字典
    let catalog_dict = doc.catalog()?;

    // 获取 Pages 字典的引用
    let pages_dict_id = catalog_dict
        .get(b"Pages")
        .map_err(|_| ImpositionError::Other("Catalog 字典中缺少 'Pages' 条目".to_string()))?
        .as_reference()
        .map_err(|_| ImpositionError::Other("Catalog 中的 'Pages' 条目不是引用".to_string()))?;

    // 获取并更新 Pages 字典
    let pages_dict = doc
        .get_object_mut(pages_dict_id)
        .and_then(Object::as_dict_mut)
        .map_err(|_| ImpositionError::Other("无法获取可变的 Pages 字典".to_string()))?;

    // 更新 Kids 数组和页面计数
    pages_dict.set(b"Kids", Object::Array(new_kids_objects));
    pages_dict.set(b"Count", Object::Integer(page_count as i64));

    Ok(())
}

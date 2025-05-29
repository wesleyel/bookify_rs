use crate::{
    args::{Cli},
    calc::{generate_booklet_imposition},
    error::ImpositionError,
};
use lopdf::{dictionary, Dictionary, Document, Object, Stream};

/// 处理 PDF 文件并生成小册子排版
pub fn process_pdf(cli: &Cli) -> Result<(), ImpositionError> {
    // 1. 打开输入 PDF
    let doc = Document::load(&cli.input)?;

    // 2. 获取实际页数
    let actual_pages = doc.get_pages().len() as u32;

    // 3. 生成排版顺序
    let imposition_order = generate_booklet_imposition(actual_pages, cli.layout);

    // 4. 创建新的 PDF 文档
    let mut new_doc = Document::new();

    // 5. 复制页面内容
    for &page_num in &imposition_order {
        if page_num == 0 {
            // 添加空白页
            let blank_page = create_blank_page(&doc)?;
            let page_id = new_doc.add_object(blank_page);

            // 将页面添加到文档的页面树中
            let mut pages_dict = Dictionary::new();
            pages_dict.set("Type", "Pages");
            pages_dict.set("Kids", vec![Object::Reference(page_id)]);
            pages_dict.set("Count", 1);
            let pages_id = new_doc.add_object(Object::Dictionary(pages_dict));

            // 更新文档目录
            let mut catalog = Dictionary::new();
            catalog.set("Type", "Catalog");
            catalog.set("Pages", Object::Reference(pages_id));
            new_doc.add_object(Object::Dictionary(catalog));
        } else {
            // 复制实际页面
            let pages = doc.get_pages();
            let page_id = pages
                .get(&(page_num as u32))
                .ok_or(ImpositionError::Other(format!("找不到页面 {}", page_num)))?;
            let page = doc.get_object(*page_id)?;

            // 克隆页面对象到新文档
            let new_page_id = new_doc.add_object(page.clone());

            // 将页面添加到文档的页面树中
            let mut pages_dict = Dictionary::new();
            pages_dict.set("Type", "Pages");
            pages_dict.set("Kids", vec![Object::Reference(new_page_id)]);
            pages_dict.set("Count", 1);
            let pages_id = new_doc.add_object(Object::Dictionary(pages_dict));

            // 更新文档目录
            let mut catalog = Dictionary::new();
            catalog.set("Type", "Catalog");
            catalog.set("Pages", Object::Reference(pages_id));
            new_doc.add_object(Object::Dictionary(catalog));
        }
    }

    // 6. 保存输出文件
    let output_path = if let Some(path) = &cli.output {
        path.clone()
    } else {
        let mut path = cli.input.clone();
        path.set_extension("booklet.pdf");
        path
    };

    new_doc.save(&output_path)?;

    Ok(())
}

/// 创建空白页面
fn create_blank_page(doc: &Document) -> Result<Object, ImpositionError> {
    // 获取第一页的尺寸作为参考
    let pages = doc.get_pages();
    let first_page_id = pages
        .get(&1)
        .ok_or(ImpositionError::Other("文档没有页面".to_string()))?;
    let first_page = doc.get_object(*first_page_id)?;

    // 获取页面尺寸
    let media_box = first_page.as_dict()?.get(b"MediaBox")?;

    // 创建空白页面字典
    let mut dict = Dictionary::new();
    dict.set("Type", "Page");
    dict.set("MediaBox", media_box.clone());
    dict.set("Resources", Dictionary::new());
    dict.set("Contents", Stream::new(dictionary! {}, vec![]));

    Ok(Object::Dictionary(dict))
}

use clap::{Parser, ValueEnum};
use lopdf::{dictionary, Dictionary, Document, Object, ObjectId, Stream};

/// 翻页方向
#[derive(Copy, Clone, Debug, ValueEnum)]
enum ReadingDirection {
    /// 从左到右翻页
    LeftToRight,
    /// 从右到左翻页  
    RightToLeft,
}

/// 翻转方向
#[derive(Copy, Clone, Debug, ValueEnum)]
enum FlipDirection {
    /// 短边翻转
    ShortEdge,
    /// 长边翻转
    LongEdge,
}

/// 拼版工具：将普通 PDF 转成小册子拼版 PDF
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// 输入 PDF 文件
    #[arg(short, long)]
    input: String,

    /// 输出 PDF 文件
    #[arg(short, long)]
    output: String,

    /// 目标总页数（默认16页）
    #[arg(long, default_value = "16")]
    pages: usize,

    /// 翻页方向
    #[arg(long, value_enum, default_value = "left-to-right")]
    reading_direction: ReadingDirection,

    /// 翻转方向
    #[arg(long, value_enum, default_value = "short-edge")]
    flip_direction: FlipDirection,
}

/// 页面尺寸信息
#[derive(Debug, Clone, Copy)]
struct PageSize {
    width: f64,
    height: f64,
}

impl PageSize {
    fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }

    /// 获取旋转后的尺寸
    fn rotated(&self) -> Self {
        Self::new(self.height, self.width)
    }
}

fn main() {
    let args = Args::parse();

    // 1. 读取 PDF
    let mut doc = Document::load(&args.input).expect("无法读取输入 PDF");

    // 2. 获取原始页面数量和尺寸
    let original_pages: Vec<_> = doc.get_pages().values().cloned().collect();
    let original_page_count = original_pages.len();

    println!("原始页面数量: {}", original_page_count);

    // 获取第一页的尺寸作为模板
    let original_size = get_page_size(&doc, original_pages[0]);
    println!(
        "原始页面尺寸: {}x{} pt",
        original_size.width, original_size.height
    );

    // 3. 自动补齐页面到目标数量
    let mut extended_pages = original_pages.clone();
    while extended_pages.len() < args.pages {
        let blank_page_id = create_blank_page(&mut doc, original_size);
        extended_pages.push(blank_page_id);
    }

    if extended_pages.len() > original_page_count {
        println!(
            "已补齐 {} 个空白页",
            extended_pages.len() - original_page_count
        );
    }

    // 4. 计算拼版顺序
    let booklet_order =
        calculate_booklet_order(args.pages, args.reading_direction, args.flip_direction);

    println!("拼版顺序: {:?}", booklet_order);

    // 5. 计算目标页面尺寸
    let target_size = calculate_target_size(original_size, args.flip_direction);
    println!(
        "目标页面尺寸: {}x{} pt",
        target_size.width, target_size.height
    );

    // 6. 创建新文档
    let mut new_doc = Document::with_version("1.5");
    let mut new_pages = Vec::new();

    for (sheet_idx, &(a, b, c, d)) in booklet_order.iter().enumerate() {
        println!(
            "创建第 {} 张纸，页面顺序: [{}, {}, {}, {}]",
            sheet_idx + 1,
            a + 1,
            b + 1,
            c + 1,
            d + 1
        );

        // 创建页面ID
        let page_id = new_doc.new_object_id();

        // 创建页面内容
        let (resources, content) = create_imposed_page_content(
            &mut doc,
            &mut new_doc,
            &extended_pages,
            [a, b, c, d],
            original_size,
            target_size,
            args.flip_direction,
        );

        // 创建内容流
        let content_id = new_doc.new_object_id();
        let content_obj =
            Object::Stream(Stream::new(Dictionary::new(), content.as_bytes().to_vec()));
        new_doc.objects.insert(content_id, content_obj);

        // 创建页面对象
        let page_obj = lopdf::dictionary! {
            "Type" => "Page",
            "MediaBox" => vec![0.into(), 0.into(), target_size.width.into(), target_size.height.into()],
            "Contents" => Object::Reference(content_id),
            "Resources" => Object::Dictionary(resources),
        };

        new_doc
            .objects
            .insert(page_id, Object::Dictionary(page_obj));
        new_pages.push(page_id);
    }

    // 7. 构建页面树和文档结构
    finalize_document(&mut new_doc, new_pages);

    // 8. 保存文档
    new_doc.save(&args.output).expect("保存 PDF 失败");
    println!("拼版完成，输出文件：{}", args.output);
}

/// 获取页面尺寸
fn get_page_size(doc: &Document, page_id: ObjectId) -> PageSize {
    if let Ok(page_obj) = doc.get_object(page_id) {
        if let Object::Dictionary(page_dict) = page_obj {
            if let Ok(Object::Array(media_box)) = page_dict.get(b"MediaBox") {
                if media_box.len() >= 4 {
                    let width = media_box[2].as_float().unwrap_or(595.0)
                        - media_box[0].as_float().unwrap_or(0.0);
                    let height = media_box[3].as_float().unwrap_or(842.0)
                        - media_box[1].as_float().unwrap_or(0.0);
                    return PageSize::new(width as f64, height as f64);
                }
            }
        }
    }

    // 默认 A4 尺寸
    PageSize::new(595.0, 842.0)
}

/// 创建空白页
fn create_blank_page(doc: &mut Document, size: PageSize) -> ObjectId {
    let page_id = doc.new_object_id();

    let page_obj = lopdf::dictionary! {
        "Type" => "Page",
        "MediaBox" => vec![0.into(), 0.into(), size.width.into(), size.height.into()],
        "Contents" => Object::Array(vec![]), // 空内容
    };

    doc.objects.insert(page_id, Object::Dictionary(page_obj));
    page_id
}

/// 计算拼版顺序
fn calculate_booklet_order(
    total_pages: usize,
    reading_direction: ReadingDirection,
    flip_direction: FlipDirection,
) -> Vec<(usize, usize, usize, usize)> {
    let sheets = total_pages / 4;
    let mut order = Vec::new();

    for sheet in 0..sheets {
        let base = sheet * 4;

        match (reading_direction, flip_direction) {
            (ReadingDirection::LeftToRight, FlipDirection::ShortEdge) => {
                // 标准从左到右，短边翻转
                let a = total_pages - 1 - base; // 最后页
                let b = base; // 第一页
                let c = base + 1; // 第二页
                let d = total_pages - 2 - base; // 倒数第二页
                order.push((a, b, c, d));
            }
            (ReadingDirection::LeftToRight, FlipDirection::LongEdge) => {
                // 从左到右，长边翻转
                let a = base; // 第一页
                let b = total_pages - 1 - base; // 最后页
                let c = total_pages - 2 - base; // 倒数第二页
                let d = base + 1; // 第二页
                order.push((a, b, c, d));
            }
            (ReadingDirection::RightToLeft, FlipDirection::ShortEdge) => {
                // 从右到左，短边翻转
                let a = base; // 第一页
                let b = total_pages - 1 - base; // 最后页
                let c = total_pages - 2 - base; // 倒数第二页
                let d = base + 1; // 第二页
                order.push((a, b, c, d));
            }
            (ReadingDirection::RightToLeft, FlipDirection::LongEdge) => {
                // 从右到左，长边翻转
                let a = total_pages - 1 - base; // 最后页
                let b = base; // 第一页
                let c = base + 1; // 第二页
                let d = total_pages - 2 - base; // 倒数第二页
                order.push((a, b, c, d));
            }
        }
    }

    order
}

/// 计算目标页面尺寸
fn calculate_target_size(original_size: PageSize, flip_direction: FlipDirection) -> PageSize {
    match flip_direction {
        FlipDirection::ShortEdge => {
            // 短边翻转：原页面并排，高度不变
            PageSize::new(original_size.width * 2.0, original_size.height * 2.0)
        }
        FlipDirection::LongEdge => {
            // 长边翻转：原页面上下排列，宽度不变
            PageSize::new(original_size.width * 2.0, original_size.height * 2.0)
        }
    }
}

/// 创建拼版页面内容
fn create_imposed_page_content(
    doc: &mut Document,
    new_doc: &mut Document,
    pages: &[ObjectId],
    indices: [usize; 4],
    original_size: PageSize,
    _target_size: PageSize,
    flip_direction: FlipDirection,
) -> (Dictionary, String) {
    let mut xobject_resources = Dictionary::new();
    let mut content = Vec::new();

    // 计算每个页面的位置
    let positions = match flip_direction {
        FlipDirection::ShortEdge => [
            (0.0, original_size.height),                 // 左上
            (original_size.width, original_size.height), // 右上
            (0.0, 0.0),                                  // 左下
            (original_size.width, 0.0),                  // 右下
        ],
        FlipDirection::LongEdge => [
            (0.0, original_size.height),                 // 左上
            (original_size.width, original_size.height), // 右上
            (0.0, 0.0),                                  // 左下
            (original_size.width, 0.0),                  // 右下
        ],
    };

    for (i, &page_idx) in indices.iter().enumerate() {
        if page_idx >= pages.len() {
            continue; // 跳过不存在的页面
        }

        let orig_page_id = pages[page_idx];
        let (x, y) = positions[i];

        // 创建 XObject 引用 - 使用正确的 ObjectId 类型
        let xobj_name = format!("X{}", i);
        let xobject_id = new_doc.new_object_id();

        // 复制原页面对象作为 XObject
        if let Ok(orig_obj) = doc.get_object(orig_page_id) {
            let mut xobj = orig_obj.clone();

            // 转换为 XObject
            if let Object::Dictionary(ref mut dict) = xobj {
                dict.set("Type", "XObject");
                dict.set("Subtype", "Form");
                dict.set(
                    "BBox",
                    Object::Array(vec![
                        0.into(),
                        0.into(),
                        original_size.width.into(),
                        original_size.height.into(),
                    ]),
                );

                // 如果原页面有内容流，需要复制
                if let Ok(contents) = dict.get(b"Contents") {
                    // 保留原有的内容流
                    dict.set("Contents", contents.clone());
                }

                // 如果原页面有资源，需要复制
                if let Ok(resources) = dict.get(b"Resources") {
                    dict.set("Resources", resources.clone());
                }
            }

            new_doc.objects.insert(xobject_id, xobj);
        }

        xobject_resources.set(xobj_name.as_bytes(), Object::Reference(xobject_id));

        // 添加到内容流
        content.push(format!("q 1 0 0 1 {} {} cm /{} Do Q\n", x, y, xobj_name));
    }

    // 创建最终的资源字典
    let mut final_resources = Dictionary::new();
    final_resources.set("XObject", Object::Dictionary(xobject_resources));

    (final_resources, content.join(""))
}

/// 完成文档结构
fn finalize_document(doc: &mut Document, page_ids: Vec<ObjectId>) {
    // 创建页面树
    let pages_id = doc.new_object_id();
    let kids = page_ids
        .iter()
        .map(|&id| Object::Reference(id))
        .collect::<Vec<_>>();

    let pages_dict = lopdf::dictionary! {
        "Type" => "Pages",
        "Kids" => kids,
        "Count" => page_ids.len() as i32,
    };

    doc.objects.insert(pages_id, Object::Dictionary(pages_dict));

    // 更新每个页面的 Parent 引用
    for &page_id in &page_ids {
        if let Ok(Object::Dictionary(ref mut page_dict)) = doc.get_object_mut(page_id) {
            page_dict.set("Parent", Object::Reference(pages_id));
        }
    }

    // 创建根目录
    let catalog_id = doc.new_object_id();
    let catalog_dict = lopdf::dictionary! {
        "Type" => "Catalog",
        "Pages" => Object::Reference(pages_id),
    };

    doc.objects
        .insert(catalog_id, Object::Dictionary(catalog_dict));
    doc.trailer.set("Root", Object::Reference(catalog_id));
}

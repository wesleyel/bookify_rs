use crate::{
    args::{Cli, FlipDirection, ReadingDirection},
    error::ImpositionError,
};
use lopdf::{dictionary, Dictionary, Document, Object, ObjectId, Stream};
use std::path::PathBuf;

/// 页面尺寸信息
#[derive(Debug, Clone, Copy)]
pub struct PageSize {
    pub width: f64,
    pub height: f64,
}

impl PageSize {
    pub fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }

    /// 获取旋转后的尺寸
    pub fn rotated(&self) -> Self {
        Self::new(self.height, self.width)
    }
}

/// PDF拼版处理结构体
pub struct Imposition {
    document: Document,
    filepath: PathBuf,
    page_size: Option<PageSize>,
}

impl Imposition {
    /// 创建新的拼版处理实例
    pub fn new(filepath: PathBuf) -> Result<Self, ImpositionError> {
        let document = Document::load(&filepath)?;
        Ok(Imposition {
            document,
            filepath,
            page_size: None,
        })
    }

    /// 获取页面尺寸
    pub fn get_page_size(&mut self, page_id: ObjectId) -> Result<PageSize, ImpositionError> {
        if let Ok(page_obj) = self.document.get_object(page_id) {
            if let Object::Dictionary(page_dict) = page_obj {
                if let Ok(Object::Array(media_box)) = page_dict.get(b"MediaBox") {
                    if media_box.len() >= 4 {
                        let width = media_box[2].as_float().unwrap_or(595.0)
                            - media_box[0].as_float().unwrap_or(0.0);
                        let height = media_box[3].as_float().unwrap_or(842.0)
                            - media_box[1].as_float().unwrap_or(0.0);
                        return Ok(PageSize::new(width as f64, height as f64));
                    }
                }
            }
        }
        Err(ImpositionError::FailedToGetPageSize)
    }

    /// 创建空白页
    pub fn create_blank_page(&mut self, size: PageSize) -> Result<ObjectId, ImpositionError> {
        let page_id = self.document.new_object_id();

        let page_obj = lopdf::dictionary! {
            "Type" => "Page",
            "MediaBox" => vec![0.into(), 0.into(), size.width.into(), size.height.into()],
            "Contents" => Object::Array(vec![]), // 空内容
        };

        self.document
            .objects
            .insert(page_id, Object::Dictionary(page_obj));
        Ok(page_id)
    }

    /// 计算拼版顺序
    pub fn calculate_booklet_order(
        &self,
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
                    let a = total_pages - 1 - base;
                    let b = base;
                    let c = base + 1;
                    let d = total_pages - 2 - base;
                    order.push((a, b, c, d));
                }
                (ReadingDirection::LeftToRight, FlipDirection::LongEdge) => {
                    let a = base;
                    let b = total_pages - 1 - base;
                    let c = total_pages - 2 - base;
                    let d = base + 1;
                    order.push((a, b, c, d));
                }
                (ReadingDirection::RightToLeft, FlipDirection::ShortEdge) => {
                    let a = base;
                    let b = total_pages - 1 - base;
                    let c = total_pages - 2 - base;
                    let d = base + 1;
                    order.push((a, b, c, d));
                }
                (ReadingDirection::RightToLeft, FlipDirection::LongEdge) => {
                    let a = total_pages - 1 - base;
                    let b = base;
                    let c = base + 1;
                    let d = total_pages - 2 - base;
                    order.push((a, b, c, d));
                }
            }
        }

        order
    }

    /// 计算目标页面尺寸
    pub fn calculate_target_size(
        &self,
        original_size: PageSize,
        flip_direction: FlipDirection,
    ) -> PageSize {
        match flip_direction {
            FlipDirection::ShortEdge => original_size.rotated(),
            FlipDirection::LongEdge => original_size,
        }
    }

    /// 创建拼版页面内容
    pub fn create_imposed_page_content(
        &mut self,
        new_doc: &mut Document,
        pages: &[ObjectId],
        indices: [usize; 4],
        original_size: PageSize,
        _target_size: PageSize,
        flip_direction: FlipDirection,
    ) -> Result<(Dictionary, String), ImpositionError> {
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

        // 处理每个页面
        for (i, &page_idx) in indices.iter().enumerate() {
            if page_idx >= pages.len() {
                continue; // 跳过不存在的页面
            }

            let orig_page_id = pages[page_idx];
            let (x, y) = positions[i];

            // 创建 XObject 引用
            let xobj_name = format!("X{}", i);
            let xobject_id = new_doc.new_object_id();

            // 复制原页面对象作为 XObject
            if let Ok(orig_obj) = self.document.get_object(orig_page_id) {
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

                    // 复制原页面的内容流
                    if let Ok(contents) = dict.get(b"Contents") {
                        dict.set("Contents", contents.clone());
                    }

                    // 复制原页面的资源
                    if let Ok(resources) = dict.get(b"Resources") {
                        dict.set("Resources", resources.clone());
                    }
                }

                new_doc.objects.insert(xobject_id, xobj);
                xobject_resources.set(xobj_name.as_bytes(), Object::Reference(xobject_id));

                // 添加到内容流
                content.push(format!("q 1 0 0 1 {} {} cm /{} Do Q\n", x, y, xobj_name));
            }
        }

        // 创建最终的资源字典
        let mut final_resources = Dictionary::new();
        final_resources.set("XObject", Object::Dictionary(xobject_resources));

        Ok((final_resources, content.join("")))
    }

    /// 完成文档处理
    pub fn finalize_document(
        &mut self,
        doc: &mut Document,
        page_ids: Vec<ObjectId>,
    ) -> Result<(), ImpositionError> {
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

        // 设置文档信息
        let info_id = doc.new_object_id();
        let info_dict = lopdf::dictionary! {
            "Producer" => "Bookify-rs",
            "Creator" => "Bookify-rs PDF Imposition Tool",
            "CreationDate" => chrono::Utc::now().format("D:%Y%m%d%H%M%S").to_string(),
        };
        doc.objects.insert(info_id, Object::Dictionary(info_dict));
        doc.trailer.set("Info", Object::Reference(info_id));

        Ok(())
    }

    /// 执行拼版处理
    pub fn impose(&mut self, args: &Cli) -> Result<(), ImpositionError> {
        // 1. 获取原始页面数量和尺寸
        let original_pages: Vec<_> = self.document.get_pages().values().cloned().collect();
        let _original_page_count = original_pages.len();

        // 获取第一页的尺寸作为模板
        let original_size = self.get_page_size(original_pages[0])?;
        self.page_size = Some(original_size);

        // 2. 自动补齐页面到目标数量
        let mut extended_pages = original_pages.clone();
        while extended_pages.len() < args.pages {
            let blank_page_id = self.create_blank_page(original_size)?;
            extended_pages.push(blank_page_id);
        }

        // 3. 计算拼版顺序
        let booklet_order =
            self.calculate_booklet_order(args.pages, args.reading_direction, args.flip_direction);

        // 4. 计算目标页面尺寸
        let target_size = self.calculate_target_size(original_size, args.flip_direction);

        // 5. 创建新文档
        let mut new_doc = Document::with_version("1.5");
        let mut new_pages = Vec::new();

        for (_sheet_idx, &(a, b, c, d)) in booklet_order.iter().enumerate() {
            // 创建页面ID
            let page_id = new_doc.new_object_id();

            // 创建页面内容
            let (resources, content) = self.create_imposed_page_content(
                &mut new_doc,
                &extended_pages,
                [a, b, c, d],
                original_size,
                target_size,
                args.flip_direction,
            )?;

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

        // 6. 构建页面树和文档结构
        self.finalize_document(&mut new_doc, new_pages)?;

        // 7. 更新文档
        self.document = new_doc;
        Ok(())
    }

    /// 保存文档
    pub fn save(&mut self, output: Option<PathBuf>) -> Result<(), ImpositionError> {
        let output_path = match output {
            Some(path) => path,
            None => self.filepath.with_extension("imposed.pdf"),
        };
        self.document.save(&output_path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tempfile::NamedTempFile;

    #[test]
    fn test_page_size_creation() {
        let size = PageSize::new(595.0, 842.0);
        assert_eq!(size.width, 595.0);
        assert_eq!(size.height, 842.0);

        let rotated = size.rotated();
        assert_eq!(rotated.width, 842.0);
        assert_eq!(rotated.height, 595.0);
    }

    #[test]
    fn test_calculate_booklet_order() {
        let imposition = Imposition::new(PathBuf::from("test.pdf")).unwrap();

        // 测试16页文档的拼版顺序
        let order = imposition.calculate_booklet_order(
            16,
            ReadingDirection::LeftToRight,
            FlipDirection::ShortEdge,
        );

        assert_eq!(order.len(), 4); // 16页需要4张纸

        // 验证第一张纸的页面顺序
        let (a, b, c, d) = order[0];
        assert_eq!(a, 15); // 最后一页
        assert_eq!(b, 0); // 第一页
        assert_eq!(c, 1); // 第二页
        assert_eq!(d, 14); // 倒数第二页
    }

    #[test]
    fn test_calculate_target_size() {
        let imposition = Imposition::new(PathBuf::from("test.pdf")).unwrap();
        let original_size = PageSize::new(595.0, 842.0);

        // 测试短边翻转
        let short_edge_size =
            imposition.calculate_target_size(original_size, FlipDirection::ShortEdge);
        assert_eq!(short_edge_size.width, 842.0);
        assert_eq!(short_edge_size.height, 595.0);

        // 测试长边翻转
        let long_edge_size =
            imposition.calculate_target_size(original_size, FlipDirection::LongEdge);
        assert_eq!(long_edge_size.width, 595.0);
        assert_eq!(long_edge_size.height, 842.0);
    }

    #[test]
    fn test_create_blank_page() -> Result<(), ImpositionError> {
        let mut imposition = Imposition::new(PathBuf::from("test.pdf"))?;
        let size = PageSize::new(595.0, 842.0);

        let page_id = imposition.create_blank_page(size)?;

        // 验证页面对象是否存在
        let page_obj = imposition.document.get_object(page_id)?;
        if let lopdf::Object::Dictionary(dict) = page_obj {
            assert_eq!(dict.get(b"Type").unwrap().as_name().unwrap(), b"Page");

            // 验证页面尺寸
            if let Object::Array(media_box) = dict.get(b"MediaBox").unwrap() {
                assert_eq!(media_box[2].as_float().unwrap(), 595.0);
                assert_eq!(media_box[3].as_float().unwrap(), 842.0);
            }
        }

        Ok(())
    }

    fn create_test_pdf(path: &PathBuf) -> Result<(), ImpositionError> {
        let mut doc = lopdf::Document::with_version("1.5");
        let mut imposition = Imposition {
            document: doc,
            filepath: path.clone(),
            page_size: Some(PageSize::new(595.0, 842.0)),
        };

        // 添加一些测试页面
        for _ in 0..4 {
            imposition.create_blank_page(PageSize::new(595.0, 842.0))?;
        }

        imposition.document.save(path)?;
        Ok(())
    }

    #[test]
    fn test_imposition_with_sample_pdf() -> Result<(), ImpositionError> {
        // 创建一个临时的测试PDF文件
        let temp_file = NamedTempFile::new().unwrap();
        let input_path = temp_file.path().to_path_buf();

        // 创建一个简单的PDF文档用于测试
        create_test_pdf(&input_path)?;

        // 创建Imposition实例
        let mut imposition = Imposition::new(input_path.clone())?;

        // 设置测试参数
        let args = Cli {
            input: input_path.clone(),
            output: Some(input_path.with_extension("imposed.pdf")),
            pages: 16,
            reading_direction: ReadingDirection::LeftToRight,
            flip_direction: FlipDirection::ShortEdge,
        };

        // 执行拼版
        imposition.impose(&args)?;

        // 保存结果
        imposition.save(None)?;

        Ok(())
    }
}

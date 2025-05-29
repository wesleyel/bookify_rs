use crate::{
    args::{Cli, FlipDirection, Method, ReadingDirection},
    error::ImpositionError,
};
use lopdf::{dictionary, Dictionary, Document, Object, ObjectId, Stream};
use std::path::{Path, PathBuf};

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

        self.document.objects.insert(page_id, Object::Dictionary(page_obj));
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
        target_size: PageSize,
        flip_direction: FlipDirection,
    ) -> Result<(Dictionary, String), ImpositionError> {
        // TODO: 实现页面内容创建逻辑
        // 这部分需要根据具体的PDF处理需求来实现
        Err(ImpositionError::FailedToCreateImpositionContent)
    }

    /// 完成文档处理
    pub fn finalize_document(&mut self, doc: &mut Document, page_ids: Vec<ObjectId>) -> Result<(), ImpositionError> {
        // TODO: 实现文档完成处理逻辑
        // 这部分需要根据具体的PDF处理需求来实现
        Err(ImpositionError::FailedToFinalizeDocument)
    }

    /// 执行拼版处理
    pub fn impose(&mut self, args: &Cli) -> Result<(), ImpositionError> {
        // 1. 获取原始页面数量和尺寸
        let original_pages: Vec<_> = self.document.get_pages().values().cloned().collect();
        let original_page_count = original_pages.len();

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
        let booklet_order = self.calculate_booklet_order(
            args.pages,
            args.reading_direction,
            args.flip_direction,
        );

        // 4. 计算目标页面尺寸
        let target_size = self.calculate_target_size(original_size, args.flip_direction);

        // 5. 创建新文档
        let mut new_doc = Document::with_version("1.5");
        let mut new_pages = Vec::new();

        for (sheet_idx, &(a, b, c, d)) in booklet_order.iter().enumerate() {
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
            let content_obj = Object::Stream(Stream::new(Dictionary::new(), content.as_bytes().to_vec()));
            new_doc.objects.insert(content_id, content_obj);

            // 创建页面对象
            let page_obj = lopdf::dictionary! {
                "Type" => "Page",
                "MediaBox" => vec![0.into(), 0.into(), target_size.width.into(), target_size.height.into()],
                "Contents" => Object::Reference(content_id),
                "Resources" => Object::Dictionary(resources),
            };

            new_doc.objects.insert(page_id, Object::Dictionary(page_obj));
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

use crate::{args::Method, error::ImpositionError};
use std::path::{Path, PathBuf};

use lopdf::{dictionary, Document, Object, ObjectId, Stream};

#[derive(Debug, Clone)]
struct Imposition {
    pub document: Document,
    pub filepath: PathBuf,
}

impl Imposition {
    pub fn new(filepath: PathBuf) -> Result<Self, ImpositionError> {
        let document = Document::load(&filepath)?;
        Ok(Imposition { document, filepath })
    }

    pub fn impose(&mut self, method: Method) -> Result<(), ImpositionError> {
        match method {
            Method::Booklet => self.impose_booklet()?,
            Method::DoubleSided => self.impose_double_sided()?,
        }
        Ok(())
    }

    fn impose_booklet(&mut self) -> Result<(), ImpositionError> {
        let total_pages = self.document.get_pages().len();
        let padded_pages = (total_pages + 3) / 4 * 4; // 向上取整到4的倍数

        let mut new_document = Document::new();

        for i in 0..(padded_pages / 4) {
            // 处理每张纸的正面
            let front_right = i + 1; // 第1, 5, 9...页
            let front_left = padded_pages - i; // 第n, n-4, n-8...页

            // 处理每张纸的反面
            let back_left = i + 2; // 第2, 6, 10...页
            let back_right = padded_pages - i - 1; // 第n-1, n-5, n-9...页

            // 创建新页面并添加到新文档
            let mut page_ids = Vec::new();
            if front_right <= total_pages {
                page_ids.push(self.document.get_page(front_right).unwrap());
            }
            if front_left <= total_pages {
                page_ids.push(self.document.get_page(front_left).unwrap());
            }
            if back_left <= total_pages {
                page_ids.push(self.document.get_page(back_left).unwrap());
            }
            if back_right <= total_pages {
                page_ids.push(self.document.get_page(back_right).unwrap());
            }
            let new_page_id = new_document.new_page();
            let mut page_dict = dictionary! {
                "Type" => "Page",
                "MediaBox" => vec![0.0, 0.0, 595.0, 842.0], // A4 size
                "Contents" => Object::Stream(Stream::new(vec![], None)),
                "Resources" => dictionary! {},
            };
            page_dict.set("Parent", Object::Reference(new_document.get_page_tree_id()));
            new_document.objects.insert(new_page_id, Object::Dictionary(page_dict));
            for page_id in page_ids {
                if let Object::Reference(ref_id) = page_id {
                    new_document.add_page(new_page_id, ref_id);
                }
            }
        }

        self.document = new_document;
        Ok(())
    }

    fn impose_double_sided(&mut self) -> Result<(), ImpositionError> {
        // Placeholder for double-sided imposition logic
        // This should rearrange the pages of the PDF for double-sided printing
        // For now, we will just print a message
        println!("Imposing PDF in Double-Sided format...");
        Ok(())
    }

    pub fn save(&mut self, output: Option<PathBuf>) -> Result<(), ImpositionError> {
        let output_path = match output {
            Some(path) => path,
            None => self.filepath.with_extension("imposed.pdf"),
        };
        self.document.save(&output_path)?;
        Ok(())
    }
}

use clap::ValueEnum;

/// Defines booklet imposition layout type.
/// This enum specifies the total number of booklet pages placed on each physical sheet (front and back).
#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum LayoutType {
    /// Place 4 booklet pages on each physical sheet (2 pages per side).
    /// Suitable for printing A5 booklets on A4 paper.
    #[value(name = "two-up")]
    TwoUp,
    /// Place 8 booklet pages on each physical sheet (4 pages per side).
    /// Suitable for printing A6 booklets on A4 paper or A5 booklets on A3 paper.
    #[value(name = "four-up")]
    FourUp,
}

/// 根据给定的页数 n 和排版布局类型，生成小册子排版的页面顺序列表。
///
/// 小册子的总页数必须是 `LayoutType` 所定义的页面数的倍数。
/// 如果输入的 `n` 不是指定倍数的倍数，将向上取整到最近的倍数，
/// 多余的页面将用 0（空白页）填充。
///
/// # 参数
/// * `n` - 小册子的实际页数。
/// * `layout` - 指定每张物理纸张上的页面排版方式。
///
/// # 返回
/// `Vec<u32>` - 按照打印顺序（从左到右，从上到下，先正面后反面）
///              排列的页面序号列表。0 代表空白页。
///
/// # 示例
/// ```
/// use bookify_rs::calc::{generate_booklet_imposition, LayoutType};
///
/// let imposition_4up = generate_booklet_imposition(16, LayoutType::FourUp);
/// assert_eq!(imposition_4up, vec![16, 1, 14, 3, 2, 15, 4, 13, 12, 5, 10, 7, 6, 11, 8, 9]);
///
/// let imposition_2up = generate_booklet_imposition(8, LayoutType::TwoUp);
/// assert_eq!(imposition_2up, vec![8, 1, 2, 7, 6, 3, 4, 5]);
/// ```
pub fn generate_booklet_imposition(n: u32, layout: LayoutType) -> Vec<u32> {
    // 1. 处理特殊情况：页数为 0
    if n == 0 {
        return Vec::new();
    }

    // 2. 根据布局类型确定每张物理纸张上的总页面数
    let pages_per_physical_sheet: u32 = match layout {
        LayoutType::TwoUp => 4,
        LayoutType::FourUp => 8,
    };

    // 3. 确定小册子实际需要排版的总页数，必须是 pages_per_physical_sheet 的倍数
    let total_pages = n.div_ceil(pages_per_physical_sheet) * pages_per_physical_sheet;

    // 4. 初始化结果列表
    let mut imposition_list: Vec<u32> = Vec::new();

    // 5. 遍历每张物理纸张
    let num_physical_sheets = total_pages / pages_per_physical_sheet;

    for k in 0..num_physical_sheets {
        match layout {
            LayoutType::FourUp => {
                // 每面 4 页 (Total 8 pages per sheet)
                // SIDE A (左上, 右上, 左下, 右下)
                let side_a_pages = [
                    total_pages - (4 * k),     // 最外侧的背面页
                    1 + (4 * k),               // 最外侧的正面页
                    total_pages - (4 * k + 2), // 次外侧的背面页
                    3 + (4 * k),               // 次外侧的正面页
                ];
                imposition_list.extend_from_slice(&side_a_pages);

                // SIDE B (左上, 右上, 左下, 右下)
                let side_b_pages = [
                    2 + (4 * k),               // 次外侧的正面页 (内侧)
                    total_pages - (4 * k + 1), // 次外侧的背面页 (内侧)
                    4 + (4 * k),               // 最内侧的正面页
                    total_pages - (4 * k + 3), // 最内侧的背面页
                ];
                imposition_list.extend_from_slice(&side_b_pages);
            }
            LayoutType::TwoUp => {
                // 每面 2 页 (Total 4 pages per sheet)
                // SIDE A (左, 右)
                let side_a_pages = [
                    total_pages - (2 * k), // 外侧背面页
                    1 + (2 * k),           // 外侧正面页
                ];
                imposition_list.extend_from_slice(&side_a_pages);

                // SIDE B (左, 右)
                let side_b_pages = [
                    2 + (2 * k),               // 内侧正面页
                    total_pages - (2 * k + 1), // 内侧背面页
                ];
                imposition_list.extend_from_slice(&side_b_pages);
            }
        }
    }

    // 6. 处理空白页：将大于原始页数 n 的页面替换为 0
    let final_imposition_list: Vec<u32> = imposition_list
        .into_iter()
        .map(|p| if p > n { 0 } else { p })
        .collect();

    final_imposition_list
}

#[cfg(test)]
mod tests {
    use super::*; // 导入父模块中的所有项

    #[test]
    fn test_eight_up_n_0_pages() {
        assert_eq!(generate_booklet_imposition(0, LayoutType::FourUp), vec![]);
    }

    #[test]
    fn test_eight_up_n_16_pages() {
        let expected = vec![
            16, 1, 14, 3, // Set 1, Side A
            2, 15, 4, 13, // Set 1, Side B
            12, 5, 10, 7, // Set 2, Side A
            6, 11, 8, 9, // Set 2, Side B
        ];
        assert_eq!(
            generate_booklet_imposition(16, LayoutType::FourUp),
            expected
        );
    }

    #[test]
    fn test_eight_up_n_1_page() {
        let expected = vec![0, 1, 0, 0, 0, 0, 0, 0];
        assert_eq!(generate_booklet_imposition(1, LayoutType::FourUp), expected);
    }

    #[test]
    fn test_eight_up_n_8_pages() {
        let expected = vec![8, 1, 6, 3, 2, 7, 4, 5];
        assert_eq!(generate_booklet_imposition(8, LayoutType::FourUp), expected);
    }

    #[test]
    fn test_eight_up_n_5_pages() {
        let expected = vec![0, 1, 0, 3, 2, 0, 4, 5];
        assert_eq!(generate_booklet_imposition(5, LayoutType::FourUp), expected);
    }

    // --- FourUp Layout Tests ---

    #[test]
    fn test_four_up_n_0_pages() {
        assert_eq!(generate_booklet_imposition(0, LayoutType::TwoUp), vec![]);
    }

    #[test]
    fn test_four_up_n_4_pages() {
        // 4 页的完整小册子
        // Sheet 1, Side A: 4, 1
        // Sheet 1, Side B: 2, 3
        let expected = vec![4, 1, 2, 3];
        assert_eq!(generate_booklet_imposition(4, LayoutType::TwoUp), expected);
    }

    #[test]
    fn test_four_up_n_8_pages() {
        // 8 页的完整小册子
        // Sheet 1, Side A: 8, 1
        // Sheet 1, Side B: 2, 7
        // Sheet 2, Side A: 6, 3
        // Sheet 2, Side B: 4, 5
        let expected = vec![8, 1, 2, 7, 6, 3, 4, 5];
        assert_eq!(generate_booklet_imposition(8, LayoutType::TwoUp), expected);
    }

    #[test]
    fn test_four_up_n_1_page() {
        // 1 页的小册子，总页数应为 4
        // 基于 n=4 的结果 [4,1,2,3]，将 >1 的替换为 0
        let expected = vec![0, 1, 0, 0];
        assert_eq!(generate_booklet_imposition(1, LayoutType::TwoUp), expected);
    }

    #[test]
    fn test_four_up_n_6_pages() {
        // 6 页的小册子，总页数应为 8
        // 基于 n=8 的结果 [8,1,2,7,6,3,4,5]，将 >6 的替换为 0
        let expected = vec![
            0, 1, 2, 0, // 8->0, 7->0
            6, 3, 4, 5,
        ];
        assert_eq!(generate_booklet_imposition(6, LayoutType::TwoUp), expected);
    }
}

use crate::args::{FlipType, LayoutType, OddEven};

/// Generates a booklet imposition sequence based on page count and layout type.
///
/// The total page count must be a multiple of the pages per sheet defined by `LayoutType`.
/// If the input `n` is not a multiple, it will be rounded up to the nearest multiple,
/// with blank pages (represented by 0) added as needed.
///
/// # Parameters
/// * `n` - Total number of pages in the booklet
/// * `layout` - Layout type defining pages per sheet
///
/// # Returns
/// `Vec<u32>` - Page sequence ordered for printing (left-to-right, top-to-bottom, front then back).
///             0 represents a blank page.
///
/// # Example
/// ```
/// use bookify_rs::{args::LayoutType, calc::generate_booklet_imposition};
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

/// Generates a page sequence for double-sided printing based on flip type and page selection.
///
/// The function handles different printing scenarios by combining flip type (RR, NN, RN, NR)
/// with page selection (odd or even pages). It automatically adds a blank page (0) when needed
/// for even page sequences with odd total page count.
///
/// # Parameters
/// * `total_pages` - Total number of pages in the document
/// * `flip_type` - Page flipping direction (RR: both odd and even pages, NN: no flip, etc.)
/// * `odd_even` - Page selection (Odd: 1,3,5... or Even: 2,4,6...)
///
/// # Returns
/// `Vec<u32>` - Page sequence ordered for double-sided printing.
///                                     0 represents a blank page.
///
/// # Example
/// ```
/// use bookify_rs::{args::{FlipType, OddEven}, calc::generate_double_sided_order};
///
/// // Generate odd pages for both odd and even pages flipping
/// let odd_pages = generate_double_sided_order(5, FlipType::RR, OddEven::Odd);
/// assert_eq!(odd_pages, vec![5, 3, 1]);
///
/// // Generate even pages for both odd and even pages flipping
/// let even_pages = generate_double_sided_order(5, FlipType::RR, OddEven::Even);
/// assert_eq!(even_pages, vec![0, 4, 2]);
///
/// ```
pub fn generate_double_sided_order(
    total_pages: u32,
    flip_type: FlipType,
    odd_even: OddEven,
) -> Vec<u32> {
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

    pages
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

    // --- Double-sided Order Tests ---

    #[test]
    fn test_double_sided_zero_pages() {
        // Test with zero pages
        let result = generate_double_sided_order(0, FlipType::RR, OddEven::Odd);
        assert_eq!(result, vec![]);
    }

    #[test]
    fn test_double_sided_single_page() {
        // Test with single page
        let result = generate_double_sided_order(1, FlipType::RR, OddEven::Odd);
        assert_eq!(result, vec![1]);

        let result = generate_double_sided_order(1, FlipType::RR, OddEven::Even);
        assert_eq!(result, vec![0]);
    }
}

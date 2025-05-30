use bookify_rs::{
    args::{
        BaseOptions, BookletOptions, DoubleSidedOptions, FlipType, LayoutType,
        OddEven,
    },
    imposition::PdfImposer,
};
use std::fs;
use std::path::PathBuf;

const DELETE_RESULT: bool = false;

#[test]
fn test_booklet_imposition() {
    let input_path = PathBuf::from("tests/sample.pdf");
    let output_path = PathBuf::from("tests/output/booklet-test.pdf");

    // 确保输出目录存在
    fs::create_dir_all("tests/output").unwrap();

    // 创建小册子选项
    let opts = BookletOptions {
        base: BaseOptions {
            input: input_path.clone(),
            output: Some(output_path.clone()),
            temp: false,
        },
        layout: LayoutType::TwoUp,
    };

    // 执行小册子排版
    let mut imposer = PdfImposer::new(input_path).unwrap();
    imposer.export_booklet(opts.layout).unwrap();
    imposer.save(output_path.clone()).unwrap();

    // 验证输出文件存在
    assert!(output_path.exists());

    // 清理测试文件
    if DELETE_RESULT {
        fs::remove_file(output_path).unwrap();
    }
}

#[test]
fn test_double_sided_imposition() {
    let input_path = PathBuf::from("tests/sample.pdf");
    let output_path = PathBuf::from("tests/output/double-sided-test.pdf");

    // 确保输出目录存在
    fs::create_dir_all("tests/output").unwrap();

    // 创建双面打印选项
    let opts = DoubleSidedOptions {
        base: BaseOptions {
            input: input_path.clone(),
            output: Some(output_path.clone()),
            temp: false,
        },
        flip_type: FlipType::RR,
        odd_even: OddEven::Odd,
    };

    // 执行双面打印排版
    let mut imposer = PdfImposer::new(input_path).unwrap();
    imposer
        .export_double_sided(opts.flip_type, opts.odd_even)
        .unwrap();
    imposer.save(output_path.clone()).unwrap();

    // 验证输出文件存在
    assert!(output_path.exists());

    // 清理测试文件
    if DELETE_RESULT {
        fs::remove_file(output_path).unwrap();
    }
}

#[test]
fn test_temp_output() {
    let input_path = PathBuf::from("tests/sample.pdf");

    // 创建临时文件选项
    let opts = BookletOptions {
        base: BaseOptions {
            input: input_path.clone(),
            output: None,
            temp: true,
        },
        layout: LayoutType::TwoUp,
    };

    // 执行小册子排版并获取临时文件路径
    let mut imposer = PdfImposer::new(input_path).unwrap();
    imposer.export_booklet(opts.layout).unwrap();

    // 创建临时文件
    let temp_file = tempfile::Builder::new()
        .prefix("booklet-")
        .suffix(".pdf")
        .tempfile()
        .unwrap();
    let temp_path = temp_file.path().to_path_buf();

    // 保存到临时文件
    imposer.save(temp_path.clone()).unwrap();

    // 验证临时文件存在
    assert!(temp_path.exists());

    // 临时文件会在作用域结束时自动删除
}

#[test]
fn test_custom_output_path() {
    let input_path = PathBuf::from("tests/sample.pdf");
    let custom_output = PathBuf::from("tests/output/custom-test.pdf");

    // 确保输出目录存在
    fs::create_dir_all("tests/output").unwrap();

    // 创建自定义输出路径选项
    let opts = BookletOptions {
        base: BaseOptions {
            input: input_path.clone(),
            output: Some(custom_output.clone()),
            temp: false,
        },
        layout: LayoutType::TwoUp,
    };

    // 执行小册子排版
    let mut imposer = PdfImposer::new(input_path).unwrap();
    imposer.export_booklet(opts.layout).unwrap();
    imposer.save(custom_output.clone()).unwrap();

    // 验证自定义输出文件存在
    assert!(custom_output.exists());

    // 清理测试文件
    if DELETE_RESULT {
        fs::remove_file(custom_output).unwrap();
    }
}

use bookify_rs::{
    args::{BaseOptions, BookletOptions, DoubleSidedOptions, FlipType, LayoutType, OddEven},
    imposition::PdfImposer,
};
use std::fs;
use std::path::PathBuf;

const DELETE_RESULT: bool = false;
const INPUT_PATH: &str = "tests/sample.pdf";

#[test]
fn test_booklet_imposition() {
    let input_path = PathBuf::from(INPUT_PATH);
    let output_path = PathBuf::from("tests/output/booklet-test.pdf");

    // Ensure output directory exists
    fs::create_dir_all("tests/output").unwrap();

    // Create booklet options
    let opts = BookletOptions {
        base: BaseOptions {
            input: input_path.clone(),
            output: Some(output_path.clone()),
            temp: false,
        },
        layout: LayoutType::TwoUp,
    };

    // Execute booklet imposition
    let mut imposer = PdfImposer::new(input_path).unwrap();
    imposer.export_booklet(opts.layout).unwrap();
    imposer.save(output_path.clone()).unwrap();

    // Verify output file exists
    assert!(output_path.exists());

    // Clean up test files
    if DELETE_RESULT {
        fs::remove_file(output_path).unwrap();
    }
}

#[test]
fn test_double_sided_imposition_odd() {
    let input_path = PathBuf::from(INPUT_PATH);
    let output_path = PathBuf::from("tests/output/double-sided-test-odd.pdf");

    // Ensure output directory exists
    fs::create_dir_all("tests/output").unwrap();

    // Create duplex printing options for odd pages
    let opts = DoubleSidedOptions {
        base: BaseOptions {
            input: input_path.clone(),
            output: Some(output_path.clone()),
            temp: false,
        },
        flip_type: FlipType::RR,
        odd_even: OddEven::Odd,
    };

    // Execute duplex printing imposition
    let mut imposer = PdfImposer::new(input_path.clone()).unwrap();
    imposer
        .export_double_sided(opts.flip_type, opts.odd_even)
        .unwrap();
    imposer.save(output_path.clone()).unwrap();

    // Verify output file exists
    assert!(output_path.exists());

    // Clean up test files
    if DELETE_RESULT {
        fs::remove_file(output_path).unwrap();
    }
}

#[test]
fn test_double_sided_imposition_even() {
    let input_path = PathBuf::from(INPUT_PATH);
    let output_path = PathBuf::from("tests/output/double-sided-test-even.pdf");

    // Ensure output directory exists
    fs::create_dir_all("tests/output").unwrap();

    // Create duplex printing options for even pages
    let opts = DoubleSidedOptions {
        base: BaseOptions {
            input: input_path.clone(),
            output: Some(output_path.clone()),
            temp: false,
        },
        flip_type: FlipType::RR,
        odd_even: OddEven::Even,
    };

    // Execute duplex printing imposition
    let mut imposer = PdfImposer::new(input_path).unwrap();
    imposer
        .export_double_sided(opts.flip_type, opts.odd_even)
        .unwrap();
    imposer.save(output_path.clone()).unwrap();

    // Verify output file exists
    assert!(output_path.exists());

    // Clean up test files
    if DELETE_RESULT {
        fs::remove_file(output_path).unwrap();
    }
}

#[test]
fn test_temp_output() {
    let input_path = PathBuf::from(INPUT_PATH);

    // Create temporary file options
    let opts = BookletOptions {
        base: BaseOptions {
            input: input_path.clone(),
            output: None,
            temp: true,
        },
        layout: LayoutType::TwoUp,
    };

    // Execute booklet imposition and get temporary file path
    let mut imposer = PdfImposer::new(input_path).unwrap();
    imposer.export_booklet(opts.layout).unwrap();

    // Create temporary file
    let temp_file = tempfile::Builder::new()
        .prefix("booklet-")
        .suffix(".pdf")
        .tempfile()
        .unwrap();
    let temp_path = temp_file.path().to_path_buf();

    // Save to temporary file
    imposer.save(temp_path.clone()).unwrap();

    // Verify temporary file exists
    assert!(temp_path.exists());

    // Temporary file will be automatically deleted when the scope ends
}

#[test]
fn test_custom_output_path() {
    let input_path = PathBuf::from(INPUT_PATH);
    let custom_output = PathBuf::from("tests/output/custom-test.pdf");

    // Ensure output directory exists
    fs::create_dir_all("tests/output").unwrap();

    // Create custom output path options
    let opts = BookletOptions {
        base: BaseOptions {
            input: input_path.clone(),
            output: Some(custom_output.clone()),
            temp: false,
        },
        layout: LayoutType::TwoUp,
    };

    // Execute booklet imposition
    let mut imposer = PdfImposer::new(input_path).unwrap();
    imposer.export_booklet(opts.layout).unwrap();
    imposer.save(custom_output.clone()).unwrap();

    // Verify custom output file exists
    assert!(custom_output.exists());

    // Clean up test files
    if DELETE_RESULT {
        fs::remove_file(custom_output).unwrap();
    }
}

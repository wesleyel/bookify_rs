use bookify_rs::{args::Cli, error::ImpositionError, imposition::rearrange_pdf_pages};
use clap::Parser;

fn main() -> Result<(), ImpositionError> {
    let args = Cli::parse();

    let input_path = args.input.clone();
    let output_path = args
        .output
        .clone()
        .unwrap_or(input_path.with_extension(format!("{:?}.pdf", args.layout)));
    // 创建拼版处理实例
    rearrange_pdf_pages(input_path, output_path.clone(), args.layout)?;

    println!("拼版完成，输出文件：{}", output_path.display());
    Ok(())
}

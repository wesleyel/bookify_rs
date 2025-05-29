use bookify_rs::{args::Cli, error::ImpositionError, imposition::process_pdf};
use clap::Parser;

fn main() -> Result<(), ImpositionError> {
    let args = Cli::parse();

    // 创建拼版处理实例
    process_pdf(&args)?;

    println!("拼版完成，输出文件：{}", args.output.unwrap().display());
    Ok(())
}

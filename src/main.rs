use bookify_rs::{args::Cli, error::ImpositionError, imposition::Imposition};
use clap::Parser;

fn main() -> Result<(), ImpositionError> {
    let args = Cli::parse();

    // 创建拼版处理实例
    let mut imposition = Imposition::new(args.input.clone())?;

    // 执行拼版处理
    imposition.impose(&args)?;

    // 保存处理后的文档
    let output_path = args.output.clone().unwrap_or_else(|| args.input.with_extension("imposed.pdf"));
    imposition.save(args.output)?;

    println!("拼版完成，输出文件：{}", output_path.display());
    Ok(())
}

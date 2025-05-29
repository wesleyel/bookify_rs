use bookify_rs::{
    args::{Cli, Commands},
    error::ImpositionError,
    imposition::{export_double_sided_pdf, rearrange_pdf_pages},
};
use clap::Parser;
use std::process;
use tempfile::Builder;

fn main() {
    let args = Cli::parse();

    if let Err(e) = match args.command {
        Commands::Booklet(opts) => handle_booklet(opts),
        Commands::DoubleSided(opts) => handle_double_sided(opts),
    } {
        eprintln!("错误：{}", e);
        process::exit(1);
    }
}

/// 处理小册子拼版命令
fn handle_booklet(opts: bookify_rs::args::BookletOptions) -> Result<(), ImpositionError> {
    let input_path = opts.input.clone();
    let output_path = opts
        .output
        .clone()
        .unwrap_or_else(|| input_path.with_extension(format!("booklet-{:?}.pdf", opts.layout)));

    rearrange_pdf_pages(input_path, output_path.clone(), opts.layout)?;

    println!("小册子拼版完成，输出文件：{}", output_path.display());
    Ok(())
}

/// 处理双面打印命令
fn handle_double_sided(opts: bookify_rs::args::DoubleSidedOptions) -> Result<(), ImpositionError> {
    let input_path = opts.input.clone();
    let output_path = if let Some(path) = opts.output.clone() {
        path
    } else {
        Builder::new()
            .prefix(&format!(
                "double-sided-{:?}-{:?}",
                opts.flip_type, opts.odd_even
            ))
            .suffix(".pdf")
            .tempfile()
            .map_err(|e| ImpositionError::Other(format!("创建临时文件失败: {}", e)))?
            .into_temp_path()
            .to_path_buf()
    };

    export_double_sided_pdf(
        input_path,
        output_path.clone(),
        opts.flip_type,
        opts.odd_even,
    )?;

    match opts.output {
        Some(path) => println!(
            "双面打印 {:?} 页生成完成，输出文件：{}",
            opts.odd_even,
            path.display()
        ),
        None => println!("{}", output_path.display()),
    }
    Ok(())
}

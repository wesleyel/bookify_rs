use bookify_rs::{
    args::{BaseOptions, BookletOptions, Cli, Commands, DoubleSidedOptions},
    error::BookifyError,
    imposition::PdfImposer,
};
use clap::Parser;
use std::path::{Path, PathBuf};
use std::process;
use tempfile::Builder;

fn main() {
    let args = Cli::parse();

    if let Err(e) = match args.command {
        Commands::Booklet(opts) => handle_booklet(opts),
        Commands::DoubleSided(opts) => handle_double_sided(opts),
    } {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

/// Generate output path with opts
fn handle_output_path(
    base_opts: &BaseOptions,
    input_path: &Path,
    prefix: &str,
) -> Result<PathBuf, BookifyError> {
    let output_path = if base_opts.temp {
        Builder::new()
            .prefix(prefix)
            .suffix(".pdf")
            .tempfile()
            .map_err(|e| BookifyError::other("Creating temporary file", format!("{}", e)))?
            .into_temp_path()
            .to_path_buf()
    } else {
        base_opts
            .output
            .clone()
            .unwrap_or_else(|| input_path.with_extension(format!("{}.pdf", prefix)))
    };
    Ok(output_path)
}

/// Conditional print result
fn print_output_result(temp: bool, output_path: &Path, message: &str) {
    if temp {
        println!("{}", output_path.display());
    } else {
        println!("{}", message);
    }
}

/// Handle booklet imposition command
fn handle_booklet(opts: BookletOptions) -> Result<(), BookifyError> {
    let input_path = opts.base.input.clone();
    let prefix = format!("booklet-{:?}", opts.layout);
    let output_path = handle_output_path(&opts.base, &input_path, &prefix)?;

    let mut imposer = PdfImposer::new(input_path)?;
    imposer.export_booklet(opts.layout)?;
    imposer.save(output_path.clone())?;

    print_output_result(
        opts.base.temp,
        &output_path,
        &format!(
            "Booklet imposition completed, output file: {}",
            output_path.display()
        ),
    );
    Ok(())
}

/// Handle double-sided printing command
fn handle_double_sided(opts: DoubleSidedOptions) -> Result<(), BookifyError> {
    let input_path = opts.base.input.clone();
    let prefix = format!("double-sided-{:?}-{:?}", opts.flip_type, opts.odd_even);
    let output_path = handle_output_path(&opts.base, &input_path, &prefix)?;

    let mut imposer = PdfImposer::new(input_path)?;
    imposer.export_double_sided(opts.flip_type, opts.odd_even)?;
    imposer.save(output_path.clone())?;

    print_output_result(
        opts.base.temp,
        &output_path,
        &format!(
            "Double-sided printing {:?} pages completed, output file: {}",
            opts.odd_even,
            output_path.display()
        ),
    );
    Ok(())
}

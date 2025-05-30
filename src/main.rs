use bookify_rs::{
    args::{Cli, Commands},
    error::ImpositionError,
    imposition::PdfImposer,
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
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

/// Handle booklet imposition command
fn handle_booklet(opts: bookify_rs::args::BookletOptions) -> Result<(), ImpositionError> {
    let input_path = opts.input.clone();
    let output_path = opts
        .output
        .clone()
        .unwrap_or_else(|| input_path.with_extension(format!("booklet-{:?}.pdf", opts.layout)));

    let mut imposer = PdfImposer::new(input_path)?;
    imposer.export_booklet(opts.layout)?;
    imposer.save(output_path.clone())?;

    println!(
        "Booklet imposition completed, output file: {}",
        output_path.display()
    );
    Ok(())
}

/// Handle double-sided printing command
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
            .map_err(|e| ImpositionError::Other(format!("Failed to create temporary file: {}", e)))?
            .into_temp_path()
            .to_path_buf()
    };

    let mut imposer = PdfImposer::new(input_path)?;
    imposer.export_double_sided(opts.flip_type, opts.odd_even)?;
    imposer.save(output_path.clone())?;

    match opts.output {
        Some(path) => println!(
            "Double-sided printing {:?} pages completed, output file: {}",
            opts.odd_even,
            path.display()
        ),
        None => println!("{}", output_path.display()),
    }
    Ok(())
}

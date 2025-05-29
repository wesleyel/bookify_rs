use std::path::PathBuf;

use clap::{Parser, ValueEnum};

#[derive(Clone, Debug, Parser, ValueEnum)]
pub enum Method {
    /// Impose PDF to Booklet format
    Booklet,
    /// Impose PDF to Booklet format with double-sided printing
    DoubleSided,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// The method to use for imposing the PDF
    #[arg(short, long)]
    pub method: Method,

    /// The input PDF file
    pub file: PathBuf,
}

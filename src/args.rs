use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

/// Flip type
#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum FlipType {
    /// Flip on both odd and even pages
    #[value(name = "rr")]
    RR,
    /// No flip on both odd and even pages
    #[value(name = "nn")]
    NN,
    /// Flip on odd pages, no flip on even pages
    #[value(name = "rn")]
    RN,
    /// Flip on even pages, no flip on odd pages
    #[value(name = "nr")]
    NR,
}

/// Output odd or even pages
#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum OddEven {
    /// Odd pages
    #[value(name = "odd")]
    Odd,
    /// Even pages
    #[value(name = "even")]
    Even,
}

/// Defines booklet imposition layout type.
/// This enum specifies the total number of booklet pages placed on each physical sheet (front and back).
#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum LayoutType {
    /// Place 4 booklet pages on each physical sheet (2 pages per side).
    /// Suitable for printing A5 booklets on A4 paper.
    #[value(name = "two-up")]
    TwoUp,
    /// Place 8 booklet pages on each physical sheet (4 pages per side).
    /// Suitable for printing A6 booklets on A4 paper or A5 booklets on A3 paper.
    #[value(name = "four-up")]
    FourUp,
}

/// Booklet imposition options
#[derive(Debug, Parser)]
pub struct BookletOptions {
    /// Input PDF file
    #[arg(value_hint = clap::ValueHint::FilePath)]
    pub input: PathBuf,

    /// Output PDF file
    #[arg(short, long, value_hint = clap::ValueHint::FilePath)]
    pub output: Option<PathBuf>,

    /// Layout type
    #[arg(long, value_enum, default_value = "four-up")]
    pub layout: LayoutType,
}

/// Double-sided printing options
#[derive(Debug, Parser)]
pub struct DoubleSidedOptions {
    /// Input PDF file
    #[arg(value_hint = clap::ValueHint::FilePath)]
    pub input: PathBuf,

    /// Output PDF file
    #[arg(short, long, value_hint = clap::ValueHint::FilePath)]
    pub output: Option<PathBuf>,

    /// Flip type, default is flip on both odd and even pages
    #[arg(long, value_enum, default_value = "rr")]
    pub flip_type: FlipType,

    /// Output odd or even pages
    #[arg(long, value_enum, default_value = "odd")]
    pub odd_even: OddEven,
}

/// Command line parameters
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// Subcommand
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Booklet imposition: Convert PDF to booklet format suitable for double-sided printing
    #[command(name = "booklet")]
    Booklet(BookletOptions),

    /// Double-sided printing: Convert PDF to format suitable for double-sided printing
    #[command(name = "double-sided")]
    DoubleSided(DoubleSidedOptions),
}

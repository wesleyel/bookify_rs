use bookify_rs::args::Cli;
use clap::Parser;

fn main() {
    let cli = Cli::parse();

    println!("Method: {:?}", cli.method);
    println!("Input file: {:?}", cli.file);
}

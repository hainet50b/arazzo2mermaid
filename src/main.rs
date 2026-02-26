use clap::Parser;
use crate::arazzo::ArazzoDocument;
use crate::renderer::{MermaidFlowchart, Renderer};
use std::error::Error;
use std::io::Read;
use std::{fs, io};

mod arazzo;
mod renderer;

/// Convert Arazzo workflows into Mermaid diagrams.
#[derive(Parser)]
#[command(version)]
struct Arazzo2Mermaid {
    /// Arazzo workflows file to convert to Mermaid diagrams
    file: Option<String>,

    /// Save to specified file
    #[arg(short, long, value_name = "FILE")]
    output: Option<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Arazzo2Mermaid::parse();

    run(&cli)?;

    Ok(())
}

fn run(cli: &Arazzo2Mermaid) -> Result<(), Box<dyn Error>> {
    let mut reader: Box<dyn Read> = match cli.file.as_deref() {
        Some("-") | None => Box::new(io::stdin()),
        Some(path) => Box::new(fs::File::open(path)?),
    };

    let mut content = String::new();
    reader.read_to_string(&mut content)?;

    let arazzo: ArazzoDocument = yaml_serde::from_str(&content)?;
    let mermaid = MermaidFlowchart.render(&arazzo);

    if let Some(file) = cli.output.as_deref() {
        fs::write(file, mermaid)?;
    } else {
        print!("{}", mermaid);
    }

    Ok(())
}

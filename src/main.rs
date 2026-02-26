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

    let reader: Box<dyn Read> = match cli.file.as_deref() {
        Some("-") | None => Box::new(io::stdin()),
        Some(file) => Box::new(fs::File::open(file)?),
    };

    let mermaid = run(reader)?;

    if let Some(file) = cli.output.as_deref() {
        fs::write(file, mermaid)?;
    } else {
        print!("{}", mermaid);
    }

    Ok(())
}

fn run(mut reader: impl Read) -> Result<String, Box<dyn Error>> {
    let mut content = String::new();
    reader.read_to_string(&mut content)?;

    let arazzo: ArazzoDocument = yaml_serde::from_str(&content)?;
    let mermaid = MermaidFlowchart.render(&arazzo);

    Ok(mermaid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn run_read_from_file() {
        let reader = fs::File::open("fixtures/minimal.yml").unwrap();

        run(reader).unwrap();
    }

    #[test]
    fn run_read_from_stdin() {
        let mut reader = fs::File::open("fixtures/minimal.yml").unwrap();

        let mut content = String::new();
        reader.read_to_string(&mut content).unwrap();

        let reader = Cursor::new(content);

        run(reader).unwrap();
    }

    #[test]
    fn run_read_invalid_yaml() {
        let reader = Cursor::new("invalid yaml");

        let actual = run(reader).is_err();

        assert!(actual);
    }
}

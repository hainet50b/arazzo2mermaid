use std::error::Error;
use std::fmt::{self, Display};
use std::io::{self, Read};
use std::{fs, process};

use clap::Parser;

use crate::arazzo::ArazzoDocument;
use crate::renderer::{MermaidFlowchart, Renderer};

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

#[derive(Debug)]
enum Arazzo2MermaidError {
    Io(io::Error),
    Yaml(yaml_serde::Error),
}

impl Display for Arazzo2MermaidError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Arazzo2MermaidError::Io(error) => write!(f, "Failed to read or write file: {}", error),
            Arazzo2MermaidError::Yaml(error) => write!(f, "Failed to parse YAML: {}", error),
        }
    }
}

impl Error for Arazzo2MermaidError {}

impl From<io::Error> for Arazzo2MermaidError {
    fn from(error: io::Error) -> Self {
        Arazzo2MermaidError::Io(error)
    }
}

impl From<yaml_serde::Error> for Arazzo2MermaidError {
    fn from(error: yaml_serde::Error) -> Self {
        Arazzo2MermaidError::Yaml(error)
    }
}

fn main() {
    let cli = Arazzo2Mermaid::parse();

    let reader: Box<dyn Read> = match cli.file.as_deref() {
        Some("-") | None => Box::new(io::stdin()),
        Some(file) => match fs::File::open(file) {
            Ok(file) => Box::new(file),
            Err(error) => {
                eprintln!("{}", Arazzo2MermaidError::Io(error));
                process::exit(1);
            }
        },
    };

    match run(reader) {
        Ok(mermaid) => {
            if let Some(file) = cli.output.as_deref() {
                if let Err(error) = fs::write(file, mermaid) {
                    eprintln!("{}", Arazzo2MermaidError::Io(error));
                    process::exit(1);
                }
            } else {
                print!("{}", mermaid);
            }
        }
        Err(error) => {
            eprintln!("{}", error);
            process::exit(1);
        }
    };
}

fn run(mut reader: impl Read) -> Result<String, Arazzo2MermaidError> {
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

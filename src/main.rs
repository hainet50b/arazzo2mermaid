use std::error::Error;
use std::fmt::{self, Display};
use std::io::{self, Read, Write};
use std::{fs, process};

use base64::prelude::*;
use clap::Parser;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use serde::Serialize;

use crate::renderer::{MermaidFlowchart, Renderer};

mod arazzo;
mod renderer;

/// Convert Arazzo workflows into Mermaid diagrams.
#[derive(Parser)]
#[command(version)]
struct Arazzo2Mermaid {
    /// Arazzo workflows file to convert to Mermaid diagrams
    file: Option<String>,

    /// Input file format to convert
    #[arg(short, long, value_name = "FORMAT", value_enum, default_value_t = Format::Yaml)]
    format: Format,

    /// Save to specified file
    #[arg(short, long, value_name = "FILE")]
    output: Option<String>,

    /// Open in mermaid.live
    #[arg(long, default_value_t = false)]
    live: bool,
}

#[derive(clap::ValueEnum, Clone)]
enum Format {
    Yaml,
    Json,
}

#[derive(Debug)]
enum Arazzo2MermaidError {
    Io(io::Error),
    Yaml(yaml_serde::Error),
    Json(serde_json::Error),
    Deflate(io::Error),
    Open(io::Error),
}

impl Display for Arazzo2MermaidError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Arazzo2MermaidError::Io(error) => write!(f, "Failed to read or write file: {}", error),
            Arazzo2MermaidError::Yaml(error) => write!(f, "Failed to parse YAML: {}", error),
            Arazzo2MermaidError::Json(error) => write!(f, "Failed to parse JSON: {}", error),
            Arazzo2MermaidError::Deflate(error) => {
                write!(f, "Failed to compress for mermaid.live: {}", error)
            }
            Arazzo2MermaidError::Open(error) => write!(f, "Failed to open browser: {}", error),
        }
    }
}

impl Error for Arazzo2MermaidError {}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MermaidLive {
    code: String,
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

    match run(reader, &cli.format) {
        Ok(mermaid) => {
            if cli.live {
                if let Err(error) = open_mermaid_live(&mermaid) {
                    eprintln!("{}", error);
                    process::exit(1);
                }
            } else if let Some(file) = cli.output.as_deref() {
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

fn run(mut reader: impl Read, format: &Format) -> Result<String, Arazzo2MermaidError> {
    let mut content = String::new();
    reader
        .read_to_string(&mut content)
        .map_err(Arazzo2MermaidError::Io)?;

    let arazzo = match format {
        Format::Yaml => yaml_serde::from_str(&content).map_err(Arazzo2MermaidError::Yaml)?,
        Format::Json => serde_json::from_str(&content).map_err(Arazzo2MermaidError::Json)?,
    };
    let mermaid = MermaidFlowchart.render(&arazzo);

    Ok(mermaid)
}

fn open_mermaid_live(mermaid: &str) -> Result<(), Arazzo2MermaidError> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());

    let mermaid_live = MermaidLive { code: mermaid.to_string() };
    let mermaid_live = serde_json::to_string(&mermaid_live)
        .map_err(Arazzo2MermaidError::Json)?;

    encoder.write_all(mermaid_live.as_bytes())
        .map_err(Arazzo2MermaidError::Deflate)?;

    let compressed_bytes = encoder.finish()
        .map_err(Arazzo2MermaidError::Deflate)?;

    let encoded = BASE64_URL_SAFE_NO_PAD.encode(compressed_bytes);

    open::that(format!("https://mermaid.live/edit#pako:{}", encoded))
        .map_err(Arazzo2MermaidError::Open)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn run_read_stdin() {
        let mut reader = fs::File::open("fixtures/minimal.yml").unwrap();

        let mut content = String::new();
        reader.read_to_string(&mut content).unwrap();

        let reader = Cursor::new(content);

        run(reader, &Format::Yaml).unwrap();
    }

    #[test]
    fn run_read_yml_file() {
        let reader = fs::File::open("fixtures/minimal.yml").unwrap();

        run(reader, &Format::Yaml).unwrap();
    }

    #[test]
    fn run_read_invalid_yaml() {
        let reader = Cursor::new("invalid yaml");

        let actual = run(reader, &Format::Yaml).is_err();

        assert!(actual);
    }

    #[test]
    fn run_read_json_file() {
        let reader = fs::File::open("fixtures/minimal.json").unwrap();

        run(reader, &Format::Json).unwrap();
    }

    #[test]
    fn run_read_invalid_json() {
        let reader = Cursor::new("invalid json");

        let actual = run(reader, &Format::Json).is_err();

        assert!(actual);
    }
}

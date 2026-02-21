use std::error::Error;
use std::{env, fs, io};
use std::io::Read;
use crate::renderer::{Renderer, MermaidFlowchart};
use crate::arazzo::ArazzoDocument;

mod renderer;
mod arazzo;

fn main() -> Result<(), Box<dyn Error>> {
    let args = env::args().skip(1);
    run(args)?;

    Ok(())
}

fn run(mut args: impl Iterator<Item = String>) -> Result<(), Box<dyn Error>> {
    let path = args.next();

    if args.next().is_some() {
        return Err("Too many arguments".into());
    }

    let mut reader: Box<dyn Read> = match path.as_deref() {
        None | Some("-") => Box::new(io::stdin()),
        Some(path) => Box::new(fs::File::open(path)?),
    };

    let mut content = String::new();
    reader.read_to_string(&mut content)?;

    let arazzo: ArazzoDocument = yaml_serde::from_str(&content)?;
    let renderer = MermaidFlowchart;
    print!("{}", renderer.render(&arazzo));

    Ok(())
}


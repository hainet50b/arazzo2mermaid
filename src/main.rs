use std::error::Error;
use std::{env, fs};

fn main() -> Result<(), Box<dyn Error>> {
    let args = env::args().skip(1);
    run(args)?;

    Ok(())
}

fn run(mut args: impl Iterator<Item = String>) -> Result<(), Box<dyn Error>> {
    let file = args.next().ok_or("File path required")?;

    if args.next().is_some() {
        return Err("Too many arguments".into());
    }

    let content = fs::read_to_string(file)?;
    println!("{}", content);

    Ok(())
}

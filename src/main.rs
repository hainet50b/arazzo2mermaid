use std::env;

fn main() {
    let args = env::args().skip(1);
    run(args);
}

fn run(args: impl Iterator<Item = String>) {
    args.for_each(|arg| println!("{}", arg));
}

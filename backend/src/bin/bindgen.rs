use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
pub struct Args {
    #[clap(default_value = "./bindings")]
    /// Path for the generated binds
    path: PathBuf,
}

fn main() {
    let Args { path } = Args::parse();
    if let Err(errs) = backend::bindgen(path) {
        for err in errs {
            eprintln!("{:?}", err)
        }
    }
}

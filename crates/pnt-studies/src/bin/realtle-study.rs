use pnt_studies::realtle::{run, RealTleConfig};
use std::path::PathBuf;

fn main() {
    let output = std::env::args_os()
        .nth(1)
        .map_or_else(|| PathBuf::from("docs/studies/realtle"), PathBuf::from);
    match run(&output, &RealTleConfig::default()) {
        Ok(report) => println!("{}", report.headline),
        Err(error) => {
            eprintln!("realtle study failed: {error}");
            std::process::exit(1);
        }
    }
}

use pnt_studies::highspeed::{self, HighSpeedConfig};
use std::path::PathBuf;

fn main() {
    let mut output = PathBuf::from("docs/studies/highspeed");
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--output" {
            output = PathBuf::from(args.next().expect("--output requires a path"));
        } else {
            panic!("unknown argument: {arg}");
        }
    }
    let report =
        highspeed::run(&output, &HighSpeedConfig::default()).expect("high-speed study failed");
    println!(
        "wrote {} (20 kn / 500 km: {:.2} h)",
        output.display(),
        report.same_distance[1].duration_h
    );
}

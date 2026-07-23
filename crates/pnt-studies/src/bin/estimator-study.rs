use pnt_studies::estimator;
use std::path::PathBuf;

fn main() {
    let mut quick = false;
    let mut output = PathBuf::from("docs/studies/estimator");
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--quick" => quick = true,
            "--output" => {
                output = PathBuf::from(args.next().expect("--output requires a path"));
            }
            _ => panic!("unknown argument: {arg}"),
        }
    }
    let report = estimator::run(&output, quick).expect("estimator study failed");
    println!(
        "wrote {} (D39 velocity RMS {:.4} m/s)",
        output.display(),
        report.d39.baseline.velocity_rms_mps
    );
}

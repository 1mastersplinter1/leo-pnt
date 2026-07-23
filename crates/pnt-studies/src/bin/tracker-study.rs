use std::env;
use std::path::PathBuf;

use pnt_studies::{run, write_results, StudySize};

fn main() {
    let mut arguments = env::args().skip(1);
    let mut output: Option<PathBuf> = None;
    let mut quick = false;
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--out" => output = arguments.next().map(PathBuf::from),
            "--quick" => quick = true,
            "-h" | "--help" => {
                println!("Usage: tracker-study --out DIR [--quick]");
                return;
            }
            unknown => {
                eprintln!("unknown argument: {unknown}");
                std::process::exit(2);
            }
        }
    }
    let Some(output) = output else {
        eprintln!("--out DIR is required");
        std::process::exit(2);
    };
    let results = run(if quick {
        StudySize::quick()
    } else {
        StudySize::full()
    });
    write_results(&results, &output).expect("write study JSON");
    println!(
        "wrote {} blocks; total wall time {:.3} s to {}",
        results.false_alarm_tail.blocks,
        results.manifest.wall_time_seconds.total,
        output.display()
    );
}

use pnt_mission::{run_study, MissionConfig};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let mut config = MissionConfig::default();
    let mut output = None;
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--seed" => config.seed = args.next().ok_or("--seed needs a value")?.parse()?,
            "--duration" => {
                config.duration_s = args.next().ok_or("--duration needs a value")?.parse()?;
            }
            "--out" => output = Some(PathBuf::from(args.next().ok_or("--out needs a value")?)),
            _ => return Err(format!("unknown argument: {argument}").into()),
        }
    }
    let output = output.ok_or("usage: mission-study --seed N --out DIR [--duration SECONDS]")?;
    let report = run_study(&output, &config)?;
    println!("run_directory={}", output.display());
    println!("report={}", output.join("replay-report.json").display());
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

use pnt_studies::maneuver::{run, ManeuverConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "docs/studies/maneuver".into());
    let report = run(output, &ManeuverConfig::default())?;
    println!(
        "wrote {} maneuver-vs-constant cells across {} seeds",
        report.cells.len(),
        report.controls.seed_count,
    );
    Ok(())
}

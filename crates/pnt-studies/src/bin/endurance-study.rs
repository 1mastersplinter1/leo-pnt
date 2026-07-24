use pnt_studies::endurance::{run, EnduranceConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "docs/studies/endurance".into());
    let report = run(output, &EnduranceConfig::default())?;
    println!(
        "wrote {} leg-duration and {} clock-discipline real-EKF outcomes",
        report.leg_duration_curve.len(),
        report.clock_discipline_curve.len()
    );
    Ok(())
}

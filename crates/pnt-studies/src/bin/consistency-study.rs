use pnt_studies::consistency::{run, ConsistencyConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "docs/studies/consistency".into());
    let report = run(output, &ConsistencyConfig::default())?;
    println!(
        "wrote covariance-consistency diagnosis: {} state-group summaries, {} per-epoch NEES samples",
        report.groups.len(),
        report.nees_trace.len(),
    );
    Ok(())
}

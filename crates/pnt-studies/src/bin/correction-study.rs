use pnt_studies::correction::{run, CorrectionConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "docs/studies/correction".into());
    let report = run(output, &CorrectionConfig::default())?;
    println!(
        "wrote empirical covariance-correction study: {} group corrections, {} recommended inflations, scalar restores distribution = {}",
        report.groups.len(),
        report.recommended_inflation.len(),
        report.scalar_restores_distribution,
    );
    Ok(())
}

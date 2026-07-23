use pnt_studies::multisat::{run, MultisatConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "docs/studies/multisat".into());
    let report = run(output, &MultisatConfig::default())?;
    println!(
        "wrote {} real-filter multi-satellite outcomes",
        report.outcomes.len()
    );
    Ok(())
}

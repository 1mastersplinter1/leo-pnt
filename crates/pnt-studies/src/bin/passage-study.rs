fn main() {
    let output = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "docs/studies/passage".into());
    let study = pnt_studies::passage::write(&output).expect("passage study failed");
    println!("wrote {output} ({:.2} km)", study.distance_km);
}

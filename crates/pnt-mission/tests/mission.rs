use pnt_config::GnssAuthority;
use pnt_journal::{MeasurementJournalRecord, MeasurementReader, TruthJournalRecord, TruthReader};
use pnt_mission::{generate_mission, read_manifest, run_study, MissionConfig};
use pnt_replay::replay_directory;
use std::{fs, path::Path};
use tempfile::TempDir;

fn small(seed: u64) -> MissionConfig {
    MissionConfig {
        seed,
        duration_s: 4,
        imu_rate_hz: 100,
        ..MissionConfig::default()
    }
}

fn directory_bytes(path: &Path) -> Vec<(String, Vec<u8>)> {
    let mut values = fs::read_dir(path)
        .unwrap()
        .map(|entry| {
            let entry = entry.unwrap();
            (
                entry.file_name().to_string_lossy().into_owned(),
                fs::read(entry.path()).unwrap(),
            )
        })
        .collect::<Vec<_>>();
    values.sort_by(|left, right| left.0.cmp(&right.0));
    values
}

#[test]
fn same_seed_produces_bit_identical_run_directory() {
    let left = TempDir::new().unwrap();
    let right = TempDir::new().unwrap();
    generate_mission(left.path(), &small(42)).unwrap();
    generate_mission(right.path(), &small(42)).unwrap();
    assert_eq!(directory_bytes(left.path()), directory_bytes(right.path()));
}

#[test]
fn generated_capture_round_trips_all_measurements_and_truth() {
    let directory = TempDir::new().unwrap();
    let summary = generate_mission(directory.path(), &small(7)).unwrap();
    let measurements = MeasurementReader::open(directory.path())
        .unwrap()
        .filter(|record| matches!(record, Ok(MeasurementJournalRecord::Envelope(_))))
        .count() as u64;
    let truth = TruthReader::open(directory.path())
        .unwrap()
        .filter(|record| matches!(record, Ok(TruthJournalRecord::Envelope(_))))
        .count() as u64;
    assert_eq!(measurements, summary.measurement_count);
    assert_eq!(truth, summary.truth_count);
    let manifest = read_manifest(directory.path()).unwrap();
    assert_eq!(manifest.run_uuid, "synthetic-mission-0000000000000007");
    assert!(!manifest.files.is_empty());
}

#[test]
fn paired_study_is_a_synthetic_qualitative_rehearsal() {
    let directory = TempDir::new().unwrap();
    let report = run_study(directory.path(), &small(11)).unwrap();
    assert!(report.caveat.contains("not a performance claim"));
    assert!(report.qualitative_demonstration.aided_smaller_than_withheld);
    assert!(
        report
            .qualitative_demonstration
            .doppler_rich_constant_heading_present
    );
    assert!(report.qualitative_demonstration.outage_or_turn_present);
    assert_eq!(
        report.replay.input_measurement_count,
        report.mission.measurement_count
    );
    assert_eq!(
        report.replay.aided.gnss_fusion_routes,
        report.mission.truth_count
    );
    assert_eq!(report.replay.withheld.gnss_fusion_routes, 0);
}

#[test]
fn d35_comparison_sign_input_identity_and_production_repeat() {
    let directory = TempDir::new().unwrap();
    let report = run_study(directory.path(), &small(19)).unwrap();

    // Every comparison value is aided error minus withheld error. With tiny-noise GNSS
    // pulling production toward truth and GNSS forbidden in withheld mode, the hand-derived
    // sign is negative; zero would mean a tie.
    assert!(report.replay.comparison.horizontal_position_error_m.n > 0);
    assert!(
        report
            .replay
            .comparison
            .horizontal_position_error_m
            .mean
            .unwrap()
            < 0.0
    );
    assert!(
        report
            .replay
            .comparison
            .horizontal_speed_error_mps
            .mean
            .unwrap()
            < 0.0
    );

    let first = replay_directory(directory.path(), GnssAuthority::Production).unwrap();
    let second = replay_directory(directory.path(), GnssAuthority::Production).unwrap();
    assert_eq!(first, second, "Production replay must be bit-exact");
    assert_eq!(
        first.input_measurement_count,
        report.mission.measurement_count
    );
    assert_eq!(
        first.input_measurement_count,
        replay_directory(directory.path(), GnssAuthority::RecordedOnly)
            .unwrap()
            .input_measurement_count,
        "paired modes must receive the identical input count"
    );

    // D35 requests a comparison-pair exclusion count, but schema v1 exposes exclusions only
    // on each run. Keep the observable invariant direct and leave the API gap in the report.
    assert!(report.replay.aided.excluded_no_near_truth > 0);
    assert!(
        report.replay.comparison.horizontal_position_error_m.n
            <= report.replay.aided.matched_epochs
    );
    assert!(report
        .integration_gaps
        .iter()
        .any(|gap| gap.contains("comparison-pair exclusion count")));
}

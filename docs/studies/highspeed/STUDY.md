# High-speed good-fix-loss study

**SYNTHETIC CAPABILITY/PLUMBING DEMONSTRATION [UNVERIFIED]; not a navigation-performance or denied-authority claim.**

Aided until covariance steady state is verified at 300 s, then GNSS denied for 100 km at each of 7/20/30 kn; the real EKF uses the production chi-square gate (9.0), wave/slam is on, and graduated ephemeris aging is on.

Consistent with D50: this synthetic run demonstrates plumbing only. It does not support denied operation at 20 kn; 30 kn remains aided/manual-only and exploratory, with no denied autonomous authority.

| tier | denied time | loss error / covariance trace | landfall error class | velocity RMS | ephemeris age / margin | gate accepted / rejected (aged accepted) | reconvergence time / truth distance |
|---|---:|---:|---:|---:|---:|---:|---:|
| displacement (7 kn) | 7.71 h | 0.71 m / 0.29 m² | 44908.15 m (10-100 km) | 14.666 m/s | 7.80 h / +22.20 h | 13 / 3 (3 aged) | not reported: filter was not converged before turn (105554.1 m / 19.753 m/s) |
| planing (20 kn) | 2.70 h | 0.39 m / 0.35 m² | 11619.87 m (10-100 km) | 2.718 m/s | 2.78 h / +27.22 h | 5 / 1 (0 aged) | not reported: filter was not converged before turn (8745.5 m / 4.606 m/s) |
| exploratory (30 kn) | 1.80 h | 0.31 m / 0.37 m² | 9677.25 m (1-10 km) | 3.166 m/s | 1.88 h / +28.12 h | 3 / 1 (0 aged) | not reported: filter was not converged before turn (8245.0 m / 7.113 m/s) |

## Model and interpretation

Zero-mean full-cycle acceleration integrated into both truth and IMU. 100-450 ms duration and 0.44 g RMS anchor are R5-sourced; 0.25 s, opportunity rate, pitch coupling, speed scaling, and mapping 0.44 g RMS to 6.10 m/s^2 sinusoidal peak are [UNVERIFIED]. The 30 kn 1.84x peak scale is [UNVERIFIED].

- This ISS-TLE-only fixture supplies one satellite. Single-satellite range-rate geometry is near-unobservable for position, so its bounded roughly-ten-to-tens-of-kilometres errors are not the 100-200 m multi-satellite class.
- D51 reconciliation: U-P1's smaller absolute errors came from a deliberately clamped toy PassageEstimator that cannot diverge; this study uses the real EKF with its production gate. U-P1 remains evidence only for the relative graduated-vs-hard aging comparison.
- Position, velocity, covariance and any valid reconvergence are read from the real EKF state against generator truth; divergent runs are explicitly flagged and receive no reconvergence metric.
- The 90-degree turn runs at a sharp, realistic 3 deg/s. Distance-to-reconvergence is accumulated from truth positions after that actual turn, independently of elapsed time.
- Graduated aging is exercised by the executive Doppler pipeline; the 30 h ceiling and synthetic TLE aging remain [UNVERIFIED] per D43/D45.
- OPEN / REQUIRED: build and run a multi-satellite fixture study before claiming the 100-200 m denied position class.

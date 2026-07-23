# High-speed good-fix-loss study

**SYNTHETIC CAPABILITY/PLUMBING DEMONSTRATION [UNVERIFIED]; not a navigation-performance or denied-authority claim.**

Aided for 300 s to a real converged EKF state, then GNSS denied for 100 km at each of 7/20/30 kn; wave/slam on; graduated ephemeris aging on.

Consistent with D50: this synthetic run demonstrates plumbing only. It does not support denied operation at 20 kn; 30 kn remains aided/manual-only and exploratory, with no denied autonomous authority.

| tier | denied time | loss error / covariance trace | landfall error class | velocity RMS | ephemeris age / margin | graduated updates | reconvergence time / truth distance |
|---|---:|---:|---:|---:|---:|---:|---:|
| displacement (7 kn) | 7.71 h | 0.71 m / 0.29 m² | 40944894.05 m (>=500 m) | 53604.856 m/s | 7.80 h / +22.20 h | 16 accepted, 3 aged | 2561 s / 8989 m |
| planing (20 kn) | 2.70 h | 0.39 m / 0.35 m² | 134375499.84 m (>=500 m) | 91052.223 m/s | 2.78 h / +27.22 h | 6 accepted, 0 aged | 1752 s / 17856 m |
| exploratory (30 kn) | 1.80 h | 0.31 m / 0.37 m² | 158771669.87 m (>=500 m) | 89004.898 m/s | 1.88 h / +28.12 h | 4 accepted, 0 aged | 1 s / 0 m |

## Model and interpretation

Zero-mean full-cycle acceleration integrated into both truth and IMU. 100-450 ms duration and 0.44 g RMS anchor are R5-sourced; 0.25 s, opportunity rate, pitch coupling, speed scaling, and mapping 0.44 g RMS to 6.10 m/s^2 sinusoidal peak are [UNVERIFIED]. The 30 kn 1.84x peak scale is [UNVERIFIED].

- Position, velocity, covariance and reconvergence are read from the real EKF state against generator truth; no endpoint or convergence formula is used.
- Distance-to-reconvergence is accumulated from truth positions after the generator's actual coordinated turn, independently of elapsed time.
- Graduated aging is exercised by the executive Doppler pipeline; the 30 h ceiling and synthetic TLE aging remain [UNVERIFIED] per D43/D45.

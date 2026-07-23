# U-P1 report — graduated ephemeris aging

## Disposition

Implemented the D45 graduated model: legacy ephemeris callers retain the inclusive 6 h gate;
the Doppler executive uses a typed propagation result carrying age, nominal weighting through
6 h, additive variance through an inclusive 30 h ceiling, and hard rejection above it.
Inflated observations produce an integrity `NOTE` containing age and applied `sigma_add`;
the authority fail-closed register is unchanged.

## Derivation

The line through 0.94 km at 6 h and 2.6 km at 24 h is
`sigma_r=0.386667+0.0922222*a_h km`. With the reference geometry
`|u_dot|=v_rel/range=7.6/1000=0.0076 rad/s`, independent added uncertainty beyond the nominal
fresh model is `sigma_add=|u_dot|*sqrt(sigma_r(a)^2-sigma_r(6h)^2)`. A central finite-difference
test of rotating-LOS range agrees with the implementation.

## Passage comparison

The committed deterministic synthetic run covers 100.01 km in 9 h at 6 kn, with GNSS lost at
2 h and ephemeris cached at departure. The hard gate loses Doppler at 6 h and finishes at
3050 m error (DR class). Graduated handling retains it through 9 h and finishes at the
synthetic 350 m bound (passage-held class). D43 applies: this is a stand-in, not real aging
validation.

## `[UNVERIFIED]`

- The two-point linear SGP4/SupGP error-growth fit and extrapolation to 30 h.
- Reference relative speed, range, isotropic orbit-error model, and Doppler mapping.
- The 30 h hard ceiling and all default inflation coefficients.
- Synthetic aided bound (350 m), DR drift rate (0.25 m/s), and position-class proxy.
- Real-SupGP aging, constellation availability, real tracker residuals, and at-sea replay.

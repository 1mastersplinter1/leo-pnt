# pnt-studies

Deterministic, JSON-emitting stress studies for `pnt-tracker`. The default run is the
committed full study (500 signal seeds per C/N0 level, 1,000,000 noise blocks, and 200
impairment trials per point):

```sh
cargo run -p pnt-studies --release --bin tracker-study -- --out docs/studies/tracker
```

Use `--quick` for a CI-scale schema and behavior smoke test. Quick output is deliberately
labelled with its actual counts and must not be cited as full-study evidence. Rayon
parallelizes independent seeded trials. Wall times vary; all scientific samples are
deterministic.

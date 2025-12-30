# Benchmarks (Criterion)

Date: 2025-12-30
Machine: Apple Silicon, stable Rust

Command: `cargo bench`

Results (ns/op unless stated):

- evaluate_five/high_card A,K,7,5,2: ~197 ns
- evaluate_five/straight_flush royal: ~126 ns
- evaluate_seven (fixed 7 cards): ~4.45 µs

Notes:
- Plotters backend used (gnuplot absent).
- Outliers present (typical); compare medians across runs.
- As evaluator optimizes, update this file with deltas and environment notes.


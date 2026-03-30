# Benchmark Baseline

This directory stores lightweight benchmark comparison data for the minimal
path work. These numbers are intended as a reproducible local baseline for
trend comparison, not as hard pass/fail thresholds or cross-machine
performance promises.

Measured configuration:

- Source commit under test: `477118a`
- Date: `2026-03-31`
- Toolchain: `1.85.0`
- Command runner: [run_benchmark_baseline.sh](../scripts/run_benchmark_baseline.sh)
- Common bench args: `--sample-count 8 --min-time 0.005`

## bench_parse

| benchmark | mean |
| --- | --- |
| `bench_full_parse` | `24.03 ms` |
| `bench_tokenizer` | `12.71 ms` |

## bench_core_vs_serde

| benchmark | mean | mean throughput |
| --- | --- | --- |
| `parse_typed_core_on_core_syntax` | `38.29 ms` | `22.93 MB/s` |
| `parse_typed_core_on_serde_syntax` | `38.86 ms` | `25.77 MB/s` |
| `parse_typed_serde` | `49.3 ms` | `20.31 MB/s` |
| `parse_value_core_on_core_syntax` | `44.43 ms` | `19.76 MB/s` |
| `parse_value_core_on_serde_syntax` | `46.22 ms` | `21.67 MB/s` |
| `parse_value_default` | `63.52 ms` | `15.76 MB/s` |
| `stringify_typed_core_direct` | `5.166 ms` | `170 MB/s` |
| `stringify_typed_serde` | `85.16 ms` | `11.76 MB/s` |
| `stringify_typed_via_core_format` | `38.75 ms` | `22.66 MB/s` |
| `stringify_value_core` | `9.814 ms` | `89.48 MB/s` |
| `stringify_value_default` | `60.83 ms` | `16.46 MB/s` |

Use these values for relative comparisons when extending the core-backed path.
If the benchmark configuration changes, update this file together with the
runner script.

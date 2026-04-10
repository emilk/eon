# Benchmark Baseline

This directory stores lightweight benchmark comparison data for the minimal
path work. These numbers are intended as a reproducible local baseline for
trend comparison, not as hard pass/fail thresholds or cross-machine
performance promises.

Measured configuration:

- Source commit under test: `8ba27dd`
- Date: `2026-03-31`
- Toolchain: `1.85.0`
- Command runner: [run_benchmark_baseline.sh](../scripts/run_benchmark_baseline.sh)
- Common bench args: `--sample-count 8 --min-time 0.005`

## bench_parse

| benchmark | mean |
| --- | --- |
| `bench_full_parse` | `7.526 ms` |
| `bench_tokenizer` | `3.758 ms` |

## bench_core_vs_serde

| benchmark | mean | mean throughput |
| --- | --- | --- |
| `parse_typed_core_on_core_syntax` | `7.227 ms` | `121.5 MB/s` |
| `parse_typed_core_on_serde_syntax` | `7.386 ms` | `135.6 MB/s` |
| `parse_typed_serde` | `11.81 ms` | `84.81 MB/s` |
| `parse_value_core_on_core_syntax` | `8.949 ms` | `98.13 MB/s` |
| `parse_value_core_on_serde_syntax` | `8.865 ms` | `112.9 MB/s` |
| `parse_value_default` | `14.29 ms` | `70.06 MB/s` |
| `stringify_typed_core_direct` | `1.859 ms` | `472.3 MB/s` |
| `stringify_typed_serde` | `14.26 ms` | `70.23 MB/s` |
| `stringify_typed_via_core_format` | `6.795 ms` | `129.2 MB/s` |
| `stringify_value_core` | `2.399 ms` | `366 MB/s` |
| `stringify_value_default` | `9.606 ms` | `104.2 MB/s` |

Use these values for relative comparisons when extending the core-backed path.
If the benchmark configuration changes, update this file together with the
runner script.

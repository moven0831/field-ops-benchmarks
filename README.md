# Field Ops Benchmarks on Metal and WebGPU

GPU benchmarking for cryptographic field arithmetic on M31 and BN254.

## Overview

Benchmarks cryptographic field arithmetic operations on GPU, comparing Metal (native macOS) vs WebGPU backends. Focus on operations relevant to zero-knowledge proving systems.

## Supported Operations

| Operation | Description |
|-----------|-------------|
| `u32_add` | Native 32-bit addition (baseline) |
| `u64_add` | 64-bit addition (native Metal, emulated WebGPU) |
| `m31_field_add` | Mersenne-31 field addition |
| `m31_field_mul` | Mersenne-31 field multiplication |
| `bn254_field_add` | BN254 field addition |
| `bn254_field_mul` | BN254 field multiplication (Montgomery) |

## Quick Start

```bash
cargo run --release  # interactive mode
```

## Understanding Results

### Metrics

- **gops_per_second** - Giga-operations per second (throughput)
- **cycles_per_op** - Estimated GPU cycles per operation (Metal only)
- **min_ns/mean_ns** - Timing statistics in nanoseconds

### Sample Results (Apple M3)

| Operation | Metal GOP/s | WebGPU GOP/s | Ratio |
|-----------|-------------|--------------|-------|
| u32_add | 52.9 | 50.9 | 1.04x |
| m31_field_add | 79.3 | 25.2 | 3.1x |
| m31_field_mul | 42.6 | 10.0 | 4.3x |
| bn254_field_add | 4.0 | 0.5 | 8x |
| bn254_field_mul | 0.58 | 0.07 | 8x |

### Key Insights

- **M31 vs BN254**: M31 (31-bit) is significantly faster than BN254 (254-bit) because it fits in native u32 operations
- **Metal vs WebGPU**: Metal significantly outperforms WebGPU for complex field operations (3-8x), while native u32 shows parity
- **Multiplication cost**: BN254 mul is ~90x slower than u32 add due to multi-limb Montgomery multiplication
- **Field choice matters**: For ZK proving, M31-based systems can achieve ~70x higher throughput than BN254 for multiplications

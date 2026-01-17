# Metal vs WebGPU GPU benchmarks for M31 and BN254 field arithmetic

Benchmarks cryptographic field arithmetic operations on GPU, comparing Metal (native macOS) vs WebGPU backends. Focus on operations relevant to zero-knowledge proving systems.

## Benchmarked Operations

| Operation | Description |
|-----------|-------------|
| `u32_add` | Native 32-bit addition (baseline) |
| `u64_add` | 64-bit addition (native Metal, emulated WebGPU) |
| `m31_field_add` | Mersenne-31 field addition |
| `m31_field_mul` | Mersenne-31 field multiplication |
| `bn254_field_add` | BN254 field addition |
| `bn254_field_mul` | BN254 field multiplication (Montgomery [CIOS](https://eprint.iacr.org/2016/487.pdf)) |

## Quick Start

```bash
cargo run --release  # interactive mode
```

## Understanding Results

### Metrics

- **gops_per_second** - Giga-operations per second (throughput)
- **min_ns/mean_ns** - Timing statistics in nanoseconds

### Sample Results (Apple M3 chip)

| Operation | Metal GOP/s | WebGPU GOP/s | Ratio |
|-----------|-------------|--------------|-------|
| u32_add | 264.2 | 250.1 | 1.06x |
| u64_add | 177.5 | 141.1 | 1.26x |
| m31_field_add | 146.0 | 121.7 | 1.20x |
| m31_field_mul | 112.0 | 57.9 | 1.93x |
| bn254_field_add | 7.9 | 1.0 | 7.64x |
| bn254_field_mul | 0.63 | 0.08 | 7.59x |

### Key Findings

- **u64 emulation overhead**: Metal supports native 64-bit integers; WebGPU requires manual emulation with two u32 values. The overhead is modest (1.26x) but amplifies on large fields, like those in elliptic curve operations.
- **Field size vs throughput**: Smaller fields yield higher throughput on client-side GPUs. M31 (31-bit) sustains over **100 Gops/s**, whereas BN254 (254-bit) falls below **1 Gops/s**. For ZKP schemes, those operating on smaller fields are better suited for client-side GPU acceleration.
- **Complexity amplifies backend gap**: GPUs natively handle 32-bit words at the hardware level. For u32, Metal and WebGPU are nearly identical (1.06x). With more bits or complex logic (e.g. multi-limb ops, Montgomery multiplication), gaps widen: M31 within **2x**, BN254's arithmetic at **7x**. Metal's native API and compiler outperform WebGPU's abstraction layer on complexity.

## Benchmark Configuration

Both backends use identical configs and shaders for fair comparison. These are **NOT** optimal production settings, real-world implementations would apply optimizations like dynamic dispatch tuned to specific GPU capabilities on devices.

| Parameter | Value |
|-----------|-------|
| Workgroup size | 64 threads |
| Num workgroups | 1024 |
| Total threads | 65,536 |
| Warmup iterations | 3 (not timed) |
| Measurement iterations | 10 |

| Operation | ops_per_thread |
|-----------|----------------|
| u32_add | 100,000 |
| u64_add | 100,000 |
| m31_field_add | 100,000 |
| m31_field_mul | 100,000 |
| bn254_field_add | 100 |
| bn254_field_mul | 100 |

## Buffer Architecture

Both backends use equivalent buffer types optimized for Apple Silicon's unified memory.

### Metal

Uses [MTLResourceOptions](https://developer.apple.com/documentation/metal/mtlresourceoptions) storage modes:

| Buffer | Storage Mode | Size | Rationale |
|--------|--------------|------|-----------|
| Input | [`StorageModeShared`](https://developer.apple.com/documentation/metal/mtlstoragemode/shared) | 64 bytes | CPU-initialized; shared is optimal for small buffers |
| Output | [`StorageModePrivate`](https://developer.apple.com/documentation/metal/mtlstoragemode/private) | 256 KB | GPU-only write; no CPU readback |
| Params | [`StorageModeShared`](https://developer.apple.com/documentation/metal/mtlstoragemode/shared) | 16 bytes | CPU-initialized uniform data |

### WebGPU (wgpu)

Uses [BufferUsages](https://docs.rs/wgpu/latest/wgpu/struct.BufferUsages.html) flags:

| Buffer | Usage Flags | Size | Rationale |
|--------|-------------|------|-----------|
| Input | [`STORAGE`](https://docs.rs/wgpu/latest/wgpu/struct.BufferUsages.html#associatedconstant.STORAGE) | 64 bytes | CPU-initialized via `create_buffer_init` |
| Output | [`STORAGE`](https://docs.rs/wgpu/latest/wgpu/struct.BufferUsages.html#associatedconstant.STORAGE) | 256 KB | GPU-only write; no `COPY_SRC` needed |
| Params | [`UNIFORM`](https://docs.rs/wgpu/latest/wgpu/struct.BufferUsages.html#associatedconstant.UNIFORM) | 16 bytes | CPU-initialized via `create_buffer_init` |

**Note**: [`StorageModeManaged`](https://developer.apple.com/documentation/metal/mtlstoragemode/managed) is NOT available on Apple Silicon—it was designed for discrete GPUs on Intel Macs.

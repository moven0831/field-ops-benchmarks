#include <metal_stdlib>
#include "types.metal"

using namespace metal;

// ============================================================================
// Benchmark: Native u32 Addition
// ============================================================================
// This benchmark measures the raw throughput of native u32 addition.

kernel void bench_u32_add(
    device const uint* input [[buffer(0)]],
    device uint* output [[buffer(1)]],
    constant BenchParams& params [[buffer(2)]],
    uint tid [[thread_position_in_grid]]
) {
    // Initialize with thread-unique seed
    uint acc = params.seed ^ tid;
    uint b = input[(tid + 8) % 16];

    // Main benchmark loop - addition operations
    for (uint i = 0; i < params.iterations; i++) {
        // Addition: acc = acc + b
        acc = acc + b;

        // Data-dependent modification to prevent optimization
        b = b ^ (acc & 0xFFu);
    }

    // Write result to prevent dead code elimination
    output[tid] = acc;
}

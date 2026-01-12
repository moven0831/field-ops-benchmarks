#include <metal_stdlib>
#include "types.metal"

using namespace metal;

// ============================================================================
// Benchmark: Native u32 Multiply-Add Baseline
// ============================================================================
// This benchmark measures the raw throughput of native u32 operations.
// Used as a baseline to compare against emulated operations.

kernel void bench_u32_baseline(
    device const uint* input [[buffer(0)]],
    device uint* output [[buffer(1)]],
    constant BenchParams& params [[buffer(2)]],
    uint tid [[thread_position_in_grid]]
) {
    // Initialize with thread-unique seed
    uint acc = params.seed ^ tid;
    uint a = input[tid % 16];
    uint b = input[(tid + 8) % 16];

    // Main benchmark loop - multiply-add operations
    for (uint i = 0; i < params.iterations; i++) {
        // Multiply-add: acc = acc * a + b
        acc = acc * a + b;

        // Data-dependent modification to prevent optimization
        a = a ^ (acc & 0xFFu);
    }

    // Write result to prevent dead code elimination
    output[tid] = acc;
}

#include <metal_stdlib>
#include "types.metal"

using namespace metal;

// ============================================================================
// Benchmark: Native u64 Operations (Metal-specific)
// ============================================================================
// This benchmark measures the throughput of native 64-bit operations
// available in Metal. Used to compare against emulated u64 in WebGPU.

kernel void bench_u64_native(
    device const uint* input [[buffer(0)]],
    device ulong* output [[buffer(1)]],
    constant BenchParams& params [[buffer(2)]],
    uint tid [[thread_position_in_grid]]
) {
    // Initialize with thread-unique seed (combine two u32s into u64)
    ulong acc = (ulong(params.seed) << 32) | (params.seed ^ tid);
    ulong a = (ulong(input[tid % 16]) << 32) | input[(tid + 1) % 16];
    ulong b = (ulong(input[(tid + 2) % 16]) << 32) | input[(tid + 3) % 16];

    // Main benchmark loop - 64-bit multiply-add operations
    for (uint i = 0; i < params.iterations; i++) {
        // 64-bit multiply-add: acc = acc * a + b
        acc = acc * a + b;

        // Data-dependent modification to prevent optimization
        a = a ^ (acc & 0xFFull);
    }

    // Write result to prevent dead code elimination
    output[tid] = acc;
}

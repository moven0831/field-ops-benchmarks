#include <metal_stdlib>
#include "field.metal"

using namespace metal;

// ============================================================================
// Benchmark: BN254 Field Multiplication (Montgomery)
// ============================================================================
// Tests Montgomery multiplication for the BN254 base field.
// This is the most critical operation for ZK proof systems.

kernel void bench_field_mul(
    device const uint* input [[buffer(0)]],
    device uint* output [[buffer(1)]],
    constant BenchParams& params [[buffer(2)]],
    uint tid [[thread_position_in_grid]]
) {
    // Initialize field elements from input
    BigInt256 a;
    BigInt256 b;

    for (uint i = 0; i < NUM_LIMBS; i++) {
        a.limbs[i] = (input[(tid + i) % 16] ^ (tid * (i + 1u))) & W_mask;
        b.limbs[i] = (input[(tid + i + 8) % 16] ^ (tid * (i + 17u))) & W_mask;
    }

    // Reduce to valid field elements
    a = field_reduce(a);
    b = field_reduce(b);

    // Accumulator in Montgomery form
    BigInt256 acc = a;

    // Main benchmark loop
    for (uint i = 0; i < params.iterations; i++) {
        // Field multiplication (Montgomery)
        acc = field_mul(acc, b);

        // Data-dependent modification to prevent optimization
        b.limbs[0] = (b.limbs[0] ^ (acc.limbs[0] & 0xFFu)) & W_mask;
    }

    // Write result (XOR all limbs to single value)
    uint result = 0u;
    for (uint i = 0; i < NUM_LIMBS; i++) {
        result ^= acc.limbs[i];
    }
    output[tid] = result;
}

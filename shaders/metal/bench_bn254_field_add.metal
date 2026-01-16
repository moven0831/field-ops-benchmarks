#include <metal_stdlib>
#include "bn254_field.metal"

using namespace metal;

// ============================================================================
// Benchmark: BN254 Field Addition
// ============================================================================
// Tests modular addition for BN254 base field.

kernel void bench_bn254_field_add(
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

    // Accumulator
    BigInt256 acc = a;

    // Main benchmark loop
    for (uint i = 0; i < params.iterations; i++) {
        // Field addition
        acc = field_add(acc, b);

        // Data-dependent modification
        b.limbs[0] = (b.limbs[0] ^ (acc.limbs[0] & 0xFFu)) & W_mask;
    }

    // Write result
    uint result = 0u;
    for (uint i = 0; i < NUM_LIMBS; i++) {
        result ^= acc.limbs[i];
    }
    output[tid] = result;
}

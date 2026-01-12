#include <metal_stdlib>
#include "bigint.metal"

using namespace metal;

// ============================================================================
// Benchmark: BigInt256 Multiplication
// ============================================================================
// Tests 256-bit integer multiplication using 16x16-bit limb representation.
// This is the foundation for field multiplication.

kernel void bench_bigint_mul(
    device const uint* input [[buffer(0)]],
    device uint* output [[buffer(1)]],
    constant BenchParams& params [[buffer(2)]],
    uint tid [[thread_position_in_grid]]
) {
    // Initialize BigInt256 operands from input
    BigInt256 a;
    BigInt256 b;

    for (uint i = 0; i < NUM_LIMBS; i++) {
        a.limbs[i] = (input[(tid + i) % 16] ^ (tid * (i + 1u))) & W_mask;
        b.limbs[i] = (input[(tid + i + 8) % 16] ^ (tid * (i + 17u))) & W_mask;
    }

    // Accumulator for results
    BigInt256 acc = a;

    // Main benchmark loop
    for (uint i = 0; i < params.iterations; i++) {
        // 256-bit multiplication and take low 256 bits
        BigInt512 product = bigint_mul_wide(acc, b);
        acc = bigint512_low(product);

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

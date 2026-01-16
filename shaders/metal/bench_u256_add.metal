#include <metal_stdlib>
#include "bigint.metal"

using namespace metal;

// ============================================================================
// Benchmark: 256-bit BigInt Addition (no modular reduction)
// ============================================================================
// Tests pure BigInt addition without field reduction overhead.

kernel void bench_u256_add(
    device const uint* input [[buffer(0)]],
    device uint* output [[buffer(1)]],
    constant BenchParams& params [[buffer(2)]],
    uint tid [[thread_position_in_grid]]
) {
    // Initialize BigInt256 values from input
    BigInt256 a;
    BigInt256 b;

    for (uint i = 0; i < NUM_LIMBS; i++) {
        a.limbs[i] = (input[(tid + i) % 16] ^ (tid * (i + 1u))) & W_mask;
        b.limbs[i] = (input[(tid + i + 8) % 16] ^ (tid * (i + 17u))) & W_mask;
    }

    // Accumulator
    BigInt256 acc = a;

    // Main benchmark loop - pure addition, no modular reduction
    for (uint i = 0; i < params.iterations; i++) {
        BigInt256 tmp;
        uint carry = bigint_add(tmp, acc, b);
        acc = tmp;

        // Data-dependent modification to prevent optimization
        b.limbs[0] = (b.limbs[0] ^ (acc.limbs[0] & 0xFFu) ^ carry) & W_mask;
    }

    // Write result (XOR all limbs)
    uint result = 0u;
    for (uint i = 0; i < NUM_LIMBS; i++) {
        result ^= acc.limbs[i];
    }
    output[tid] = result;
}

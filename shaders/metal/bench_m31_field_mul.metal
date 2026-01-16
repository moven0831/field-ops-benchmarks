#include <metal_stdlib>
#include "types.metal"

using namespace metal;

// ============================================================================
// Benchmark: Mersenne Prime (2^31-1) Field Multiplication
// ============================================================================
// Uses native ulong (64-bit) for the intermediate product, then reduces.
// Product of two 31-bit values fits in 62 bits.

constant uint MERSENNE_P = 0x7FFFFFFFu;  // 2^31 - 1

// Reduce a 64-bit value modulo Mersenne prime
// Uses the identity: 2^31 = 1 (mod p)
inline uint mersenne_reduce_u64(ulong x) {
    // First reduction: split into 31-bit chunks
    uint low = uint(x) & MERSENNE_P;           // bits 0-30
    uint mid = uint(x >> 31) & MERSENNE_P;     // bits 31-61
    uint sum = low + mid;                       // at most 32 bits

    // Second reduction if needed
    sum = (sum & MERSENNE_P) + (sum >> 31);

    // Final reduction if sum == p
    return sum >= MERSENNE_P ? sum - MERSENNE_P : sum;
}

// Field multiplication: (a * b) mod p
// Assumes a, b < p
inline uint mersenne_mul(uint a, uint b) {
    ulong product = ulong(a) * ulong(b);  // 62 bits max
    return mersenne_reduce_u64(product);
}

kernel void bench_m31_field_mul(
    device const uint* input [[buffer(0)]],
    device uint* output [[buffer(1)]],
    constant BenchParams& params [[buffer(2)]],
    uint tid [[thread_position_in_grid]]
) {
    // Initialize with thread-unique seed, reduced to field
    uint acc = (params.seed ^ tid) & MERSENNE_P;
    if (acc == 0u) acc = 1u;  // Avoid multiplicative identity trap

    uint b = input[(tid + 8) % 16] & MERSENNE_P;
    if (b == 0u) b = 1u;

    // Main benchmark loop - field multiplication operations
    for (uint i = 0; i < params.iterations; i++) {
        // Field multiplication: acc = (acc * b) mod p
        acc = mersenne_mul(acc, b);

        // Data-dependent modification to prevent optimization
        b = (b ^ (acc & 0xFFu)) & MERSENNE_P;
        if (b == 0u) b = 1u;  // Keep b non-zero
    }

    // Write result to prevent dead code elimination
    output[tid] = acc;
}

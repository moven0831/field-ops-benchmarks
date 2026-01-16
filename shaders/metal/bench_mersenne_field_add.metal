#include <metal_stdlib>
#include "types.metal"

using namespace metal;

// ============================================================================
// Benchmark: Mersenne Prime (2^31-1) Field Addition
// ============================================================================
// The Mersenne prime p = 2^31 - 1 = 0x7FFFFFFF allows extremely efficient
// modular reduction using the identity: 2^31 = 1 (mod p)
// Therefore: x mod p = (x & p) + (x >> 31), with possible final reduction

constant uint MERSENNE_P = 0x7FFFFFFFu;  // 2^31 - 1

// Reduce to [0, p) range
// Input: value in range [0, 2*p)
// Output: value in range [0, p)
inline uint mersenne_reduce(uint x) {
    uint r = (x & MERSENNE_P) + (x >> 31);
    return r >= MERSENNE_P ? r - MERSENNE_P : r;
}

// Field addition: (a + b) mod p
// Assumes a, b < p
inline uint mersenne_add(uint a, uint b) {
    uint sum = a + b;  // Range: [0, 2p-2]
    return mersenne_reduce(sum);
}

kernel void bench_mersenne_field_add(
    device const uint* input [[buffer(0)]],
    device uint* output [[buffer(1)]],
    constant BenchParams& params [[buffer(2)]],
    uint tid [[thread_position_in_grid]]
) {
    // Initialize with thread-unique seed, reduced to field
    uint acc = (params.seed ^ tid) & MERSENNE_P;
    uint b = input[(tid + 8) % 16] & MERSENNE_P;

    // Main benchmark loop - field addition operations
    for (uint i = 0; i < params.iterations; i++) {
        // Field addition: acc = (acc + b) mod p
        acc = mersenne_add(acc, b);

        // Data-dependent modification to prevent optimization
        b = (b ^ (acc & 0xFFu)) & MERSENNE_P;
    }

    // Write result to prevent dead code elimination
    output[tid] = acc;
}

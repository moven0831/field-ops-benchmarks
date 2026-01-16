// ============================================================================
// Benchmark: Mersenne Prime (2^31-1) Field Addition
// ============================================================================
// The Mersenne prime p = 2^31 - 1 = 0x7FFFFFFF allows extremely efficient
// modular reduction using the identity: 2^31 = 1 (mod p)

const MERSENNE_P: u32 = 0x7FFFFFFFu;  // 2^31 - 1

struct BenchParams {
    iterations: u32,
    seed: u32,
    _pad0: u32,
    _pad1: u32,
}

@group(0) @binding(0) var<storage, read> input: array<u32>;
@group(0) @binding(1) var<storage, read_write> output: array<u32>;
@group(0) @binding(2) var<uniform> params: BenchParams;

// Reduce to [0, p) range
// Input: value in range [0, 2*p)
fn mersenne_reduce(x: u32) -> u32 {
    let r = (x & MERSENNE_P) + (x >> 31u);
    if (r >= MERSENNE_P) {
        return r - MERSENNE_P;
    }
    return r;
}

// Field addition: (a + b) mod p
fn mersenne_add(a: u32, b: u32) -> u32 {
    let sum = a + b;  // Range: [0, 2p-2]
    return mersenne_reduce(sum);
}

@compute @workgroup_size(64)
fn bench_m31_field_add(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let tid = global_id.x;

    // Initialize with thread-unique seed, reduced to field
    var acc: u32 = (params.seed ^ tid) & MERSENNE_P;
    var b: u32 = input[(tid + 8u) % 16u] & MERSENNE_P;

    // Main benchmark loop - field addition operations
    for (var i: u32 = 0u; i < params.iterations; i = i + 1u) {
        // Field addition: acc = (acc + b) mod p
        acc = mersenne_add(acc, b);

        // Data-dependent modification to prevent optimization
        b = (b ^ (acc & 0xFFu)) & MERSENNE_P;
    }

    // Write result to prevent dead code elimination
    output[tid] = acc;
}

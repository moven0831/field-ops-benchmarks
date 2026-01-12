// ============================================================================
// Benchmark: Emulated u64 Operations (WebGPU-specific)
// ============================================================================
// This benchmark measures the overhead of emulating 64-bit operations
// using 32-bit pairs with carry propagation.

struct BenchParams {
    iterations: u32,
    seed: u32,
    _pad0: u32,
    _pad1: u32,
}

@group(0) @binding(0) var<storage, read> input: array<u32>;
@group(0) @binding(1) var<storage, read_write> output: array<u32>;
@group(0) @binding(2) var<uniform> params: BenchParams;

// Emulated u64 as two u32 values (lo, hi)
struct U64 {
    lo: u32,
    hi: u32,
}

// u64 addition with carry
fn u64_add(a: U64, b: U64) -> U64 {
    let lo = a.lo + b.lo;
    let carry = select(0u, 1u, lo < a.lo);
    let hi = a.hi + b.hi + carry;
    return U64(lo, hi);
}

// u64 multiplication (32x32 -> 64 using the identity:
// (a_hi * 2^32 + a_lo) * (b_hi * 2^32 + b_lo) mod 2^64
// = a_lo * b_lo + (a_hi * b_lo + a_lo * b_hi) * 2^32
fn u64_mul(a: U64, b: U64) -> U64 {
    // Split into 16-bit parts for 32-bit multiplication
    let a_lo_lo = a.lo & 0xFFFFu;
    let a_lo_hi = a.lo >> 16u;
    let b_lo_lo = b.lo & 0xFFFFu;
    let b_lo_hi = b.lo >> 16u;

    // Compute partial products for low 64 bits
    let p0 = a_lo_lo * b_lo_lo;                    // bits 0-31
    let p1 = a_lo_lo * b_lo_hi;                    // bits 16-47
    let p2 = a_lo_hi * b_lo_lo;                    // bits 16-47
    let p3 = a_lo_hi * b_lo_hi;                    // bits 32-63

    // Cross terms from hi*lo
    let p4 = a.lo * b.hi;                          // contributes to hi
    let p5 = a.hi * b.lo;                          // contributes to hi

    // Combine partial products
    let mid = (p0 >> 16u) + (p1 & 0xFFFFu) + (p2 & 0xFFFFu);
    let lo = (p0 & 0xFFFFu) | ((mid & 0xFFFFu) << 16u);
    let hi = p3 + (p1 >> 16u) + (p2 >> 16u) + (mid >> 16u) + p4 + p5;

    return U64(lo, hi);
}

@compute @workgroup_size(64)
fn bench_u64_emulated(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let tid = global_id.x;

    // Initialize with thread-unique seed
    var acc = U64(params.seed ^ tid, params.seed);
    var a = U64(input[tid % 16u], input[(tid + 1u) % 16u]);
    var b = U64(input[(tid + 2u) % 16u], input[(tid + 3u) % 16u]);

    // Main benchmark loop - emulated 64-bit multiply-add
    for (var i: u32 = 0u; i < params.iterations; i = i + 1u) {
        // Emulated 64-bit multiply-add: acc = acc * a + b
        acc = u64_add(u64_mul(acc, a), b);

        // Data-dependent modification
        a.lo = a.lo ^ (acc.lo & 0xFFu);
    }

    // Write result (XOR both halves)
    output[tid] = acc.lo ^ acc.hi;
}

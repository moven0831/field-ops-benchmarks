// ============================================================================
// Benchmark: Emulated u64 Addition (WebGPU-specific)
// ============================================================================
// This benchmark measures the overhead of emulating 64-bit addition
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

@compute @workgroup_size(64)
fn bench_u64_add(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let tid = global_id.x;

    // Initialize with thread-unique seed
    var acc = U64(params.seed ^ tid, params.seed);
    var b = U64(input[(tid + 2u) % 16u], input[(tid + 3u) % 16u]);

    // Main benchmark loop - emulated 64-bit addition
    for (var i: u32 = 0u; i < params.iterations; i = i + 1u) {
        // Emulated 64-bit addition: acc = acc + b
        acc = u64_add(acc, b);

        // Data-dependent modification
        b.lo = b.lo ^ (acc.lo & 0xFFu);
    }

    // Write result (XOR both halves)
    output[tid] = acc.lo ^ acc.hi;
}

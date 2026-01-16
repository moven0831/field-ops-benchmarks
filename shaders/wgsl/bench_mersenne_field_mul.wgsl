// ============================================================================
// Benchmark: Mersenne Prime (2^31-1) Field Multiplication
// ============================================================================
// WebGPU lacks native u64, so we emulate the 64-bit product using 16-bit
// partial products.

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

// Multiply two u32 values and return result as vec2<u32> (low, high)
fn mul_u32_wide(a: u32, b: u32) -> vec2<u32> {
    // Split into 16-bit halves
    let a_lo = a & 0xFFFFu;
    let a_hi = a >> 16u;
    let b_lo = b & 0xFFFFu;
    let b_hi = b >> 16u;

    // Partial products (each fits in 32 bits)
    let p0 = a_lo * b_lo;           // bits 0-31
    let p1 = a_lo * b_hi;           // bits 16-47
    let p2 = a_hi * b_lo;           // bits 16-47
    let p3 = a_hi * b_hi;           // bits 32-63

    // Combine middle terms
    let mid = p1 + p2;
    let mid_carry = select(0u, 1u, mid < p1);

    // Combine into low and high 32-bit words
    let low = p0 + (mid << 16u);
    let low_carry = select(0u, 1u, low < p0);

    let high = p3 + (mid >> 16u) + (mid_carry << 16u) + low_carry;

    return vec2<u32>(low, high);
}

// Reduce a 62-bit product (stored as vec2<u32>) modulo Mersenne prime
// Uses: 2^31 = 1 (mod p)
fn mersenne_reduce_u64(x: vec2<u32>) -> u32 {
    // Extract 31-bit chunks:
    // chunk0: bits 0-30
    // chunk1: bits 31-61
    let chunk0 = x.x & MERSENNE_P;
    let chunk1 = ((x.y << 1u) | (x.x >> 31u)) & MERSENNE_P;

    // Sum the chunks
    var sum = chunk0 + chunk1;

    // Reduce if needed
    sum = (sum & MERSENNE_P) + (sum >> 31u);

    // Final reduction if sum >= p
    if (sum >= MERSENNE_P) {
        return sum - MERSENNE_P;
    }
    return sum;
}

// Field multiplication: (a * b) mod p
fn mersenne_mul(a: u32, b: u32) -> u32 {
    let product = mul_u32_wide(a, b);
    return mersenne_reduce_u64(product);
}

@compute @workgroup_size(64)
fn bench_mersenne_field_mul(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let tid = global_id.x;

    // Initialize with thread-unique seed, reduced to field
    var acc: u32 = (params.seed ^ tid) & MERSENNE_P;
    if (acc == 0u) { acc = 1u; }  // Avoid multiplicative identity trap

    var b: u32 = input[(tid + 8u) % 16u] & MERSENNE_P;
    if (b == 0u) { b = 1u; }

    // Main benchmark loop - field multiplication operations
    for (var i: u32 = 0u; i < params.iterations; i = i + 1u) {
        // Field multiplication: acc = (acc * b) mod p
        acc = mersenne_mul(acc, b);

        // Data-dependent modification to prevent optimization
        b = (b ^ (acc & 0xFFu)) & MERSENNE_P;
        if (b == 0u) { b = 1u; }  // Keep b non-zero
    }

    // Write result to prevent dead code elimination
    output[tid] = acc;
}

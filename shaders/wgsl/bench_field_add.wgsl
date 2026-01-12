// ============================================================================
// Benchmark: BN254 Field Addition
// ============================================================================
// Tests modular addition for BN254 base field.

// Constants
const W: u32 = 16u;
const W_mask: u32 = 0xFFFFu;
const NUM_LIMBS: u32 = 16u;

// BN254 modulus p
const BN254_P: array<u32, 16> = array<u32, 16>(
    0x0D87u, 0x06C3u, 0x0550u, 0x048Du,
    0x09D5u, 0x01E3u, 0x0E88u, 0x0879u,
    0x051Au, 0x0181u, 0x0B20u, 0x0C1Cu,
    0x057Bu, 0x074Eu, 0x09D6u, 0x030Cu
);

struct BenchParams {
    iterations: u32,
    seed: u32,
    _pad0: u32,
    _pad1: u32,
}

@group(0) @binding(0) var<storage, read> input: array<u32>;
@group(0) @binding(1) var<storage, read_write> output: array<u32>;
@group(0) @binding(2) var<uniform> params: BenchParams;

// Check if BigInt256 >= BN254_P
fn bigint_gte_p(a: array<u32, 16>) -> bool {
    for (var i: i32 = 15; i >= 0; i = i - 1) {
        if (a[i] > BN254_P[i]) { return true; }
        if (a[i] < BN254_P[i]) { return false; }
    }
    return true;
}

// BigInt256 subtraction with BN254_P
fn bigint_sub_p(a: array<u32, 16>) -> array<u32, 16> {
    var result: array<u32, 16>;
    var borrow: u32 = 0u;

    for (var i: u32 = 0u; i < NUM_LIMBS; i = i + 1u) {
        if (a[i] >= BN254_P[i] + borrow) {
            result[i] = a[i] - BN254_P[i] - borrow;
            borrow = 0u;
        } else {
            result[i] = ((1u << W) + a[i]) - BN254_P[i] - borrow;
            borrow = 1u;
        }
    }

    return result;
}

// Reduce modulo p
fn field_reduce(a: array<u32, 16>) -> array<u32, 16> {
    if (bigint_gte_p(a)) {
        return bigint_sub_p(a);
    }
    return a;
}

// BigInt256 addition
fn bigint_add(a: array<u32, 16>, b: array<u32, 16>) -> array<u32, 17> {
    var result: array<u32, 17>;
    var carry: u32 = 0u;

    for (var i: u32 = 0u; i < NUM_LIMBS; i = i + 1u) {
        let sum = a[i] + b[i] + carry;
        result[i] = sum & W_mask;
        carry = sum >> W;
    }
    result[16] = carry;

    return result;
}

// Field addition
fn field_add(a: array<u32, 16>, b: array<u32, 16>) -> array<u32, 16> {
    let sum = bigint_add(a, b);

    var result: array<u32, 16>;
    for (var i: u32 = 0u; i < NUM_LIMBS; i = i + 1u) {
        result[i] = sum[i];
    }

    // If result >= p or carry occurred, subtract p
    if (sum[16] != 0u || bigint_gte_p(result)) {
        return bigint_sub_p(result);
    }
    return result;
}

@compute @workgroup_size(64)
fn bench_field_add(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let tid = global_id.x;

    // Initialize field elements
    var a: array<u32, 16>;
    var b: array<u32, 16>;

    for (var i: u32 = 0u; i < NUM_LIMBS; i = i + 1u) {
        a[i] = (input[(tid + i) % 16u] ^ (tid * (i + 1u))) & W_mask;
        b[i] = (input[(tid + i + 8u) % 16u] ^ (tid * (i + 17u))) & W_mask;
    }

    // Reduce to valid field elements
    a = field_reduce(a);
    b = field_reduce(b);

    var acc: array<u32, 16> = a;

    // Main benchmark loop
    for (var iter: u32 = 0u; iter < params.iterations; iter = iter + 1u) {
        acc = field_add(acc, b);
        b[0] = (b[0] ^ (acc[0] & 0xFFu)) & W_mask;
    }

    // Write result
    var result: u32 = 0u;
    for (var i: u32 = 0u; i < NUM_LIMBS; i = i + 1u) {
        result = result ^ acc[i];
    }
    output[tid] = result;
}

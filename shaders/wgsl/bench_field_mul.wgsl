// ============================================================================
// Benchmark: BN254 Field Multiplication (Montgomery)
// ============================================================================
// Tests Montgomery multiplication for the BN254 base field.

// Constants
const W: u32 = 16u;
const W_mask: u32 = 0xFFFFu;
const NUM_LIMBS: u32 = 16u;
const MONTGOMERY_INV: u32 = 0x63B9u;

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

// CIOS Montgomery multiplication: computes (a * b * R^-1) mod p
// Fuses multiplication and reduction in a single pass using only 18 limbs
fn mont_mul_cios(a: array<u32, 16>, b: array<u32, 16>) -> array<u32, 16> {
    var t: array<u32, 18>;
    for (var i: u32 = 0u; i < 18u; i = i + 1u) {
        t[i] = 0u;
    }

    for (var i: u32 = 0u; i < NUM_LIMBS; i = i + 1u) {
        // Phase 1: Multiply-accumulate a[i] * b
        var c: u32 = 0u;
        for (var j: u32 = 0u; j < NUM_LIMBS; j = j + 1u) {
            let prod = a[i] * b[j];
            let sum = t[j] + (prod & W_mask) + c;
            t[j] = sum & W_mask;
            c = (prod >> W) + (sum >> W);
        }
        let sum16 = t[16] + c;
        t[16] = sum16 & W_mask;
        t[17] = t[17] + (sum16 >> W);

        // Phase 2: Reduction - compute m and add m * p
        let m = (t[0] * MONTGOMERY_INV) & W_mask;
        c = 0u;
        for (var j: u32 = 0u; j < NUM_LIMBS; j = j + 1u) {
            let prod = m * BN254_P[j];
            let sum = t[j] + (prod & W_mask) + c;
            t[j] = sum & W_mask;
            c = (prod >> W) + (sum >> W);
        }
        let sum16_2 = t[16] + c + t[17];
        t[16] = sum16_2 & W_mask;
        t[17] = sum16_2 >> W;

        // Phase 3: Shift right (discard t[0] which is now 0)
        for (var j: u32 = 0u; j < 17u; j = j + 1u) {
            t[j] = t[j + 1u];
        }
        t[17] = 0u;
    }

    var result: array<u32, 16>;
    for (var i: u32 = 0u; i < NUM_LIMBS; i = i + 1u) {
        result[i] = t[i];
    }
    return field_reduce(result);
}

// Montgomery reduction
fn mont_reduce(t: array<u32, 32>) -> array<u32, 16> {
    var limbs: array<u32, 33>;
    for (var i: u32 = 0u; i < 32u; i = i + 1u) {
        limbs[i] = t[i];
    }
    limbs[32] = 0u;

    for (var i: u32 = 0u; i < NUM_LIMBS; i = i + 1u) {
        let q = (limbs[i] * MONTGOMERY_INV) & W_mask;

        var carry: u32 = 0u;
        for (var j: u32 = 0u; j < NUM_LIMBS; j = j + 1u) {
            let product = q * BN254_P[j] + limbs[i + j] + carry;
            limbs[i + j] = product & W_mask;
            carry = product >> W;
        }
        var k: u32 = i + NUM_LIMBS;
        while (k < 33u) {
            let sum = limbs[k] + carry;
            limbs[k] = sum & W_mask;
            carry = sum >> W;
            if (carry == 0u) { break; }
            k = k + 1u;
        }
    }

    var result: array<u32, 16>;
    for (var i: u32 = 0u; i < NUM_LIMBS; i = i + 1u) {
        result[i] = limbs[i + NUM_LIMBS];
    }

    return field_reduce(result);
}

// Field multiplication using CIOS
fn field_mul(a: array<u32, 16>, b: array<u32, 16>) -> array<u32, 16> {
    return mont_mul_cios(a, b);
}

@compute @workgroup_size(64)
fn bench_field_mul(@builtin(global_invocation_id) global_id: vec3<u32>) {
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
        acc = field_mul(acc, b);
        b[0] = (b[0] ^ (acc[0] & 0xFFu)) & W_mask;
    }

    // Write result
    var result: u32 = 0u;
    for (var i: u32 = 0u; i < NUM_LIMBS; i = i + 1u) {
        result = result ^ acc[i];
    }
    output[tid] = result;
}

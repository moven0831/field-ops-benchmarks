// ============================================================================
// BigInt256 Type Definition and Arithmetic for BN254 Field Operations
// ============================================================================
// Uses 16 x 16-bit limbs stored in u32 for consistent representation.
// WebGPU (WGSL) lacks native u64 support, so we use 16-bit limbs.

const W: u32 = 16u;                          // Limb width in bits
const W_mask: u32 = 0xFFFFu;                 // (1 << W) - 1
const NUM_LIMBS: u32 = 16u;                  // 256 / 16 = 16 limbs

// Montgomery constant: -p[0]^(-1) mod 2^16
const MONTGOMERY_INV: u32 = 0x63B9u;

// BN254 base field modulus p (16-bit limbs, little-endian)
// p = 21888242871839275222246405745257275088696311157297823662689037894645226208583
const BN254_P: array<u32, 16> = array<u32, 16>(
    0x0D87u, 0x06C3u, 0x0550u, 0x048Du,
    0x09D5u, 0x01E3u, 0x0E88u, 0x0879u,
    0x051Au, 0x0181u, 0x0B20u, 0x0C1Cu,
    0x057Bu, 0x074Eu, 0x09D6u, 0x030Cu
);

// Benchmark parameters passed from host
struct BenchParams {
    iterations: u32,
    seed: u32,
    _pad0: u32,
    _pad1: u32,
}

// BigInt256 comparison: returns true if a >= b
fn bigint_gte(a: array<u32, 16>, b: array<u32, 16>) -> bool {
    for (var i: i32 = 15; i >= 0; i = i - 1) {
        if (a[i] > b[i]) { return true; }
        if (a[i] < b[i]) { return false; }
    }
    return true; // Equal
}

// Check if BigInt256 >= BN254_P
fn bigint_gte_p(a: array<u32, 16>) -> bool {
    for (var i: i32 = 15; i >= 0; i = i - 1) {
        if (a[i] > BN254_P[i]) { return true; }
        if (a[i] < BN254_P[i]) { return false; }
    }
    return true; // Equal
}

// BigInt256 addition: result = a + b, returns carry
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

// BigInt256 subtraction: result = a - b, returns borrow
fn bigint_sub(a: array<u32, 16>, b: array<u32, 16>) -> array<u32, 17> {
    var result: array<u32, 17>;
    var borrow: u32 = 0u;

    for (var i: u32 = 0u; i < NUM_LIMBS; i = i + 1u) {
        if (a[i] >= b[i] + borrow) {
            result[i] = a[i] - b[i] - borrow;
            borrow = 0u;
        } else {
            result[i] = ((1u << W) + a[i]) - b[i] - borrow;
            borrow = 1u;
        }
    }
    result[16] = borrow;

    return result;
}

// BigInt256 addition with constant (BN254_P)
fn bigint_add_p(a: array<u32, 16>) -> array<u32, 17> {
    var result: array<u32, 17>;
    var carry: u32 = 0u;

    for (var i: u32 = 0u; i < NUM_LIMBS; i = i + 1u) {
        let sum = a[i] + BN254_P[i] + carry;
        result[i] = sum & W_mask;
        carry = sum >> W;
    }
    result[16] = carry;

    return result;
}

// BigInt256 subtraction with constant (BN254_P)
fn bigint_sub_p(a: array<u32, 16>) -> array<u32, 17> {
    var result: array<u32, 17>;
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
    result[16] = borrow;

    return result;
}

// Extract array<u32, 16> from first 16 elements
fn extract_low(a: array<u32, 17>) -> array<u32, 16> {
    var result: array<u32, 16>;
    for (var i: u32 = 0u; i < 16u; i = i + 1u) {
        result[i] = a[i];
    }
    return result;
}

// Extract array<u32, 16> from array<u32, 32> starting at index
fn extract_from_32(a: array<u32, 32>, start: u32) -> array<u32, 16> {
    var result: array<u32, 16>;
    for (var i: u32 = 0u; i < 16u; i = i + 1u) {
        result[i] = a[start + i];
    }
    return result;
}

// Zero BigInt256
fn bigint_zero() -> array<u32, 16> {
    var result: array<u32, 16>;
    for (var i: u32 = 0u; i < 16u; i = i + 1u) {
        result[i] = 0u;
    }
    return result;
}

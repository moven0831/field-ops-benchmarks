// ============================================================================
// BN254 Field Arithmetic (Montgomery Form)
// ============================================================================
// Field operations for the BN254 base field using Montgomery representation.
// All field elements are stored in Montgomery form: aR mod p, where R = 2^256.

// Include bigint definitions
// Note: In WGSL, we need to include these definitions directly or import them

// Reduce a BigInt256 modulo p (ensure result < p)
fn field_reduce(a: array<u32, 16>) -> array<u32, 16> {
    if (bigint_gte_p(a)) {
        let result = bigint_sub_p(a);
        return extract_low(result);
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

// Field addition: (a + b) mod p
fn field_add(a: array<u32, 16>, b: array<u32, 16>) -> array<u32, 16> {
    let sum = bigint_add(a, b);
    var result = extract_low(sum);

    // If result >= p or carry occurred, subtract p
    if (sum[16] != 0u || bigint_gte_p(result)) {
        let reduced = bigint_sub_p(result);
        return extract_low(reduced);
    }
    return result;
}

// Field subtraction: (a - b) mod p
fn field_sub(a: array<u32, 16>, b: array<u32, 16>) -> array<u32, 16> {
    let diff = bigint_sub(a, b);
    var result = extract_low(diff);

    // If borrow occurred, add p
    if (diff[16] != 0u) {
        let corrected = bigint_add_p(result);
        return extract_low(corrected);
    }
    return result;
}

// Montgomery reduction: given T (up to 512 bits), compute T * R^{-1} mod p
fn mont_reduce(t: array<u32, 32>) -> array<u32, 16> {
    // Working copy (33 limbs for overflow)
    var limbs: array<u32, 33>;
    for (var i: u32 = 0u; i < 32u; i = i + 1u) {
        limbs[i] = t[i];
    }
    limbs[32] = 0u;

    // Montgomery reduction: for each limb, eliminate the low bits
    for (var i: u32 = 0u; i < NUM_LIMBS; i = i + 1u) {
        // Compute q = limbs[i] * MONTGOMERY_INV mod 2^W
        let q = (limbs[i] * MONTGOMERY_INV) & W_mask;

        // Add q * p to limbs, starting at position i
        var carry: u32 = 0u;
        for (var j: u32 = 0u; j < NUM_LIMBS; j = j + 1u) {
            let product = q * BN254_P[j] + limbs[i + j] + carry;
            limbs[i + j] = product & W_mask;
            carry = product >> W;
        }
        // Propagate carry
        var k: u32 = i + NUM_LIMBS;
        while (k < 33u) {
            let sum = limbs[k] + carry;
            limbs[k] = sum & W_mask;
            carry = sum >> W;
            if (carry == 0u) { break; }
            k = k + 1u;
        }
    }

    // Extract result from upper half
    var result: array<u32, 16>;
    for (var i: u32 = 0u; i < NUM_LIMBS; i = i + 1u) {
        result[i] = limbs[i + NUM_LIMBS];
    }

    // Final reduction if result >= p
    return field_reduce(result);
}

// Field multiplication: (a * b) mod p using Montgomery multiplication
fn field_mul(a: array<u32, 16>, b: array<u32, 16>) -> array<u32, 16> {
    return mont_mul_cios(a, b);
}

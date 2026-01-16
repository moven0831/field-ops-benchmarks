#pragma once

#include "bigint.metal"

// ============================================================================
// BN254 Field Arithmetic (Montgomery Form)
// ============================================================================
// Field operations for the BN254 base field using Montgomery representation.
// All field elements are stored in Montgomery form: aR mod p, where R = 2^256.

// Reduce a BigInt256 modulo p (ensure result < p)
inline BigInt256 field_reduce(BigInt256 a) {
    BigInt256 result;
    if (bigint_gte_p(a)) {
        bigint_sub_const(result, a, BN254_P);
        return result;
    }
    return a;
}

// CIOS Montgomery multiplication: computes (a * b * R^-1) mod p
// Fuses multiplication and reduction in a single pass using only 18 limbs
inline BigInt256 mont_mul_cios(BigInt256 a, BigInt256 b) {
    uint t[18];
    for (uint i = 0u; i < 18u; i++) {
        t[i] = 0u;
    }

    for (uint i = 0u; i < NUM_LIMBS; i++) {
        // Phase 1: Multiply-accumulate a[i] * b
        uint c = 0u;
        for (uint j = 0u; j < NUM_LIMBS; j++) {
            uint prod = a.limbs[i] * b.limbs[j];
            uint sum = t[j] + (prod & W_mask) + c;
            t[j] = sum & W_mask;
            c = (prod >> W) + (sum >> W);
        }
        uint sum16 = t[16] + c;
        t[16] = sum16 & W_mask;
        t[17] = t[17] + (sum16 >> W);

        // Phase 2: Reduction - compute m and add m * p
        uint m = (t[0] * MONTGOMERY_INV) & W_mask;
        c = 0u;
        for (uint j = 0u; j < NUM_LIMBS; j++) {
            uint prod = m * BN254_P[j];
            uint sum = t[j] + (prod & W_mask) + c;
            t[j] = sum & W_mask;
            c = (prod >> W) + (sum >> W);
        }
        uint sum16_2 = t[16] + c + t[17];
        t[16] = sum16_2 & W_mask;
        t[17] = sum16_2 >> W;

        // Phase 3: Shift right (discard t[0] which is now 0)
        for (uint j = 0u; j < 17u; j++) {
            t[j] = t[j + 1];
        }
        t[17] = 0u;
    }

    BigInt256 result;
    for (uint i = 0u; i < NUM_LIMBS; i++) {
        result.limbs[i] = t[i];
    }
    return field_reduce(result);
}

// Field addition: (a + b) mod p
// Assumes a, b < p
inline BigInt256 field_add(BigInt256 a, BigInt256 b) {
    BigInt256 result;
    uint carry = bigint_add(result, a, b);

    // If result >= p, subtract p
    if (carry != 0u || bigint_gte_p(result)) {
        BigInt256 reduced;
        bigint_sub_const(reduced, result, BN254_P);
        return reduced;
    }
    return result;
}

// Field subtraction: (a - b) mod p
// Assumes a, b < p
inline BigInt256 field_sub(BigInt256 a, BigInt256 b) {
    BigInt256 result;
    uint borrow = bigint_sub(result, a, b);

    // If borrow occurred, add p to result
    if (borrow != 0u) {
        BigInt256 corrected;
        bigint_add_const(corrected, result, BN254_P);
        return corrected;
    }
    return result;
}

// Montgomery reduction: given T (up to 512 bits), compute T * R^{-1} mod p
// Uses the CIOS (Coarsely Integrated Operand Scanning) algorithm
inline BigInt256 mont_reduce(BigInt512 t) {
    // Working copy
    uint limbs[33];  // Extra limb for overflow
    for (uint i = 0u; i < 32u; i++) {
        limbs[i] = t.limbs[i];
    }
    limbs[32] = 0u;

    // Montgomery reduction: for each limb, eliminate the low bits
    for (uint i = 0u; i < NUM_LIMBS; i++) {
        // Compute q = limbs[i] * MONTGOMERY_INV mod 2^W
        uint q = (limbs[i] * MONTGOMERY_INV) & W_mask;

        // Add q * p to limbs, starting at position i
        uint carry = 0u;
        for (uint j = 0u; j < NUM_LIMBS; j++) {
            uint product = q * BN254_P[j] + limbs[i + j] + carry;
            limbs[i + j] = product & W_mask;
            carry = product >> W;
        }
        // Propagate carry
        for (uint k = i + NUM_LIMBS; k < 33u; k++) {
            uint sum = limbs[k] + carry;
            limbs[k] = sum & W_mask;
            carry = sum >> W;
            if (carry == 0u) break;
        }
    }

    // Extract result from upper half
    BigInt256 result;
    for (uint i = 0u; i < NUM_LIMBS; i++) {
        result.limbs[i] = limbs[i + NUM_LIMBS];
    }

    // Final reduction if result >= p
    return field_reduce(result);
}

// Field multiplication: (a * b) mod p using Montgomery multiplication
// Inputs and output are in Montgomery form
inline BigInt256 field_mul(BigInt256 a, BigInt256 b) {
    return mont_mul_cios(a, b);
}

// Field squaring: a^2 mod p using Montgomery multiplication
inline BigInt256 field_sqr(BigInt256 a) {
    BigInt512 product = bigint_sqr_wide(a);
    return mont_reduce(product);
}

// Convert to Montgomery form: a * R mod p
// Input: standard representation, Output: Montgomery form
inline BigInt256 to_montgomery(BigInt256 a) {
    // Multiply by R^2 and reduce
    BigInt256 r2;
    for (uint i = 0u; i < NUM_LIMBS; i++) {
        r2.limbs[i] = BN254_R2[i];
    }
    return field_mul(a, r2);
}

// Convert from Montgomery form: a * R^{-1} mod p
// Input: Montgomery form, Output: standard representation
inline BigInt256 from_montgomery(BigInt256 a) {
    // Create BigInt512 with a in low half, zeros in high half
    BigInt512 extended;
    for (uint i = 0u; i < NUM_LIMBS; i++) {
        extended.limbs[i] = a.limbs[i];
    }
    for (uint i = NUM_LIMBS; i < 32u; i++) {
        extended.limbs[i] = 0u;
    }
    return mont_reduce(extended);
}

// Field negation: -a mod p
inline BigInt256 field_neg(BigInt256 a) {
    // Check if a is zero
    bool is_zero = true;
    for (uint i = 0u; i < NUM_LIMBS; i++) {
        if (a.limbs[i] != 0u) {
            is_zero = false;
            break;
        }
    }
    if (is_zero) {
        return a;
    }

    // Return p - a
    BigInt256 result;
    BigInt256 p;
    for (uint i = 0u; i < NUM_LIMBS; i++) {
        p.limbs[i] = BN254_P[i];
    }
    bigint_sub(result, p, a);
    return result;
}

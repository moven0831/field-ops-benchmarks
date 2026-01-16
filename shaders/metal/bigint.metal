#pragma once

#include "types.metal"

// ============================================================================
// BigInt256 Arithmetic Operations
// ============================================================================
// All operations use 16-bit limbs stored in u32 for portability.

// Compare two BigInt256 values: returns -1 if a < b, 0 if a == b, 1 if a > b
inline int bigint_compare(BigInt256 a, BigInt256 b) {
    for (int i = NUM_LIMBS - 1; i >= 0; i--) {
        if (a.limbs[i] < b.limbs[i]) return -1;
        if (a.limbs[i] > b.limbs[i]) return 1;
    }
    return 0;
}

// Check if BigInt256 is greater than or equal to the BN254 modulus
inline bool bigint_gte_p(BigInt256 a) {
    for (int i = NUM_LIMBS - 1; i >= 0; i--) {
        if (a.limbs[i] > BN254_P[i]) return true;
        if (a.limbs[i] < BN254_P[i]) return false;
    }
    return true;  // Equal
}

// BigInt256 addition: result = a + b
// Returns carry (0 or 1)
inline uint bigint_add(thread BigInt256& result, BigInt256 a, BigInt256 b) {
    uint carry = 0u;
    for (uint i = 0u; i < NUM_LIMBS; i++) {
        uint sum = a.limbs[i] + b.limbs[i] + carry;
        result.limbs[i] = sum & W_mask;
        carry = sum >> W;
    }
    return carry;
}

// BigInt256 subtraction: result = a - b
// Returns borrow (0 or 1)
inline uint bigint_sub(thread BigInt256& result, BigInt256 a, BigInt256 b) {
    uint borrow = 0u;
    for (uint i = 0u; i < NUM_LIMBS; i++) {
        uint diff = a.limbs[i] - b.limbs[i] - borrow;
        // Check if borrow occurred (diff wrapped around)
        borrow = (diff > a.limbs[i]) ? 1u : ((diff == a.limbs[i]) ? borrow : 0u);
        // More robust borrow check
        if (a.limbs[i] < b.limbs[i] + borrow) {
            diff = (1u << W) + a.limbs[i] - b.limbs[i] - borrow;
            borrow = 1u;
        } else {
            diff = a.limbs[i] - b.limbs[i] - borrow;
            borrow = 0u;
        }
        result.limbs[i] = diff & W_mask;
    }
    return borrow;
}

// BigInt256 addition with constant array
inline uint bigint_add_const(thread BigInt256& result, BigInt256 a, constant uint* b) {
    uint carry = 0u;
    for (uint i = 0u; i < NUM_LIMBS; i++) {
        uint sum = a.limbs[i] + b[i] + carry;
        result.limbs[i] = sum & W_mask;
        carry = sum >> W;
    }
    return carry;
}

// BigInt256 subtraction with constant array
inline uint bigint_sub_const(thread BigInt256& result, BigInt256 a, constant uint* b) {
    uint borrow = 0u;
    for (uint i = 0u; i < NUM_LIMBS; i++) {
        if (a.limbs[i] >= b[i] + borrow) {
            result.limbs[i] = a.limbs[i] - b[i] - borrow;
            borrow = 0u;
        } else {
            result.limbs[i] = ((1u << W) + a.limbs[i]) - b[i] - borrow;
            borrow = 1u;
        }
    }
    return borrow;
}

// BigInt256 squaring -> BigInt512 (optimized)
inline BigInt512 bigint_sqr_wide(BigInt256 a) {
    BigInt512 result;
    for (uint i = 0u; i < 32u; i++) {
        result.limbs[i] = 0u;
    }

    // Compute off-diagonal terms (doubled)
    for (uint i = 0u; i < NUM_LIMBS; i++) {
        uint carry = 0u;
        for (uint j = i + 1u; j < NUM_LIMBS; j++) {
            uint idx = i + j;
            uint product = 2u * a.limbs[i] * a.limbs[j] + result.limbs[idx] + carry;
            result.limbs[idx] = product & W_mask;
            carry = product >> W;
        }
        // Propagate carry
        for (uint k = i + NUM_LIMBS; carry != 0u && k < 32u; k++) {
            uint sum = result.limbs[k] + carry;
            result.limbs[k] = sum & W_mask;
            carry = sum >> W;
        }
    }

    // Add diagonal terms (a[i] * a[i])
    uint carry = 0u;
    for (uint i = 0u; i < NUM_LIMBS; i++) {
        uint idx = 2u * i;
        uint product = a.limbs[i] * a.limbs[i] + result.limbs[idx] + carry;
        result.limbs[idx] = product & W_mask;
        carry = product >> W;

        // Propagate to next limb
        uint sum = result.limbs[idx + 1u] + carry;
        result.limbs[idx + 1u] = sum & W_mask;
        carry = sum >> W;
    }

    return result;
}

// Extract low 256 bits from BigInt512
inline BigInt256 bigint512_low(BigInt512 a) {
    BigInt256 result;
    for (uint i = 0u; i < NUM_LIMBS; i++) {
        result.limbs[i] = a.limbs[i];
    }
    return result;
}

// Extract high 256 bits from BigInt512
inline BigInt256 bigint512_high(BigInt512 a) {
    BigInt256 result;
    for (uint i = 0u; i < NUM_LIMBS; i++) {
        result.limbs[i] = a.limbs[i + NUM_LIMBS];
    }
    return result;
}

// Zero a BigInt256
inline BigInt256 bigint_zero() {
    BigInt256 result;
    for (uint i = 0u; i < NUM_LIMBS; i++) {
        result.limbs[i] = 0u;
    }
    return result;
}

// Create BigInt256 from a single u32 value
inline BigInt256 bigint_from_u32(uint value) {
    BigInt256 result = bigint_zero();
    result.limbs[0] = value & W_mask;
    result.limbs[1] = (value >> W) & W_mask;
    return result;
}

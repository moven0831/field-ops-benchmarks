#pragma once

#include <metal_stdlib>
using namespace metal;

// ============================================================================
// BigInt256 Type Definition for BN254 Field Arithmetic
// ============================================================================
// Uses 16 x 16-bit limbs stored in u32 for consistent representation
// with WebGPU (which lacks native u64 support).

constant uint W = 16u;                          // Limb width in bits
constant uint W_mask = 0xFFFFu;                 // (1 << W) - 1
constant uint NUM_LIMBS = 16u;                  // 256 / 16 = 16 limbs

// Montgomery constant: -p[0]^(-1) mod 2^16
// For BN254 base field modulus
constant uint MONTGOMERY_INV = 0x63B9u;

// BigInt256 structure using 16-bit limbs in u32 storage
struct BigInt256 {
    uint limbs[NUM_LIMBS];  // Each limb holds 16 bits in low half
};

// BigInt512 for multiplication results
struct BigInt512 {
    uint limbs[32];  // 32 x 16-bit limbs = 512 bits
};

// BN254 base field modulus p
// p = 21888242871839275222246405745257275088696311157297823662689037894645226208583
constant uint BN254_P[NUM_LIMBS] = {
    0xD87u, 0x6C3u, 0x550u, 0x48Du,
    0x9D5u, 0x1E3u, 0xE88u, 0x879u,
    0x51Au, 0x181u, 0xB20u, 0xC1Cu,
    0x57Bu, 0x74Eu, 0x9D6u, 0x30Cu
};

// 2 * p for subtraction borrow handling
constant uint BN254_2P[NUM_LIMBS] = {
    0x1B0Eu, 0x0D87u, 0x0AA1u, 0x091Au,
    0x13ABu, 0x03C7u, 0x1D11u, 0x10F3u,
    0x0A35u, 0x0302u, 0x1641u, 0x1838u,
    0x0AF7u, 0x0E9Cu, 0x13ACu, 0x0618u
};

// R^2 mod p for Montgomery conversion
// R = 2^256
constant uint BN254_R2[NUM_LIMBS] = {
    0x0B91u, 0x1546u, 0x0D25u, 0x0ABEu,
    0x0D59u, 0x001Bu, 0x0E35u, 0x0ED3u,
    0x0B46u, 0x0D8Bu, 0x0E8Bu, 0x0D44u,
    0x0D62u, 0x0EB4u, 0x01D6u, 0x06F9u
};

// Benchmark parameters passed from host
struct BenchParams {
    uint iterations;
    uint seed;
    uint _pad0;
    uint _pad1;
};

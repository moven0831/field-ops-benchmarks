// ============================================================================
// Benchmark: 256-bit BigInt Addition (no modular reduction)
// ============================================================================
// Tests pure BigInt addition without field reduction overhead.

// Constants
const W: u32 = 16u;
const W_mask: u32 = 0xFFFFu;
const NUM_LIMBS: u32 = 16u;

struct BenchParams {
    iterations: u32,
    seed: u32,
    _pad0: u32,
    _pad1: u32,
}

@group(0) @binding(0) var<storage, read> input: array<u32>;
@group(0) @binding(1) var<storage, read_write> output: array<u32>;
@group(0) @binding(2) var<uniform> params: BenchParams;

// BigInt256 addition returning result with carry in index 16
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

// Extract low 256 bits from result
fn extract_low(a: array<u32, 17>) -> array<u32, 16> {
    var result: array<u32, 16>;
    for (var i: u32 = 0u; i < 16u; i = i + 1u) {
        result[i] = a[i];
    }
    return result;
}

@compute @workgroup_size(64)
fn bench_u256_add(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let tid = global_id.x;

    // Initialize BigInt256 values
    var a: array<u32, 16>;
    var b: array<u32, 16>;

    for (var i: u32 = 0u; i < NUM_LIMBS; i = i + 1u) {
        a[i] = (input[(tid + i) % 16u] ^ (tid * (i + 1u))) & W_mask;
        b[i] = (input[(tid + i + 8u) % 16u] ^ (tid * (i + 17u))) & W_mask;
    }

    var acc: array<u32, 16> = a;

    // Main benchmark loop - pure addition, no modular reduction
    for (var iter: u32 = 0u; iter < params.iterations; iter = iter + 1u) {
        let sum = bigint_add(acc, b);
        acc = extract_low(sum);

        // Data-dependent modification to prevent optimization
        b[0] = (b[0] ^ (acc[0] & 0xFFu) ^ sum[16]) & W_mask;
    }

    // Write result (XOR all limbs)
    var result: u32 = 0u;
    for (var i: u32 = 0u; i < NUM_LIMBS; i = i + 1u) {
        result = result ^ acc[i];
    }
    output[tid] = result;
}

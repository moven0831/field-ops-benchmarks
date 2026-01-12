// ============================================================================
// Benchmark: BigInt256 Multiplication
// ============================================================================
// Tests 256-bit integer multiplication using 16x16-bit limb representation.

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

// BigInt256 x BigInt256 -> BigInt512 multiplication (schoolbook)
fn bigint_mul_wide(a: array<u32, 16>, b: array<u32, 16>) -> array<u32, 32> {
    var result: array<u32, 32>;

    // Initialize to zero
    for (var i: u32 = 0u; i < 32u; i = i + 1u) {
        result[i] = 0u;
    }

    // Schoolbook multiplication
    for (var i: u32 = 0u; i < NUM_LIMBS; i = i + 1u) {
        var carry: u32 = 0u;
        for (var j: u32 = 0u; j < NUM_LIMBS; j = j + 1u) {
            let product = a[i] * b[j] + result[i + j] + carry;
            result[i + j] = product & W_mask;
            carry = product >> W;
        }
        // Propagate remaining carry
        var k: u32 = i + NUM_LIMBS;
        while (carry != 0u && k < 32u) {
            let sum = result[k] + carry;
            result[k] = sum & W_mask;
            carry = sum >> W;
            k = k + 1u;
        }
    }

    return result;
}

@compute @workgroup_size(64)
fn bench_bigint_mul(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let tid = global_id.x;

    // Initialize BigInt256 operands from input
    var a: array<u32, 16>;
    var b: array<u32, 16>;

    for (var i: u32 = 0u; i < NUM_LIMBS; i = i + 1u) {
        a[i] = (input[(tid + i) % 16u] ^ (tid * (i + 1u))) & W_mask;
        b[i] = (input[(tid + i + 8u) % 16u] ^ (tid * (i + 17u))) & W_mask;
    }

    // Accumulator for results
    var acc: array<u32, 16> = a;

    // Main benchmark loop
    for (var iter: u32 = 0u; iter < params.iterations; iter = iter + 1u) {
        // 256-bit multiplication and take low 256 bits
        let product = bigint_mul_wide(acc, b);

        // Extract low 256 bits
        for (var i: u32 = 0u; i < NUM_LIMBS; i = i + 1u) {
            acc[i] = product[i];
        }

        // Data-dependent modification
        b[0] = (b[0] ^ (acc[0] & 0xFFu)) & W_mask;
    }

    // Write result (XOR all limbs to single value)
    var result: u32 = 0u;
    for (var i: u32 = 0u; i < NUM_LIMBS; i = i + 1u) {
        result = result ^ acc[i];
    }
    output[tid] = result;
}

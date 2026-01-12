// ============================================================================
// Benchmark: Native u32 Multiply-Add Baseline
// ============================================================================
// This benchmark measures the raw throughput of native u32 operations.

struct BenchParams {
    iterations: u32,
    seed: u32,
    _pad0: u32,
    _pad1: u32,
}

@group(0) @binding(0) var<storage, read> input: array<u32>;
@group(0) @binding(1) var<storage, read_write> output: array<u32>;
@group(0) @binding(2) var<uniform> params: BenchParams;

@compute @workgroup_size(64)
fn bench_u32_baseline(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let tid = global_id.x;

    // Initialize with thread-unique seed
    var acc: u32 = params.seed ^ tid;
    var a: u32 = input[tid % 16u];
    var b: u32 = input[(tid + 8u) % 16u];

    // Main benchmark loop - multiply-add operations
    for (var i: u32 = 0u; i < params.iterations; i = i + 1u) {
        // Multiply-add: acc = acc * a + b
        acc = acc * a + b;

        // Data-dependent modification to prevent optimization
        a = a ^ (acc & 0xFFu);
    }

    // Write result to prevent dead code elimination
    output[tid] = acc;
}

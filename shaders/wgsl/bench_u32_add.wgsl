// ============================================================================
// Benchmark: Native u32 Addition
// ============================================================================
// This benchmark measures the raw throughput of native u32 addition.

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
fn bench_u32_add(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let tid = global_id.x;

    // Initialize with thread-unique seed
    var acc: u32 = params.seed ^ tid;
    var b: u32 = input[(tid + 8u) % 16u];

    // Main benchmark loop - addition operations
    for (var i: u32 = 0u; i < params.iterations; i = i + 1u) {
        // Addition: acc = acc + b
        acc = acc + b;

        // Data-dependent modification to prevent optimization
        b = b ^ (acc & 0xFFu);
    }

    // Write result to prevent dead code elimination
    output[tid] = acc;
}

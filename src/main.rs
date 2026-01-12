use clap::Parser;
use console::Style;
use field_ops_benchmarks::{
    config::BenchmarkConfig, reporter, results::BenchmarkReport, tui::InteractiveTui, Backend,
    Operation,
};

#[derive(Parser, Debug)]
#[command(name = "field-ops-bench")]
#[command(about = "GPU benchmark for u256/field arithmetic operations")]
struct Args {
    /// Run in batch mode (non-interactive)
    #[arg(long)]
    batch: bool,

    /// Backend to use (metal, webgpu, vulkan)
    #[arg(long, short = 'b')]
    backend: Option<String>,

    /// Operation to benchmark
    #[arg(long, short = 'o')]
    op: Option<String>,

    /// Workgroup size
    #[arg(long, short = 'w', default_value = "64")]
    workgroup: u32,

    /// Operations per thread
    #[arg(long, default_value = "10000")]
    ops: u32,

    /// Measurement iterations
    #[arg(long, short = 'i', default_value = "100")]
    iterations: u32,

    /// Output file for JSON results
    #[arg(long)]
    output: Option<String>,
}

fn main() {
    let args = Args::parse();

    if args.batch {
        run_batch_mode(args);
    } else {
        run_interactive_mode();
    }
}

fn run_interactive_mode() {
    let tui = InteractiveTui::new();

    if let Some(selection) = tui.quick_run() {
        println!();
        println!(
            "Running benchmarks for {} backend...",
            selection.backend.name()
        );
        println!(
            "Operations: {:?}",
            selection
                .operations
                .iter()
                .map(|o| o.name())
                .collect::<Vec<_>>()
        );
        println!("Workgroup size: {}", selection.config.workgroup_size);
        println!("Ops per thread: {}", selection.config.ops_per_thread);
        println!();

        // Run benchmarks
        let report = run_benchmarks(selection.backend, &selection.operations, &selection.config);

        // Print results
        reporter::print_results(&report);

        // Ask to save
        if let Some(filename) = tui.ask_save_results() {
            if let Err(e) = reporter::export_json(&report, &filename) {
                eprintln!("Failed to save results: {}", e);
            } else {
                println!("Results saved to {}", filename);
            }
        }
    }
}

fn run_batch_mode(args: Args) {
    let backend = match args.backend.as_deref() {
        Some("metal") => Backend::Metal,
        Some("webgpu") => Backend::WebGPU,
        Some("vulkan") => Backend::Vulkan,
        Some(other) => {
            eprintln!("Unknown backend: {}", other);
            eprintln!("Available: metal, webgpu, vulkan");
            return;
        }
        None => {
            // Use first available backend
            if let Some(b) = Backend::available().first() {
                *b
            } else {
                eprintln!("No GPU backends available!");
                return;
            }
        }
    };

    if !backend.is_available() {
        eprintln!("Backend {} is not available on this system", backend.name());
        return;
    }

    let operations: Vec<Operation> = match args.op.as_deref() {
        Some("all") | None => Operation::available_for(backend),
        Some(op_name) => {
            if let Some(op) = Operation::all().into_iter().find(|o| o.name() == op_name) {
                vec![op]
            } else {
                eprintln!("Unknown operation: {}", op_name);
                eprintln!(
                    "Available: {}",
                    Operation::all()
                        .iter()
                        .map(|o| o.name())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                return;
            }
        }
    };

    let config = BenchmarkConfig::default()
        .with_workgroup_size(args.workgroup)
        .with_ops_per_thread(args.ops)
        .with_iterations(args.iterations);

    let report = run_benchmarks(backend, &operations, &config);

    reporter::print_results(&report);

    if let Some(output) = args.output {
        if output.ends_with(".csv") {
            if let Err(e) = reporter::export_csv(&report, &output) {
                eprintln!("Failed to save CSV: {}", e);
            }
        } else {
            if let Err(e) = reporter::export_json(&report, &output) {
                eprintln!("Failed to save JSON: {}", e);
            }
        }
    }
}

fn run_benchmarks(
    backend: Backend,
    operations: &[Operation],
    config: &BenchmarkConfig,
) -> BenchmarkReport {
    let mut report = BenchmarkReport::new(
        format!("{} Device", backend.name()),
        backend.name().to_string(),
    );

    let info_style = Style::new().dim();

    for op in operations {
        println!(
            "{}",
            info_style.apply_to(format!("  Running {}...", op.name()))
        );

        // TODO: Implement actual benchmark execution
        // For now, create placeholder results
        let result = placeholder_result(backend, *op, config);
        report.add_result(result);
    }

    report
}

/// Placeholder result until actual GPU execution is implemented
fn placeholder_result(
    backend: Backend,
    operation: Operation,
    config: &BenchmarkConfig,
) -> field_ops_benchmarks::results::BenchmarkResult {
    use std::time::Duration;

    // Generate fake timing data for demonstration
    let timings: Vec<Duration> = (0..config.measurement_iterations)
        .map(|i| {
            let base_ms = match operation {
                Operation::U32Baseline => 0.5,
                Operation::U64Native => 0.9,
                Operation::U64Emulated => 2.0,
                Operation::BigIntMul => 45.0,
                Operation::FieldMul => 52.0,
                Operation::FieldAdd => 1.0,
                Operation::FieldSub => 1.0,
            };
            // Add some variance
            let variance = (i as f64 / 100.0) * 0.1;
            Duration::from_secs_f64((base_ms + variance) / 1000.0)
        })
        .collect();

    field_ops_benchmarks::results::BenchmarkResult::from_timings(
        backend,
        operation,
        config.workgroup_size,
        config.total_threads(),
        config.ops_per_thread,
        &timings,
        Some(1.5), // Assume 1.5 GHz GPU clock
    )
}

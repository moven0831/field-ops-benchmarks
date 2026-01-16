use clap::Parser;
use console::Style;
use field_ops_benchmarks::{
    config::BenchmarkConfig, reporter, results::BenchmarkReport, tui::InteractiveTui, Backend,
    Operation,
};
use indicatif::{ProgressBar, ProgressStyle};

// Embedded metallib (compiled at build time)
#[cfg(feature = "metal")]
const METAL_LIB: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/benchmarks.metallib"));

#[derive(Parser, Debug)]
#[command(name = "field-ops-bench")]
#[command(about = "GPU benchmark for u256/field arithmetic operations")]
struct Args {
    /// Run in batch mode (non-interactive)
    #[arg(long)]
    batch: bool,

    /// Run comparison between all available backends
    #[arg(long)]
    compare: bool,

    /// Backend to use (metal, webgpu)
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

    /// Run full benchmark (10000 ops, 100 iterations) - takes much longer
    #[arg(long)]
    full: bool,
}

fn main() {
    let args = Args::parse();

    if args.compare {
        run_comparison_mode(args);
    } else if args.batch {
        run_batch_mode(args);
    } else {
        run_interactive_mode();
    }
}

fn run_interactive_mode() {
    let tui = InteractiveTui::new();

    if let Some(selection) = tui.quick_run() {
        let mut all_reports: Vec<BenchmarkReport> = Vec::new();

        for backend in &selection.backends {
            println!();
            println!("Running benchmarks for {} backend...", backend.name());

            // Filter operations to those available for this backend
            let backend_ops: Vec<Operation> = Operation::available_for(*backend)
                .into_iter()
                .filter(|op| selection.operations.contains(op))
                .collect();

            println!(
                "Operations: {:?}",
                backend_ops.iter().map(|o| o.name()).collect::<Vec<_>>()
            );
            println!("Workgroup size: {}", selection.config.workgroup_size);
            println!(
                "Auto-calibrate: {}",
                if selection.config.auto_calibrate {
                    "enabled"
                } else {
                    "disabled"
                }
            );
            println!();

            // Run benchmarks
            let report = run_benchmarks(*backend, &backend_ops, &selection.config);

            // Print results
            reporter::print_results(&report);
            all_reports.push(report);
        }

        // Print comparison if multiple backends
        if all_reports.len() > 1 {
            reporter::print_comparison(&all_reports);
        }

        // Ask to save
        if let Some(filename) = tui.ask_save_results() {
            let combined = reporter::merge_reports(&all_reports);
            if let Err(e) = reporter::export_json(&combined, &filename) {
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
        Some(other) => {
            eprintln!("Unknown backend: {}", other);
            eprintln!("Available: metal, webgpu");
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

    let config = if args.full {
        // Full benchmark mode: high ops, many iterations, no auto-calibrate
        BenchmarkConfig::default()
            .with_workgroup_size(args.workgroup)
            .with_ops_per_thread(10_000)
            .with_iterations(100)
            .with_auto_calibrate(false)
    } else {
        // Default: use auto-calibration for fast benchmarks
        BenchmarkConfig::default()
            .with_workgroup_size(args.workgroup)
            .with_ops_per_thread(args.ops)
            .with_iterations(args.iterations)
    };

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

fn run_comparison_mode(args: Args) {
    let header_style = Style::new().bold().cyan();

    println!();
    println!(
        "{}",
        header_style.apply_to("Running comparison across all available backends...")
    );
    println!();

    let available_backends = Backend::available();
    if available_backends.is_empty() {
        eprintln!("No GPU backends available!");
        return;
    }

    let config = if args.full {
        // Full benchmark mode: high ops, many iterations, no auto-calibrate
        BenchmarkConfig::default()
            .with_workgroup_size(args.workgroup)
            .with_ops_per_thread(10_000)
            .with_iterations(100)
            .with_auto_calibrate(false)
    } else {
        // Default: use auto-calibration for fast benchmarks
        BenchmarkConfig::default()
            .with_workgroup_size(args.workgroup)
            .with_ops_per_thread(args.ops)
            .with_iterations(args.iterations)
    };

    let mut all_reports: Vec<BenchmarkReport> = Vec::new();

    for backend in &available_backends {
        println!();
        println!(
            "{}",
            header_style.apply_to(format!("=== {} Backend ===", backend.name()))
        );

        let operations: Vec<Operation> = match args.op.as_deref() {
            Some("all") | None => Operation::available_for(*backend),
            Some(op_name) => {
                if let Some(op) = Operation::all().into_iter().find(|o| o.name() == op_name) {
                    if Operation::available_for(*backend).contains(&op) {
                        vec![op]
                    } else {
                        println!("  Operation {} not available for {}", op_name, backend.name());
                        continue;
                    }
                } else {
                    continue;
                }
            }
        };

        let report = run_benchmarks(*backend, &operations, &config);
        reporter::print_results(&report);
        all_reports.push(report);
    }

    // Print comparison summary
    if all_reports.len() > 1 {
        reporter::print_comparison(&all_reports);
    }

    // Export combined results if requested
    if let Some(output) = args.output {
        let combined = reporter::merge_reports(&all_reports);
        if output.ends_with(".csv") {
            if let Err(e) = reporter::export_csv(&combined, &output) {
                eprintln!("Failed to save CSV: {}", e);
            } else {
                println!("Results saved to {}", output);
            }
        } else {
            if let Err(e) = reporter::export_json(&combined, &output) {
                eprintln!("Failed to save JSON: {}", e);
            } else {
                println!("Results saved to {}", output);
            }
        }
    }
}

fn run_benchmarks(
    backend: Backend,
    operations: &[Operation],
    config: &BenchmarkConfig,
) -> BenchmarkReport {
    match backend {
        #[cfg(feature = "metal")]
        Backend::Metal => run_metal_benchmarks(operations, config),

        #[cfg(feature = "webgpu")]
        Backend::WebGPU => run_webgpu_benchmarks(operations, config),

        #[allow(unreachable_patterns)]
        _ => {
            eprintln!("Backend {} not compiled in", backend.name());
            BenchmarkReport::new("Unknown".to_string(), "Unknown".to_string())
        }
    }
}

#[cfg(feature = "metal")]
fn run_metal_benchmarks(operations: &[Operation], config: &BenchmarkConfig) -> BenchmarkReport {
    use field_ops_benchmarks::metal::MetalRunner;

    let error_style = Style::new().red();

    // Create Metal runner
    let mut runner = match MetalRunner::new() {
        Ok(r) => r,
        Err(e) => {
            eprintln!(
                "{}",
                error_style.apply_to(format!("Failed to create Metal runner: {}", e))
            );
            return BenchmarkReport::new("Unknown".to_string(), "Metal".to_string());
        }
    };

    let device_name = runner.device_name();
    println!("Device: {}", device_name);

    // Load the embedded metallib
    if let Err(e) = runner.load_library_data(METAL_LIB) {
        eprintln!(
            "{}",
            error_style.apply_to(format!("Failed to load Metal library: {}", e))
        );
        return BenchmarkReport::new(device_name, "Metal".to_string());
    }

    let mut report = BenchmarkReport::new(device_name, "Metal".to_string());

    // Run each benchmark with spinner
    for op in operations {
        // Create spinner for each operation
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg} [{elapsed_precise}]")
                .unwrap(),
        );
        spinner.set_message(format!("Running {}...", op.name()));
        spinner.enable_steady_tick(std::time::Duration::from_millis(100));

        // Get operation-specific config
        let op_config = config.for_operation(*op);

        match runner.run_benchmark(*op, &op_config) {
            Ok(result) => {
                let time_ms = result.min_ns as f64 / 1_000_000.0;
                spinner.finish_with_message(format!("✓ {} ({:.2}ms)", op.name(), time_ms));
                report.add_result(result);
            }
            Err(e) => {
                spinner.finish_with_message(format!("✗ {} failed: {}", op.name(), e));
            }
        }
    }

    report
}

#[cfg(feature = "webgpu")]
fn run_webgpu_benchmarks(operations: &[Operation], config: &BenchmarkConfig) -> BenchmarkReport {
    use field_ops_benchmarks::webgpu::WebGpuRunner;

    let error_style = Style::new().red();

    // Create WebGPU runner
    let runner = match WebGpuRunner::new() {
        Ok(r) => r,
        Err(e) => {
            eprintln!(
                "{}",
                error_style.apply_to(format!("Failed to create WebGPU runner: {}", e))
            );
            return BenchmarkReport::new("Unknown".to_string(), "WebGPU".to_string());
        }
    };

    let device_name = runner.device_name();
    println!("Device: {}", device_name);

    let mut report = BenchmarkReport::new(device_name, "WebGPU".to_string());

    // Run each benchmark with spinner
    for op in operations {
        // Create spinner for each operation
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg} [{elapsed_precise}]")
                .unwrap(),
        );
        spinner.set_message(format!("Running {}...", op.name()));
        spinner.enable_steady_tick(std::time::Duration::from_millis(100));

        // Get operation-specific config
        let op_config = config.for_operation(*op);

        match runner.run_benchmark(*op, &op_config) {
            Ok(result) => {
                let time_ms = result.min_ns as f64 / 1_000_000.0;
                spinner.finish_with_message(format!("✓ {} ({:.2}ms)", op.name(), time_ms));
                report.add_result(result);
            }
            Err(e) => {
                spinner.finish_with_message(format!("✗ {} failed: {}", op.name(), e));
            }
        }
    }

    report
}

/// Placeholder benchmarks for backends not yet implemented
#[allow(dead_code)]
fn run_placeholder_benchmarks(
    backend: Backend,
    operations: &[Operation],
    config: &BenchmarkConfig,
) -> BenchmarkReport {
    use std::time::Duration;

    let info_style = Style::new().dim();

    let mut report = BenchmarkReport::new(
        format!("{} Device (placeholder)", backend.name()),
        backend.name().to_string(),
    );

    for op in operations {
        println!(
            "{}",
            info_style.apply_to(format!("  Running {} (placeholder)...", op.name()))
        );

        // Generate fake timing data
        let timings: Vec<Duration> = (0..config.measurement_iterations)
            .map(|i| {
                let base_ms = match op {
                    Operation::U32Baseline => 0.5,
                    Operation::U64Native => 0.9,
                    Operation::U64Emulated => 2.0,
                    Operation::FieldMul => 52.0,
                    Operation::FieldAdd => 1.0,
                    Operation::FieldSub => 1.0,
                };
                let variance = (i as f64 / 100.0) * 0.1;
                Duration::from_secs_f64((base_ms + variance) / 1000.0)
            })
            .collect();

        let result = field_ops_benchmarks::results::BenchmarkResult::from_timings(
            backend,
            *op,
            config.workgroup_size,
            config.total_threads(),
            config.ops_per_thread,
            &timings,
            Some(1.5),
        );
        report.add_result(result);
    }

    report
}

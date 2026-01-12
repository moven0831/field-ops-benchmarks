use crate::results::{BenchmarkReport, BenchmarkResult};
use console::Style;
use std::io::Write;

/// Print benchmark results to console
pub fn print_results(report: &BenchmarkReport) {
    let header_style = Style::new().bold().cyan();
    let label_style = Style::new().bold();
    let value_style = Style::new().green();

    println!();
    println!(
        "{}",
        header_style.apply_to(
            "================================================================================"
        )
    );
    println!(
        "{}",
        header_style.apply_to("                        FIELD OPS BENCHMARK RESULTS")
    );
    println!(
        "{}",
        header_style.apply_to(
            "================================================================================"
        )
    );
    println!();

    println!(
        "{}: {} ({})",
        label_style.apply_to("Device"),
        value_style.apply_to(&report.device_name),
        &report.device_vendor
    );
    println!();

    // Table header
    println!(
        "{:<25} {:>10} {:>12} {:>12} {:>12}",
        label_style.apply_to("Benchmark"),
        label_style.apply_to("WG Size"),
        label_style.apply_to("Min (ms)"),
        label_style.apply_to("GOP/s"),
        label_style.apply_to("Cycles/Op"),
    );
    println!("{}", "-".repeat(80));

    // Results
    for result in &report.results {
        let cycles = result
            .cycles_per_op
            .map(|c| format!("{:.2}", c))
            .unwrap_or_else(|| "-".to_string());

        println!(
            "{:<25} {:>10} {:>12.3} {:>12.2} {:>12}",
            result.operation,
            result.workgroup_size,
            result.min_ms(),
            result.gops_per_second,
            cycles,
        );
    }

    println!();

    // Overhead analysis
    if let Some(overhead) = report.u64_overhead() {
        println!("{}", label_style.apply_to("Overhead Analysis:"));
        println!("- u64 emulated vs native: {:.1}x slower", overhead);
    }

    println!(
        "{}",
        header_style.apply_to(
            "================================================================================"
        )
    );
    println!();
}

/// Print a single result line (for live updates)
pub fn print_result_line(result: &BenchmarkResult) {
    let cycles = result
        .cycles_per_op
        .map(|c| format!("{:.2}", c))
        .unwrap_or_else(|| "-".to_string());

    println!(
        "{:<25} {:>10} {:>12.3} {:>12.2} {:>12}",
        result.operation,
        result.workgroup_size,
        result.min_ms(),
        result.gops_per_second,
        cycles,
    );
}

/// Export results to JSON file
pub fn export_json(report: &BenchmarkReport, path: &str) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(report)?;
    std::fs::write(path, json)
}

/// Export results to CSV file
pub fn export_csv(report: &BenchmarkReport, path: &str) -> std::io::Result<()> {
    let mut file = std::fs::File::create(path)?;

    // Header
    writeln!(
        file,
        "backend,operation,workgroup_size,total_threads,ops_per_thread,total_operations,min_ns,max_ns,mean_ns,std_dev_ns,gops_per_second,cycles_per_op"
    )?;

    // Data
    for r in &report.results {
        let cycles = r
            .cycles_per_op
            .map(|c| format!("{:.4}", c))
            .unwrap_or_default();

        writeln!(
            file,
            "{},{},{},{},{},{},{},{},{:.2},{:.2},{:.4},{}",
            r.backend,
            r.operation,
            r.workgroup_size,
            r.total_threads,
            r.ops_per_thread,
            r.total_operations,
            r.min_ns,
            r.max_ns,
            r.mean_ns,
            r.std_dev_ns,
            r.gops_per_second,
            cycles,
        )?;
    }

    Ok(())
}

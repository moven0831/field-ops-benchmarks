use crate::results::{BenchmarkReport, BenchmarkResult};
use console::Style;
use std::io::Write;

/// Get equivalent operation names for comparison matching
fn get_equivalent_ops(op: &str) -> Vec<&'static str> {
    match op {
        "u64_add_native" | "u64_add_emulated" | "u64_add" => {
            vec!["u64_add_native", "u64_add_emulated"]
        }
        _ => vec![],
    }
}

/// Get canonical display name for an operation
fn get_display_name(op: &str) -> &str {
    match op {
        "u64_add_native" | "u64_add_emulated" => "u64_add",
        _ => op,
    }
}

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

/// Print comparison between multiple backend reports
pub fn print_comparison(reports: &[BenchmarkReport]) {
    let header_style = Style::new().bold().cyan();
    let label_style = Style::new().bold();
    let good_style = Style::new().green();
    let warn_style = Style::new().yellow();

    println!();
    println!(
        "{}",
        header_style.apply_to(
            "================================================================================"
        )
    );
    println!(
        "{}",
        header_style.apply_to("                          COMPARISON SUMMARY")
    );
    println!(
        "{}",
        header_style.apply_to(
            "================================================================================"
        )
    );
    println!();

    // Collect all unique operations across reports using canonical names
    let mut all_ops: Vec<String> = Vec::new();
    for report in reports {
        for result in &report.results {
            let canonical = get_display_name(&result.operation).to_string();
            if !all_ops.contains(&canonical) {
                all_ops.push(canonical);
            }
        }
    }

    // Sort operations in a logical order
    fn get_operation_order(op: &str) -> usize {
        match op {
            "u32_add" => 0,
            "u64_add" => 1,
            "m31_field_add" => 2,
            "m31_field_mul" => 3,
            "bn254_field_add" => 4,
            "bn254_field_mul" => 5,
            _ => 100,
        }
    }
    all_ops.sort_by_key(|op| get_operation_order(op));

    // Print header with backend names
    print!("{:<20}", label_style.apply_to("Operation"));
    for report in reports {
        print!(" {:>15}", label_style.apply_to(&report.device_vendor));
    }
    if reports.len() == 2 {
        print!(" {:>12}", label_style.apply_to("Ratio"));
    }
    println!();
    println!(
        "{}",
        "-".repeat(20 + reports.len() * 16 + if reports.len() == 2 { 13 } else { 0 })
    );

    // Print comparison for each operation
    for op in &all_ops {
        print!("{:<20}", op);

        let mut gops_values: Vec<Option<f64>> = Vec::new();

        // Get equivalent operation names for matching
        let equivalents = get_equivalent_ops(op);

        for report in reports {
            // Search for the operation or any equivalent
            let result = report
                .results
                .iter()
                .find(|r| &r.operation == op || equivalents.contains(&r.operation.as_str()));

            if let Some(result) = result {
                print!(" {:>12.2} GOP/s", result.gops_per_second);
                gops_values.push(Some(result.gops_per_second));
            } else {
                print!(" {:>15}", "-");
                gops_values.push(None);
            }
        }

        // Calculate ratio if we have exactly 2 backends with values
        if reports.len() == 2 {
            if let (Some(Some(v1)), Some(Some(v2))) = (gops_values.get(0), gops_values.get(1)) {
                if *v2 > 0.0 {
                    let ratio = v1 / v2;
                    if ratio > 1.0 {
                        print!(" {}", good_style.apply_to(format!("{:>10.2}x", ratio)));
                    } else {
                        print!(" {}", warn_style.apply_to(format!("{:>10.2}x", ratio)));
                    }
                }
            }
        }
        println!();
    }

    println!();

    // Print legend
    if reports.len() == 2 {
        println!(
            "{}",
            label_style
                .apply_to("Ratio: First backend / Second backend (higher = first is faster)")
        );
    }

    println!(
        "{}",
        header_style.apply_to(
            "================================================================================"
        )
    );
    println!();
}

/// Merge multiple reports into a single combined report
pub fn merge_reports(reports: &[BenchmarkReport]) -> BenchmarkReport {
    let device_names: Vec<&str> = reports.iter().map(|r| r.device_name.as_str()).collect();
    let vendors: Vec<&str> = reports.iter().map(|r| r.device_vendor.as_str()).collect();

    let mut combined = BenchmarkReport::new(device_names.join(" + "), vendors.join(" + "));

    for report in reports {
        for result in &report.results {
            combined.add_result(result.clone());
        }
    }

    combined
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

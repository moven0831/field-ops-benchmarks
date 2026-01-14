use crate::config::{BenchmarkConfig, WORKGROUP_SIZES};
use crate::{Backend, Operation};
use console::Style;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, MultiSelect, Select};

/// Interactive TUI for benchmark configuration
pub struct InteractiveTui {
    theme: ColorfulTheme,
}

impl Default for InteractiveTui {
    fn default() -> Self {
        Self::new()
    }
}

impl InteractiveTui {
    pub fn new() -> Self {
        Self {
            theme: ColorfulTheme::default(),
        }
    }

    /// Display the welcome banner
    pub fn show_banner(&self) {
        let header_style = Style::new().bold().cyan();

        println!();
        println!(
            "{}",
            header_style.apply_to(
                "================================================================================"
            )
        );
        println!(
            "{}",
            header_style.apply_to("                     FIELD OPS BENCHMARK - Interactive Mode")
        );
        println!(
            "{}",
            header_style.apply_to(
                "================================================================================"
            )
        );
        println!();
    }

    /// Main menu loop
    pub fn run(&self) -> Option<BenchmarkSelection> {
        self.show_banner();

        loop {
            let choices = &[
                "Select Backend(s)",
                "Select Operation",
                "Configure Parameters",
                "Run Benchmark",
                "Exit",
            ];

            let selection = Select::with_theme(&self.theme)
                .with_prompt("Choose an option")
                .items(choices)
                .default(0)
                .interact()
                .ok()?;

            match selection {
                0 => {
                    if let Some(backends) = self.select_backends() {
                        // Get operations from first backend for selection UI
                        // Operations will be filtered per-backend during execution
                        let first_backend = backends[0];
                        return Some(BenchmarkSelection {
                            backends,
                            operations: self.select_operations(first_backend)?,
                            config: self.configure_parameters()?,
                        });
                    }
                }
                4 => return None,
                _ => continue,
            }
        }
    }

    /// Quick run - select backend(s) and run all operations with defaults
    pub fn quick_run(&self) -> Option<BenchmarkSelection> {
        self.show_banner();

        let backends = self.select_backends()?;

        // Collect all unique operations available across selected backends
        let mut operations: Vec<Operation> = backends
            .iter()
            .flat_map(|b| Operation::available_for(*b))
            .collect();
        operations.sort_by_key(|op| op.name());
        operations.dedup();

        let config = BenchmarkConfig::default();

        Some(BenchmarkSelection {
            backends,
            operations,
            config,
        })
    }

    /// Select GPU backend(s) - supports multi-selection
    pub fn select_backends(&self) -> Option<Vec<Backend>> {
        let available = Backend::available();

        if available.is_empty() {
            println!("No GPU backends available!");
            return None;
        }

        let all_backends = Backend::all();
        let items: Vec<String> = all_backends
            .iter()
            .map(|b| {
                if b.is_available() {
                    format!("{} (available)", b.name())
                } else {
                    format!("{} (not available)", b.name())
                }
            })
            .collect();

        // Default: first available backend is selected
        let defaults: Vec<bool> = all_backends.iter().map(|b| b.is_available()).collect();

        let selections = MultiSelect::with_theme(&self.theme)
            .with_prompt("Select GPU Backend(s) (space to toggle, enter to confirm)")
            .items(&items)
            .defaults(&defaults)
            .interact()
            .ok()?;

        if selections.is_empty() {
            // Default to first available if nothing selected
            return Some(vec![available[0]]);
        }

        let backends: Vec<Backend> = selections
            .iter()
            .filter_map(|&i| {
                let backend = all_backends[i];
                if backend.is_available() {
                    Some(backend)
                } else {
                    None
                }
            })
            .collect();

        if backends.is_empty() {
            println!("No available backends selected!");
            None
        } else {
            Some(backends)
        }
    }

    /// Select operations to benchmark
    pub fn select_operations(&self, backend: Backend) -> Option<Vec<Operation>> {
        let available = Operation::available_for(backend);

        let items: Vec<String> = available
            .iter()
            .map(|op| format!("{} - {}", op.name(), op.description()))
            .collect();

        // Add "All" option
        let mut all_items = vec!["All operations".to_string()];
        all_items.extend(items);

        let selections = MultiSelect::with_theme(&self.theme)
            .with_prompt("Select operations to benchmark (space to select, enter to confirm)")
            .items(&all_items)
            .interact()
            .ok()?;

        if selections.is_empty() {
            // Default to all if nothing selected
            return Some(available);
        }

        if selections.contains(&0) {
            // "All" was selected
            return Some(available);
        }

        // Map selections back to operations (subtract 1 for "All" option)
        let ops: Vec<Operation> = selections
            .iter()
            .filter_map(|&i| {
                if i > 0 {
                    available.get(i - 1).copied()
                } else {
                    None
                }
            })
            .collect();

        if ops.is_empty() {
            Some(available)
        } else {
            Some(ops)
        }
    }

    /// Configure benchmark parameters
    pub fn configure_parameters(&self) -> Option<BenchmarkConfig> {
        let use_defaults = Confirm::with_theme(&self.theme)
            .with_prompt("Use default parameters? (10000 ops, 100 iterations)")
            .default(true)
            .interact()
            .ok()?;

        if use_defaults {
            return Some(BenchmarkConfig::default());
        }

        // Workgroup size
        let wg_items: Vec<String> = WORKGROUP_SIZES.iter().map(|s| s.to_string()).collect();
        let wg_selection = Select::with_theme(&self.theme)
            .with_prompt("Select workgroup size")
            .items(&wg_items)
            .default(0)
            .interact()
            .ok()?;
        let workgroup_size = WORKGROUP_SIZES[wg_selection];

        // Operations per thread
        let ops_per_thread: u32 = Input::with_theme(&self.theme)
            .with_prompt("Operations per thread")
            .default(10_000)
            .interact()
            .ok()?;

        // Measurement iterations
        let iterations: u32 = Input::with_theme(&self.theme)
            .with_prompt("Measurement iterations")
            .default(100)
            .interact()
            .ok()?;

        Some(BenchmarkConfig {
            ops_per_thread,
            workgroup_size,
            measurement_iterations: iterations,
            ..BenchmarkConfig::default()
        })
    }

    /// Ask whether to save results
    pub fn ask_save_results(&self) -> Option<String> {
        let save = Confirm::with_theme(&self.theme)
            .with_prompt("Save results to JSON?")
            .default(false)
            .interact()
            .ok()?;

        if save {
            let filename: String = Input::with_theme(&self.theme)
                .with_prompt("Filename")
                .default("results.json".to_string())
                .interact()
                .ok()?;
            Some(filename)
        } else {
            None
        }
    }
}

/// User's benchmark selection
#[derive(Debug, Clone)]
pub struct BenchmarkSelection {
    pub backends: Vec<Backend>,
    pub operations: Vec<Operation>,
    pub config: BenchmarkConfig,
}

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Compile Metal shaders on macOS
    #[cfg(target_os = "macos")]
    compile_metal_shaders(&out_dir);

    // Compile SPIR-V shaders for Vulkan (when implemented)
    // compile_vulkan_shaders(&out_dir);

    println!("cargo:rerun-if-changed=shaders/");
}

#[cfg(target_os = "macos")]
fn compile_metal_shaders(out_dir: &PathBuf) {
    use std::fs;

    let shader_dir = PathBuf::from("shaders/metal");
    if !shader_dir.exists() {
        println!(
            "cargo:warning=Metal shader directory not found, skipping Metal shader compilation"
        );
        return;
    }

    // Find all .metal files
    let metal_files: Vec<_> = fs::read_dir(&shader_dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path()
                        .extension()
                        .map(|ext| ext == "metal")
                        .unwrap_or(false)
                })
                .map(|e| e.path())
                .collect()
        })
        .unwrap_or_default();

    if metal_files.is_empty() {
        println!("cargo:warning=No Metal shader files found");
        return;
    }

    let mut ir_files = Vec::new();

    // Compile each .metal file to .ir
    for metal_file in &metal_files {
        let file_stem = metal_file.file_stem().unwrap().to_str().unwrap();
        let ir_path = out_dir.join(format!("{}.ir", file_stem));

        let status = Command::new("xcrun")
            .args(["-sdk", "macosx", "metal"])
            .args(["-O3"]) // Full optimization
            .args(["-c", metal_file.to_str().unwrap()])
            .args(["-I", shader_dir.to_str().unwrap()])
            .args(["-o", ir_path.to_str().unwrap()])
            .status();

        match status {
            Ok(s) if s.success() => {
                ir_files.push(ir_path);
                println!("cargo:warning=Compiled {}", metal_file.display());
            }
            Ok(s) => {
                println!(
                    "cargo:warning=Failed to compile {}: exit code {}",
                    metal_file.display(),
                    s.code().unwrap_or(-1)
                );
            }
            Err(e) => {
                println!(
                    "cargo:warning=Failed to run xcrun for {}: {}",
                    metal_file.display(),
                    e
                );
            }
        }
    }

    // Link all .ir files into a single .metallib
    if !ir_files.is_empty() {
        let metallib_path = out_dir.join("benchmarks.metallib");
        let mut cmd = Command::new("xcrun");
        cmd.args(["-sdk", "macosx", "metallib"]);
        for ir_file in &ir_files {
            cmd.arg(ir_file.to_str().unwrap());
        }
        cmd.args(["-o", metallib_path.to_str().unwrap()]);

        match cmd.status() {
            Ok(s) if s.success() => {
                println!(
                    "cargo:warning=Created metallib at {}",
                    metallib_path.display()
                );
            }
            Ok(s) => {
                println!(
                    "cargo:warning=Failed to create metallib: exit code {}",
                    s.code().unwrap_or(-1)
                );
            }
            Err(e) => {
                println!("cargo:warning=Failed to run metallib: {}", e);
            }
        }
    }
}

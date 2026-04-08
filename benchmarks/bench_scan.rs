//! Benchmarks for codebase scanning performance.
//!
//! This is a simple benchmark harness using only the standard library.
//! For production use, consider switching to `criterion` or `divan` and
//! wiring this up as a `[[bench]]` target in Cargo.toml.
//!
//! Run with:
//!   rustc --edition 2021 -O benchmarks/bench_scan.rs -o target/bench_scan && ./target/bench_scan
//!
//! Or integrate into the workspace (see below).

use std::time::Instant;

fn main() {
    println!("=== codemod-pilot Benchmarks ===\n");

    // Benchmark 1: File walking speed
    bench_file_walking();

    // Benchmark 2: Pattern matching speed (string-level simulation)
    bench_pattern_matching();

    // Benchmark 3: Simple diff-like comparison speed
    bench_line_comparison();

    println!("\n=== Benchmarks complete ===");
}

/// Benchmark walking the current directory tree.
fn bench_file_walking() {
    let target_dir = std::env::current_dir().unwrap();
    let start = Instant::now();

    let mut file_count: usize = 0;
    walk_dir(&target_dir, &mut file_count);

    let duration = start.elapsed();
    println!(
        "File walking : {:>6} files in {:?} ({:.0} files/sec)",
        file_count,
        duration,
        file_count as f64 / duration.as_secs_f64().max(1e-9)
    );
}

/// Recursively walk a directory, counting files.
fn walk_dir(dir: &std::path::Path, count: &mut usize) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Skip hidden directories and common large directories.
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if name.starts_with('.') || name == "target" || name == "node_modules" {
                    continue;
                }
                walk_dir(&path, count);
            } else {
                *count += 1;
            }
        }
    }
}

/// Benchmark simple string-level pattern matching.
///
/// In a real integration this would use `PatternMatcher` with a tree-sitter
/// grammar, but here we measure the baseline overhead of string scanning.
fn bench_pattern_matching() {
    // Generate a large synthetic source file.
    let line = "    let result = fetchUserInfo({ userId: id });\n";
    let source: String = line.repeat(10_000); // ~500 KB
    let needle = "fetchUserInfo";

    let start = Instant::now();
    let mut match_count: usize = 0;
    for line_text in source.lines() {
        if line_text.contains(needle) {
            match_count += 1;
        }
    }
    let duration = start.elapsed();

    println!(
        "Pattern match: {:>6} matches in {:?} ({} lines scanned)",
        match_count,
        duration,
        source.lines().count()
    );
}

/// Benchmark a naive line-by-line comparison (simulating diff generation).
fn bench_line_comparison() {
    let original_line = "let x = oldFunction(arg);\n";
    let changed_line = "let x = newFunction(arg);\n";
    let unchanged_line = "// unchanged code\n";

    // Build a ~1000-line file with changes every 10 lines.
    let mut original = String::new();
    let mut transformed = String::new();
    for i in 0..1000 {
        if i % 10 == 0 {
            original.push_str(original_line);
            transformed.push_str(changed_line);
        } else {
            original.push_str(unchanged_line);
            transformed.push_str(unchanged_line);
        }
    }

    let iterations = 1_000;
    let start = Instant::now();
    for _ in 0..iterations {
        let orig_lines: Vec<&str> = original.lines().collect();
        let new_lines: Vec<&str> = transformed.lines().collect();
        let mut _diff_count = 0usize;
        for (a, b) in orig_lines.iter().zip(new_lines.iter()) {
            if a != b {
                _diff_count += 1;
            }
        }
    }
    let duration = start.elapsed();

    println!(
        "Line compare : {:>6} iterations in {:?} ({:.1} ops/sec)",
        iterations,
        duration,
        iterations as f64 / duration.as_secs_f64().max(1e-9)
    );
}

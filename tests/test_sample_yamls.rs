#![deny(clippy::all)]

use std::fs;
use std::path::Path;
use yamp::{emit, parse};

#[test]
fn test_all_sample_yamls() {
    let sample_dir = Path::new("sample_yamls");

    // Ensure the sample directory exists
    assert!(sample_dir.exists(), "sample_yamls directory not found");

    // Read all .yaml and .yml files in the directory
    let yaml_files: Vec<_> = fs::read_dir(sample_dir)
        .expect("Failed to read sample_yamls directory")
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_file() {
                let extension = path.extension()?.to_str()?;
                if extension == "yaml" || extension == "yml" {
                    Some(path)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    // Ensure we found some YAML files
    assert!(
        !yaml_files.is_empty(),
        "No YAML files found in sample_yamls directory"
    );

    println!("Testing {} YAML files", yaml_files.len());

    let mut successful = 0;
    let mut failed = 0;

    for yaml_file in yaml_files {
        let file_name = yaml_file.file_name().unwrap().to_string_lossy();
        print!("Testing {}: ", file_name);

        // Read the file content
        let content = match fs::read_to_string(&yaml_file) {
            Ok(content) => content,
            Err(e) => {
                println!("FAILED - Could not read file: {}", e);
                failed += 1;
                continue;
            }
        };

        // Try to parse the YAML
        match parse(&content) {
            Ok(parsed) => {
                // Try to emit it back to ensure roundtrip works
                let emitted = emit(&parsed);

                // Try to parse the emitted content to ensure it's valid
                match parse(&emitted) {
                    Ok(_) => {
                        println!("PASSED");
                        successful += 1;
                    }
                    Err(e) => {
                        println!("FAILED - Roundtrip parse failed: {}", e);
                        println!("  Emitted content:");
                        for (i, line) in emitted.lines().enumerate() {
                            println!("    {}: {}", i + 1, line);
                        }
                        failed += 1;
                    }
                }
            }
            Err(e) => {
                println!("FAILED - Parse error: {}", e);
                println!("  Content preview:");
                for (i, line) in content.lines().take(5).enumerate() {
                    println!("    {}: {}", i + 1, line);
                }
                if content.lines().count() > 5 {
                    println!("    ... ({} more lines)", content.lines().count() - 5);
                }
                failed += 1;
            }
        }
    }

    println!("\nSummary: {} passed, {} failed", successful, failed);

    // Fail the test if any files failed to parse
    if failed > 0 {
        panic!("{} YAML files failed to parse or roundtrip", failed);
    }
}

#[test]
fn test_specific_sample_files() {
    let sample_dir = Path::new("sample_yamls");
    assert!(sample_dir.exists(), "sample_yamls directory not found");

    // Define critical files that must exist and work
    let critical_files = [
        "simple.yaml",
        "arrays.yaml",
        "nested.yaml",
        "minimal.yaml",
        "types.yaml",
    ];

    let mut found_files = Vec::new();

    // Read directory and find critical files
    for entry in fs::read_dir(sample_dir).expect("Failed to read sample_yamls directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();

        if path.is_file()
            && let Some(file_name) = path.file_name().and_then(|n| n.to_str())
            && critical_files.contains(&file_name)
        {
            found_files.push(path);
        }
    }

    // Ensure we found all critical files
    assert_eq!(
        found_files.len(),
        critical_files.len(),
        "Not all critical files found. Expected: {:?}, Found: {:?}",
        critical_files,
        found_files
    );

    // Test each critical file
    for file_path in found_files {
        let file_name = file_path.file_name().unwrap().to_string_lossy();
        println!("Testing critical file: {}", file_name);

        let content = fs::read_to_string(&file_path)
            .unwrap_or_else(|_| panic!("Could not read {:?}", file_path));

        let parsed =
            parse(&content).unwrap_or_else(|e| panic!("Failed to parse {:?}: {}", file_path, e));

        // Verify we can emit and re-parse
        let emitted = emit(&parsed);
        let _reparsed = parse(&emitted).unwrap_or_else(|e| {
            panic!(
                "Failed to reparse emitted content from {:?}: {}",
                file_path, e
            )
        });

        println!("✓ {} passed", file_name);
    }
}

#[test]
fn test_real_world_configs() {
    let sample_dir = Path::new("sample_yamls");
    assert!(sample_dir.exists(), "sample_yamls directory not found");

    // Define patterns for real-world config files
    let real_world_patterns = [
        "kubernetes",
        "docker",
        "github_actions",
        "app_config",
        "ansible",
    ];

    let mut real_world_files = Vec::new();

    // Read directory and find real-world config files
    for entry in fs::read_dir(sample_dir).expect("Failed to read sample_yamls directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();

        if path.is_file()
            && let Some(extension) = path.extension().and_then(|e| e.to_str())
            && (extension == "yaml" || extension == "yml")
            && let Some(file_name) = path.file_name().and_then(|n| n.to_str())
            && real_world_patterns
                .iter()
                .any(|pattern| file_name.contains(pattern))
        {
            real_world_files.push(path);
        }
    }

    // Ensure we found some real-world config files
    assert!(
        !real_world_files.is_empty(),
        "No real-world config files found matching patterns: {:?}",
        real_world_patterns
    );

    println!("Found {} real-world config files", real_world_files.len());

    for file_path in real_world_files {
        let file_name = file_path.file_name().unwrap().to_string_lossy();
        println!("Testing real-world config: {}", file_name);

        let content = fs::read_to_string(&file_path)
            .unwrap_or_else(|_| panic!("Could not read {:?}", file_path));

        let parsed = parse(&content).unwrap_or_else(|e| {
            println!("Content that failed to parse:");
            for (i, line) in content.lines().enumerate() {
                println!("  {}: {}", i + 1, line);
            }
            panic!("Failed to parse {:?}: {}", file_path, e)
        });

        // For real-world configs, just ensure they parse successfully
        // Roundtrip testing might be affected by formatting differences
        let _emitted = emit(&parsed);

        println!("✓ {} parsed successfully", file_name);
    }
}

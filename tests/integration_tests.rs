use anyhow::{Context, Result};
use colored::Colorize;
use dtolator::{generate, GenerateOptions, GeneratorType, InputType};
use similar::{ChangeTag, TextDiff};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Configuration for a single test case
#[derive(Debug, Clone)]
struct TestCase {
    name: String,
    input_file: String,
    command_args: Vec<String>,
    expected_dir: String,
}

/// Test suite runner
struct TestSuite {
    project_root: PathBuf,
    test_cases: Vec<TestCase>,
    refresh_mode: bool,
    passed_tests: usize,
    failed_tests: usize,
}

impl TestSuite {
    fn new() -> Self {
        let project_root = std::env::current_dir().expect("Failed to get current directory");

        let test_cases = vec![
            // Angular tests with full-sample.json
            TestCase {
                name: "Angular Full Sample".to_string(),
                input_file: "input-files/full-sample.json".to_string(),
                command_args: vec![
                    "--angular".to_string(),
                    "--zod".to_string(),
                    "--skip-file".to_string(),
                    "fill-url.ts".to_string(),
                ],
                expected_dir: "output-samples/angular/full-sample".to_string(),
            },
            TestCase {
                name: "Angular Simple Sample".to_string(),
                input_file: "input-files/simple-sample.json".to_string(),
                command_args: vec!["--angular".to_string(), "--zod".to_string()],
                expected_dir: "output-samples/angular/simple-sample".to_string(),
            },
            // Angular nested tests
            TestCase {
                name: "Angular Nested Test".to_string(),
                input_file: "input-files/full-sample.json".to_string(),
                command_args: vec![
                    "--angular".to_string(),
                    "--skip-file".to_string(),
                    "fill-url.ts".to_string(),
                ],
                expected_dir: "output-samples/angular-nested".to_string(),
            },
            // Comprehensive nested test
            TestCase {
                name: "Comprehensive Nested Test".to_string(),
                input_file: "input-files/full-sample.json".to_string(),
                command_args: vec![
                    "--angular".to_string(),
                    "--zod".to_string(),
                    "--skip-file".to_string(),
                    "fill-url.ts".to_string(),
                ],
                expected_dir: "output-samples/comprehensive-nested-test".to_string(),
            },
            // Nested test (TypeScript + Zod only)
            TestCase {
                name: "Nested Test".to_string(),
                input_file: "input-files/full-sample.json".to_string(),
                command_args: vec!["--typescript".to_string(), "--zod".to_string()],
                expected_dir: "output-samples/nested-test".to_string(),
            },
            // .NET test
            TestCase {
                name: "DotNet Test".to_string(),
                input_file: "input-files/simple-sample.json".to_string(),
                command_args: vec!["--dotnet".to_string()],
                expected_dir: "output-samples/dotnet".to_string(),
            },
            // Pydantic test
            TestCase {
                name: "Pydantic Test".to_string(),
                input_file: "input-files/simple-sample.json".to_string(),
                command_args: vec!["--pydantic".to_string()],
                expected_dir: "output-samples/pydantic".to_string(),
            },
            // Python TypedDict tests
            TestCase {
                name: "Python TypedDict Test".to_string(),
                input_file: "input-files/simple-sample.json".to_string(),
                command_args: vec!["--python-dict".to_string()],
                expected_dir: "output-samples/python-typed-dict".to_string(),
            },
            TestCase {
                name: "Python TypedDict Full Test".to_string(),
                input_file: "input-files/full-sample.json".to_string(),
                command_args: vec!["--python-dict".to_string()],
                expected_dir: "output-samples/python-typed-dict-full".to_string(),
            },
            // Promises tests
            TestCase {
                name: "Angular Promises with Zod".to_string(),
                input_file: "input-files/simple-sample.json".to_string(),
                command_args: vec![
                    "--angular".to_string(),
                    "--zod".to_string(),
                    "--promises".to_string(),
                    "--skip-file".to_string(),
                    "fill-url.ts".to_string(),
                ],
                expected_dir: "output-samples/angular-promises-with-zod".to_string(),
            },
            TestCase {
                name: "Angular Promises without Zod".to_string(),
                input_file: "input-files/simple-sample.json".to_string(),
                command_args: vec![
                    "--angular".to_string(),
                    "--promises".to_string(),
                    "--skip-file".to_string(),
                    "fill-url.ts".to_string(),
                ],
                expected_dir: "output-samples/angular-promises-no-zod".to_string(),
            },
            // JSON to TypeScript/Zod/Pydantic tests
            TestCase {
                name: "JSON Simple TypeScript".to_string(),
                input_file: "input-files/test-data-simple.json".to_string(),
                command_args: vec!["--from-json".to_string(), "--typescript".to_string()],
                expected_dir: "output-samples/json-simple-typescript".to_string(),
            },
            TestCase {
                name: "JSON Simple Zod".to_string(),
                input_file: "input-files/test-data-simple.json".to_string(),
                command_args: vec!["--from-json".to_string(), "--zod".to_string()],
                expected_dir: "output-samples/json-simple-zod".to_string(),
            },
            TestCase {
                name: "JSON Simple Pydantic".to_string(),
                input_file: "input-files/test-data-simple.json".to_string(),
                command_args: vec!["--from-json".to_string(), "--pydantic".to_string()],
                expected_dir: "output-samples/json-simple-pydantic".to_string(),
            },
            TestCase {
                name: "JSON Complex TypeScript".to_string(),
                input_file: "input-files/test-data-complex.json".to_string(),
                command_args: vec!["--from-json".to_string(), "--typescript".to_string()],
                expected_dir: "output-samples/json-complex-typescript".to_string(),
            },
            TestCase {
                name: "JSON Complex Pydantic".to_string(),
                input_file: "input-files/test-data-complex.json".to_string(),
                command_args: vec!["--from-json".to_string(), "--pydantic".to_string()],
                expected_dir: "output-samples/json-complex-pydantic".to_string(),
            },
            // JSON Schema tests
            TestCase {
                name: "JSON Simple JSON Schema".to_string(),
                input_file: "input-files/test-data-simple.json".to_string(),
                command_args: vec!["--from-json".to_string(), "--json-schema".to_string()],
                expected_dir: "output-samples/json-simple-json-schema".to_string(),
            },
            TestCase {
                name: "JSON Complex JSON Schema".to_string(),
                input_file: "input-files/test-data-complex.json".to_string(),
                command_args: vec!["--from-json".to_string(), "--json-schema".to_string()],
                expected_dir: "output-samples/json-complex-json-schema".to_string(),
            },
            TestCase {
                name: "OpenAPI JSON Schema".to_string(),
                input_file: "input-files/simple-sample.json".to_string(),
                command_args: vec!["--from-openapi".to_string(), "--json-schema".to_string()],
                expected_dir: "output-samples/openapi-json-schema".to_string(),
            },
            // From JSON Schema tests
            TestCase {
                name: "From JSON Schema TypeScript".to_string(),
                input_file: "output-samples/json-simple-json-schema/schema.json".to_string(),
                command_args: vec!["--from-json-schema".to_string(), "--typescript".to_string()],
                expected_dir: "output-samples/from-json-schema-typescript".to_string(),
            },
            TestCase {
                name: "From JSON Schema Zod".to_string(),
                input_file: "output-samples/json-complex-json-schema/schema.json".to_string(),
                command_args: vec!["--from-json-schema".to_string(), "--zod".to_string()],
                expected_dir: "output-samples/from-json-schema-zod".to_string(),
            },
            TestCase {
                name: "From JSON Schema Pydantic".to_string(),
                input_file: "output-samples/openapi-json-schema/schema.json".to_string(),
                command_args: vec!["--from-json-schema".to_string(), "--pydantic".to_string()],
                expected_dir: "output-samples/from-json-schema-pydantic".to_string(),
            },
            // Endpoints generator test
            TestCase {
                name: "Endpoints Test".to_string(),
                input_file: "input-files/simple-sample.json".to_string(),
                command_args: vec!["--from-openapi".to_string(), "--endpoints".to_string()],
                expected_dir: "output-samples/endpoints-test".to_string(),
            },
            // Pydantic advanced features test
            TestCase {
                name: "Pydantic Advanced Features".to_string(),
                input_file: "input-files/test-pydantic-advanced.json".to_string(),
                command_args: vec!["--from-json".to_string(), "--pydantic".to_string()],
                expected_dir: "output-samples/pydantic-advanced".to_string(),
            },
            // DotNet advanced types test
            TestCase {
                name: "DotNet Advanced Types".to_string(),
                input_file: "input-files/test-dotnet-advanced.json".to_string(),
                command_args: vec!["--from-json".to_string(), "--dotnet".to_string()],
                expected_dir: "output-samples/dotnet-advanced".to_string(),
            },
            // Deep nesting test
            TestCase {
                name: "Deep Nesting Test".to_string(),
                input_file: "input-files/test-deep-nesting.json".to_string(),
                command_args: vec!["--from-json".to_string(), "--typescript".to_string()],
                expected_dir: "output-samples/deep-nesting".to_string(),
            },
            // Special characters test
            TestCase {
                name: "Special Characters Test".to_string(),
                input_file: "input-files/test-special-chars.json".to_string(),
                command_args: vec!["--from-json".to_string(), "--typescript".to_string()],
                expected_dir: "output-samples/special-chars".to_string(),
            },
            // Empty schema test
            TestCase {
                name: "Empty Schema Test".to_string(),
                input_file: "input-files/test-empty-schema.json".to_string(),
                command_args: vec!["--from-openapi".to_string(), "--typescript".to_string()],
                expected_dir: "output-samples/empty-schema".to_string(),
            },
            // Zod all features test
            TestCase {
                name: "Zod All Features Test".to_string(),
                input_file: "input-files/test-zod-all-features.json".to_string(),
                command_args: vec!["--from-json-schema".to_string(), "--zod".to_string()],
                expected_dir: "output-samples/zod-all-features".to_string(),
            },
        ];

        Self {
            project_root,
            test_cases,
            refresh_mode: false,
            passed_tests: 0,
            failed_tests: 0,
        }
    }

    fn compare_files(&self, file1: &Path, file2: &Path) -> Result<(bool, String)> {
        let content1 = fs::read_to_string(file1).context("Failed to read file1")?;
        let content2 = fs::read_to_string(file2).context("Failed to read file2")?;

        // Normalize by removing trailing whitespace from each line and blank lines at end
        let normalize = |content: &str| {
            let lines: Vec<&str> = content.lines().map(|line| line.trim_end()).collect();
            // Remove trailing empty lines
            let mut lines = lines;
            while lines.last() == Some(&"") {
                lines.pop();
            }
            lines.join("\n")
        };

        let content1_normalized = normalize(&content1);
        let content2_normalized = normalize(&content2);

        if content1_normalized == content2_normalized {
            return Ok((true, String::new()));
        }

        // Generate unified diff format
        let diff = TextDiff::from_lines(&content1_normalized, &content2_normalized);
        let mut diff_output = String::new();

        // Add unified diff header
        diff_output.push_str(&format!(
            "--- expected ({})\n+++ actual ({})\n",
            file1.display(),
            file2.display()
        ));

        // Group changes by context for better readability
        let changes: Vec<_> = diff.iter_all_changes().collect();
        const CONTEXT_LINES: usize = 2;

        let mut in_hunk = false;
        let mut hunk_start = 0;
        let mut hunk_size = 0;
        let mut diff_lines = Vec::new();

        for (i, change) in changes.iter().enumerate() {
            match change.tag() {
                ChangeTag::Delete => {
                    if !in_hunk {
                        in_hunk = true;
                        hunk_start = i.saturating_sub(CONTEXT_LINES);
                    }
                    diff_lines.push((i, format!("-{}", change.value())));
                    hunk_size = i - hunk_start + 1;
                }
                ChangeTag::Insert => {
                    if !in_hunk {
                        in_hunk = true;
                        hunk_start = i.saturating_sub(CONTEXT_LINES);
                    }
                    diff_lines.push((i, format!("+{}", change.value())));
                    hunk_size = i - hunk_start + 1;
                }
                ChangeTag::Equal => {
                    if in_hunk && i - hunk_start <= hunk_size + CONTEXT_LINES {
                        // Still in context
                        diff_lines.push((i, format!(" {}", change.value())));
                    } else if in_hunk {
                        // End of hunk
                        in_hunk = false;
                    }
                }
            }
        }

        // Format output with color codes if terminal supports it
        for (_, line) in diff_lines {
            if line.starts_with('+') {
                diff_output.push_str(&format!("{}\n", line.green()));
            } else if line.starts_with('-') {
                diff_output.push_str(&format!("{}\n", line.red()));
            } else {
                diff_output.push_str(&format!("{line}\n"));
            }
        }

        Ok((false, diff_output))
    }

    fn compare_directories(
        &self,
        expected_dir: &Path,
        actual_dir: &Path,
    ) -> Result<(bool, Vec<String>)> {
        let mut errors = Vec::new();

        let mut expected_files = std::collections::HashSet::new();
        let mut actual_files = std::collections::HashSet::new();

        // Collect expected files
        if expected_dir.exists() {
            for entry in WalkDir::new(expected_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_file())
            {
                if let Ok(rel_path) = entry.path().strip_prefix(expected_dir) {
                    expected_files.insert(rel_path.to_path_buf());
                }
            }
        }

        // Collect actual files
        if actual_dir.exists() {
            for entry in WalkDir::new(actual_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_file())
            {
                if let Ok(rel_path) = entry.path().strip_prefix(actual_dir) {
                    actual_files.insert(rel_path.to_path_buf());
                }
            }
        }

        // Check for missing files (ignore README.md)
        for missing in expected_files.difference(&actual_files) {
            if !missing.ends_with("README.md") {
                errors.push(format!("Missing file: {}", missing.display()));
            }
        }

        // Check for extra files (ignore README.md)
        for extra in actual_files.difference(&expected_files) {
            if !extra.ends_with("README.md") {
                errors.push(format!("Extra file: {}", extra.display()));
            }
        }

        // Compare common files (ignore README.md)
        let common_files = expected_files.intersection(&actual_files);
        for file_rel_path in common_files {
            if file_rel_path.ends_with("README.md") {
                continue;
            }

            let expected_file = expected_dir.join(file_rel_path);
            let actual_file = actual_dir.join(file_rel_path);

            match self.compare_files(&expected_file, &actual_file) {
                Ok((matches, diff)) => {
                    if !matches {
                        errors.push(format!(
                            "{}",
                            format!("File differs: {}", file_rel_path.display()).bold()
                        ));
                        if !diff.is_empty() {
                            errors.push(diff);
                        }
                    }
                }
                Err(e) => {
                    errors.push(format!(
                        "Error comparing {}: {}",
                        file_rel_path.display(),
                        e
                    ));
                }
            }
        }

        Ok((errors.is_empty(), errors))
    }

    fn determine_input_type(command_args: &[String]) -> InputType {
        if command_args.iter().any(|arg| arg == "--from-json") {
            InputType::Json
        } else if command_args.iter().any(|arg| arg == "--from-json-schema") {
            InputType::JsonSchema
        } else if command_args.iter().any(|arg| arg == "--from-openapi") {
            InputType::OpenApi
        } else {
            InputType::OpenApi
        }
    }

    fn determine_generator_type(command_args: &[String]) -> GeneratorType {
        if command_args.iter().any(|arg| arg == "--angular") {
            GeneratorType::Angular
        } else if command_args.iter().any(|arg| arg == "--pydantic") {
            GeneratorType::Pydantic
        } else if command_args.iter().any(|arg| arg == "--python-dict") {
            GeneratorType::PythonDict
        } else if command_args.iter().any(|arg| arg == "--dotnet") {
            GeneratorType::DotNet
        } else if command_args.iter().any(|arg| arg == "--json-schema") {
            GeneratorType::JsonSchema
        } else if command_args.iter().any(|arg| arg == "--endpoints") {
            GeneratorType::Endpoints
        } else if command_args.iter().any(|arg| arg == "--typescript") {
            GeneratorType::TypeScript
        } else if command_args.iter().any(|arg| arg == "--zod") {
            GeneratorType::Zod
        } else {
            GeneratorType::TypeScript
        }
    }

    fn run_single_test(&mut self, test_case: &TestCase) -> Result<bool> {
        if self.refresh_mode {
            eprintln!("{}", format!("\nRefreshing: {}", test_case.name).magenta());
        } else {
            eprintln!("{}", format!("\nRunning: {}", test_case.name).blue());
        }

        // Temporary output directory for this test case
        let temp_dir = tempfile::TempDir::new().context("Failed to create temp directory")?;
        let output_dir = temp_dir.path().join("output");
        fs::create_dir_all(&output_dir).context("Failed to create output directory")?;

        let input_type = Self::determine_input_type(&test_case.command_args);
        let generator_type = Self::determine_generator_type(&test_case.command_args);
        let with_zod = test_case.command_args.iter().any(|arg| arg == "--zod");
        let with_promises = test_case.command_args.iter().any(|arg| arg == "--promises");

        // Parse skip_files from command_args
        let mut skip_files = Vec::new();
        let mut skip_next = false;
        for arg in &test_case.command_args {
            if skip_next {
                skip_files.push(arg.clone());
                skip_next = false;
            } else if arg == "--skip-file" {
                skip_next = true;
            }
        }

        let input_path = self.project_root.join(&test_case.input_file);

        let options = GenerateOptions {
            input_type,
            input_path,
            output_dir: output_dir.clone(),
            generator_type,
            with_zod,
            with_promises,
            hide_version: true,
            root_name: "Root".to_string(),
            debug: false,
            skip_files,
        };

        if let Err(e) = generate(options) {
            eprintln!("{}", format!("ERROR: Generation failed: {}", e).red());
            return Ok(false);
        }

        if self.refresh_mode {
            // Refresh mode: update expected directory
            let expected_path = self.project_root.join(&test_case.expected_dir);

            if expected_path.exists() {
                fs::remove_dir_all(&expected_path)
                    .context("Failed to remove existing expected directory")?;
            }
            fs::create_dir_all(&expected_path).context("Failed to create expected directory")?;

            // Copy generated files to expected directory
            if output_dir.exists() {
                for entry in WalkDir::new(&output_dir)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().is_file())
                {
                    let rel_path = entry
                        .path()
                        .strip_prefix(&output_dir)
                        .context("Failed to get relative path")?;
                    let dest_path = expected_path.join(rel_path);
                    fs::create_dir_all(
                        dest_path
                            .parent()
                            .context("Failed to get parent directory")?,
                    )
                    .context("Failed to create destination directory")?;
                    fs::copy(entry.path(), &dest_path).context("Failed to copy file")?;
                }
            }

            eprintln!(
                "{}",
                format!(
                    "SUCCESS: Updated expected output in {}",
                    test_case.expected_dir
                )
                .green()
            );
            Ok(true)
        } else {
            // Compare with expected directory
            let expected_path = self.project_root.join(&test_case.expected_dir);
            match self.compare_directories(&expected_path, &output_dir) {
                Ok((matches, errors)) => {
                    if matches {
                        eprintln!("{}", "SUCCESS: Output matches expected".green());
                        Ok(true)
                    } else {
                        eprintln!("{}", "ERROR: Output differs from expected".red());
                        eprintln!();
                        for (idx, error) in errors.iter().enumerate() {
                            // Don't limit output, show all diffs for better visibility
                            if error.contains("---") {
                                // This is a unified diff - display it more prominently
                                eprintln!("{}", error);
                            } else if error.contains("File differs") {
                                eprintln!("{}", format!("   {}", error).yellow().bold());
                            } else {
                                eprintln!("{}", format!("   {}", error).yellow());
                            }

                            // Add spacing between different files
                            if idx < errors.len() - 1 && errors[idx + 1].contains("File differs") {
                                eprintln!();
                            }
                        }
                        Ok(false)
                    }
                }
                Err(e) => {
                    eprintln!("{}", format!("ERROR: Comparison failed: {}", e).red());
                    Ok(false)
                }
            }
        }
    }

    fn run_all_tests(&mut self) -> Result<()> {
        if self.refresh_mode {
            eprintln!("{}", "=".repeat(60).cyan().bold());
            eprintln!(
                "{}",
                "Refreshing dtolator Test Expected Outputs".cyan().bold()
            );
            eprintln!("{}", "=".repeat(60).cyan().bold());
        } else {
            eprintln!("{}", "=".repeat(60).cyan().bold());
            eprintln!("{}", "Running dtolator Test Suite".cyan().bold());
            eprintln!("{}", "=".repeat(60).cyan().bold());
        }

        // Run each test case
        for test_case in self.test_cases.clone() {
            match self.run_single_test(&test_case) {
                Ok(true) => {
                    self.passed_tests += 1;
                }
                Ok(false) => {
                    self.failed_tests += 1;
                }
                Err(e) => {
                    eprintln!("{}", format!("ERROR: {}", e).red());
                    self.failed_tests += 1;
                }
            }
        }

        self.print_summary();
        Ok(())
    }

    fn print_summary(&self) {
        eprintln!("{}", "=".repeat(60).cyan().bold());
        if self.refresh_mode {
            eprintln!("{}", "Refresh Results Summary".cyan().bold());
        } else {
            eprintln!("{}", "Test Results Summary".cyan().bold());
        }
        eprintln!("{}", "=".repeat(60).cyan().bold());

        let total_tests = self.passed_tests + self.failed_tests;
        eprintln!("Total tests: {}", total_tests);
        eprintln!("{}", format!("Passed: {}", self.passed_tests).green());
        if self.failed_tests > 0 {
            eprintln!("{}", format!("Failed: {}", self.failed_tests).red());
        } else {
            eprintln!("Failed: {}", self.failed_tests);
        }

        eprintln!();
        if self.failed_tests == 0 {
            eprintln!("{}", "ALL TESTS PASSED!".green().bold());
        } else {
            eprintln!(
                "{}",
                format!("{} TEST(S) FAILED!", self.failed_tests)
                    .red()
                    .bold()
            );
        }
    }
}

#[test]
fn test_suite() {
    let mut suite = TestSuite::new();

    // Check environment variable to refresh expected outputs
    let refresh_mode = std::env::var("DTOLATOR_TEST_REFRESH").is_ok();
    suite.refresh_mode = refresh_mode;

    // Run all tests
    if let Err(e) = suite.run_all_tests() {
        panic!("Test suite error: {}", e);
    }

    assert_eq!(
        suite.failed_tests, 0,
        "{} test(s) failed! See output above for details.",
        suite.failed_tests
    );
}

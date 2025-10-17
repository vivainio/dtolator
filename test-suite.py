#!/usr/bin/env python3
"""
Test Suite for dtolator

This script runs all the variants already in output-samples and ensures that 
the output is exactly the same as expected.

The test suite covers all supported output formats:
- Angular API services (with and without Zod validation)
- TypeScript interfaces and Zod schemas
- Pydantic BaseModel classes
- Python TypedDict definitions  
- C# classes with System.Text.Json serialization
- JSON to TypeScript/Zod/Pydantic conversion

Usage:
    python test-suite.py                # Build and run all tests
    python test-suite.py --no-build     # Run tests without building (faster for development)
    python test-suite.py --refresh      # Regenerate expected output files (updates output-samples)
    python test-suite.py --typecheck    # Enable TypeScript type checking on generated .ts files
    python test-suite.py --help         # Show help information

Test Cases:
- Angular Full Sample: full-sample.json -> Angular services with Zod
- Angular Simple Sample: simple-sample.json -> Angular services with Zod  
- Angular Nested: full-sample.json -> Angular services without Zod
- Comprehensive Nested: full-sample.json -> Angular services with Zod
- Nested Test: full-sample.json -> TypeScript + Zod schemas only
- DotNet Test: simple-sample.json -> C# classes
- Pydantic Test: simple-sample.json -> Python Pydantic models
- Python TypedDict Tests: Various OpenAPI files -> Python TypedDict
- Angular Promises Tests: simple-sample.json -> Angular services with Promise-based methods
- JSON Tests: test-data-simple.json & test-data-complex.json -> TypeScript, Zod, and Pydantic models
- JSON Schema Tests: Various inputs -> JSON Schema generation and conversion
- From JSON Schema Tests: Generated JSON Schema files -> TypeScript, Zod, and Pydantic models

Each test runs the dtolator command with appropriate flags and compares the output
with the expected files in the output-samples directory. Any differences are
reported with detailed diff information.

TypeScript Type Checking:
When --typecheck is enabled, generated TypeScript files are validated using the
TypeScript compiler. A temporary Node.js project is created with the required
dependencies (TypeScript, Zod, Angular) and tsc --noEmit is run to check for
type errors without generating output files.

NOTE: Environment imports have been removed from generated Angular services.
The generated code now requires (window as any).API_URL to be set globally
instead of importing from @env/environment.
"""

import os
import subprocess
import tempfile
import difflib
import shutil
import json
from pathlib import Path
from typing import Dict, List, Tuple, Optional
import sys

# ANSI color codes for better output
class Colors:
    GREEN = '\033[92m'
    RED = '\033[91m'
    YELLOW = '\033[93m'
    BLUE = '\033[94m'
    MAGENTA = '\033[95m'
    CYAN = '\033[96m'
    WHITE = '\033[97m'
    BOLD = '\033[1m'
    END = '\033[0m'

def print_colored(text: str, color: str = Colors.WHITE) -> None:
    """Print colored text to console"""
    print(f"{color}{text}{Colors.END}")

def print_header(text: str) -> None:
    """Print section header"""
    print_colored(f"\n{'='*60}", Colors.CYAN)
    print_colored(f"{text.center(60)}", Colors.CYAN + Colors.BOLD)
    print_colored(f"{'='*60}", Colors.CYAN)

def run_command(cmd: List[str], cwd: Optional[str] = None) -> Tuple[bool, str, str]:
    """
    Run a command and return success status, stdout, and stderr
    """
    try:
        result = subprocess.run(
            cmd, 
            capture_output=True, 
            text=True, 
            cwd=cwd,
            timeout=60  # Increased timeout for npm operations
        )
        return result.returncode == 0, result.stdout, result.stderr
    except subprocess.TimeoutExpired:
        return False, "", "Command timed out after 60 seconds"
    except Exception as e:
        return False, "", str(e)

def compare_files(file1: Path, file2: Path) -> Tuple[bool, str]:
    """
    Compare two files and return whether they match and diff if they don't
    """
    try:
        with open(file1, 'r', encoding='utf-8') as f1, open(file2, 'r', encoding='utf-8') as f2:
            content1 = f1.readlines()
            content2 = f2.readlines()
        
        if content1 == content2:
            return True, ""
        
        diff = ''.join(difflib.unified_diff(
            content1, content2,
            fromfile=str(file1),
            tofile=str(file2),
            lineterm=''
        ))
        return False, diff
    except Exception as e:
        return False, f"Error comparing files: {str(e)}"

def compare_directories(expected_dir: Path, actual_dir: Path) -> Tuple[bool, List[str]]:
    """
    Compare two directories recursively
    """
    errors = []
    
    # Get all files in both directories
    expected_files = set()
    actual_files = set()
    
    if expected_dir.exists():
        for file_path in expected_dir.rglob('*'):
            if file_path.is_file():
                expected_files.add(file_path.relative_to(expected_dir))
    
    if actual_dir.exists():
        for file_path in actual_dir.rglob('*'):
            if file_path.is_file():
                actual_files.add(file_path.relative_to(actual_dir))
    
    # Check for missing files
    missing_files = expected_files - actual_files
    extra_files = actual_files - expected_files
    
    for missing in missing_files:
        errors.append(f"Missing file: {missing}")
    
    for extra in extra_files:
        errors.append(f"Extra file: {extra}")
    
    # Compare common files
    common_files = expected_files & actual_files
    for file_rel_path in common_files:
        expected_file = expected_dir / file_rel_path
        actual_file = actual_dir / file_rel_path
        
        matches, diff = compare_files(expected_file, actual_file)
        if not matches:
            errors.append(f"File differs: {file_rel_path}")
            if diff:
                errors.append(f"Diff for {file_rel_path}:\n{diff}")
    
    return len(errors) == 0, errors

class TestCase:
    """Represents a single test case"""
    def __init__(self, name: str, input_file: str, command_args: List[str], 
                 expected_dir: Optional[str] = None, expected_files: Optional[List[str]] = None):
        self.name = name
        self.input_file = input_file
        self.command_args = command_args
        self.expected_dir = expected_dir
        self.expected_files = expected_files or []

class TestSuite:
    """Main test suite runner"""
    
    def __init__(self):
        self.project_root = Path.cwd()
        # Try MSVC target first (for Windows), then default
        self.dtolator_binary = self.project_root / "target" / "x86_64-pc-windows-msvc" / "release" / "dtolator.exe"
        if not self.dtolator_binary.exists():
            self.dtolator_binary = self.project_root / "target" / "release" / "dtolator.exe"
        if not self.dtolator_binary.exists():
            # Try without .exe for non-Windows systems
            self.dtolator_binary = self.project_root / "target" / "release" / "dtolator"
        
        self.output_samples_dir = self.project_root / "output-samples"
        self.passed_tests = 0
        self.failed_tests = 0
        self.test_details = []
        self.refresh_mode = False
        self.enable_typescript_check = False  # New: TypeScript checking flag
        self.shared_ts_env = None  # Shared TypeScript environment directory
        
        # TypeScript test cases that should be type-checked
        self.typescript_test_cases = {
            "Angular Full Sample",
            "Angular Simple Sample", 
            "Angular Nested Test",
            "Comprehensive Nested Test",
            "Nested Test",
            "Angular Promises with Zod",
            "Angular Promises without Zod",
            "JSON Complex TypeScript",
            "JSON Simple TypeScript",
            "JSON Simple Zod",
            "From JSON Schema TypeScript",
            "From JSON Schema Zod",
        }
        
        # Define all test cases based on the output-samples directory structure
        self.test_cases = [
            # Angular tests with full-sample.json
            TestCase(
                name="Angular Full Sample",
                input_file="full-sample.json",
                command_args=["--angular", "--zod"],
                expected_dir="output-samples/angular/full-sample"
            ),
            TestCase(
                name="Angular Simple Sample", 
                input_file="simple-sample.json",
                command_args=["--angular", "--zod"],
                expected_dir="output-samples/angular/simple-sample"
            ),
            
            # Angular nested tests (appears to use full-sample.json based on file complexity)
            TestCase(
                name="Angular Nested Test",
                input_file="full-sample.json", 
                command_args=["--angular"],
                expected_dir="output-samples/angular-nested"
            ),
            
            # Comprehensive nested test (uses full-sample.json with zod)
            TestCase(
                name="Comprehensive Nested Test",
                input_file="full-sample.json",
                command_args=["--angular", "--zod"],
                expected_dir="output-samples/comprehensive-nested-test"
            ),
            
            # Nested test (TypeScript + Zod only)
            TestCase(
                name="Nested Test",
                input_file="full-sample.json",
                command_args=["--typescript", "--zod"],
                expected_dir="output-samples/nested-test"
            ),
            
            # .NET test
            TestCase(
                name="DotNet Test",
                input_file="simple-sample.json",
                command_args=["--dotnet"],
                expected_dir="output-samples/dotnet"
            ),
            
            # Pydantic test
            TestCase(
                name="Pydantic Test",
                input_file="simple-sample.json", 
                command_args=["--pydantic"],
                expected_dir="output-samples/pydantic"
            ),
            
            # Python TypedDict tests
            TestCase(
                name="Python TypedDict Test",
                input_file="simple-sample.json",
                command_args=["--python-dict"],
                expected_dir="output-samples/python-typed-dict"
            ),
            TestCase(
                name="Python TypedDict Full Test",
                input_file="full-sample.json",
                command_args=["--python-dict"],
                expected_dir="output-samples/python-typed-dict-full"
            ),
            
            # Promises tests - Testing new --promises flag
            TestCase(
                name="Angular Promises with Zod",
                input_file="simple-sample.json",
                command_args=["--angular", "--zod", "--promises"],
                expected_dir="output-samples/angular-promises-with-zod"
            ),
            TestCase(
                name="Angular Promises without Zod",
                input_file="simple-sample.json",
                command_args=["--angular", "--promises"],
                expected_dir="output-samples/angular-promises-no-zod"
            ),
            
            # JSON to TypeScript/Zod/Pydantic tests using actual JSON files
            TestCase(
                name="JSON Simple TypeScript",
                input_file="test-data-simple.json",
                command_args=["--from-json", "--typescript"],
                expected_dir="output-samples/json-simple-typescript"
            ),
            TestCase(
                name="JSON Simple Zod",
                input_file="test-data-simple.json",
                command_args=["--from-json", "--zod"],
                expected_dir="output-samples/json-simple-zod"
            ),
            TestCase(
                name="JSON Simple Pydantic",
                input_file="test-data-simple.json",
                command_args=["--from-json", "--pydantic"],
                expected_dir="output-samples/json-simple-pydantic"
            ),
            TestCase(
                name="JSON Complex TypeScript",
                input_file="test-data-complex.json",
                command_args=["--from-json", "--typescript"],
                expected_dir="output-samples/json-complex-typescript"
            ),
            TestCase(
                name="JSON Complex Pydantic",
                input_file="test-data-complex.json",
                command_args=["--from-json", "--pydantic"],
                expected_dir="output-samples/json-complex-pydantic"
            ),
            
            # JSON Schema tests
            TestCase(
                name="JSON Simple JSON Schema",
                input_file="test-data-simple.json",
                command_args=["--from-json", "--json-schema"],
                expected_dir="output-samples/json-simple-json-schema"
            ),
            TestCase(
                name="JSON Complex JSON Schema",
                input_file="test-data-complex.json", 
                command_args=["--from-json", "--json-schema"],
                expected_dir="output-samples/json-complex-json-schema"
            ),
            TestCase(
                name="OpenAPI JSON Schema",
                input_file="simple-sample.json",
                command_args=["--from-openapi", "--json-schema"],
                expected_dir="output-samples/openapi-json-schema"
            ),
            
            # From JSON Schema tests - using existing generated JSON Schema files as input
            TestCase(
                name="From JSON Schema TypeScript",
                input_file="output-samples/json-simple-json-schema/schema.json",
                command_args=["--from-json-schema", "--typescript"],
                expected_dir="output-samples/from-json-schema-typescript"
            ),
            TestCase(
                name="From JSON Schema Zod",
                input_file="output-samples/json-complex-json-schema/schema.json",
                command_args=["--from-json-schema", "--zod"],
                expected_dir="output-samples/from-json-schema-zod"
            ),
            TestCase(
                name="From JSON Schema Pydantic",
                input_file="output-samples/openapi-json-schema/schema.json",
                command_args=["--from-json-schema", "--pydantic"],
                expected_dir="output-samples/from-json-schema-pydantic"
            ),
        ]
    
    def check_typescript_environment(self) -> bool:
        """
        Check if the static TypeScript environment is set up correctly
        """
        # Check if required files exist
        package_json = self.project_root / "package.json"
        node_modules = self.project_root / "node_modules"
        tsconfig = self.project_root / "tsconfig.json"
        
        if not package_json.exists():
            print_colored("   ERROR: package.json not found", Colors.RED)
            return False
            
        if not node_modules.exists():
            print_colored("   ERROR: node_modules not found. Run 'npm install' first.", Colors.RED)
            return False
            
        if not tsconfig.exists():
            print_colored("   ERROR: tsconfig.json not found", Colors.RED)
            return False
        
        # Check if TypeScript is available
        tsc_binary = node_modules / ".bin" / "tsc.cmd" if os.name == 'nt' else node_modules / ".bin" / "tsc"
        if not tsc_binary.exists():
            print_colored("   ERROR: TypeScript compiler not found in node_modules", Colors.RED)
            return False
            
        return True
            
    def setup_shared_typescript_environment(self) -> bool:
        """
        Use the static TypeScript environment (no setup needed)
        """
        if not self.enable_typescript_check:
            return True
            
        print_colored(f"Checking static TypeScript environment...", Colors.BLUE)
        
        # Check if the static TypeScript environment is ready
        if not self.check_typescript_environment():
            print_colored(f"   ERROR: Static TypeScript environment not ready.", Colors.RED)
            print_colored(f"   INFO: Run 'npm install' to set up dependencies.", Colors.YELLOW)
            return False
            
        # Use the project root as the shared environment
        self.shared_ts_env = self.project_root
        
        print_colored(f"   SUCCESS: Static TypeScript environment ready", Colors.GREEN)
        return True
            
    
    def format_typescript_errors(self, tsc_output: str) -> str:
        """
        Format TypeScript compiler errors for better readability
        """
        lines = tsc_output.strip().split('\n')
        formatted_lines = []
        
        for line in lines:
            line = line.strip()
            if line:
                # Color error lines
                if ') error TS' in line:
                    formatted_lines.append(f"      ERROR: {line}")
                elif line.startswith('Found ') and ' error' in line:
                    formatted_lines.append(f"      SUMMARY: {line}")
                else:
                    formatted_lines.append(f"      {line}")
        
        return '\n'.join(formatted_lines)
    
    def execute_typescript_check(self, project_dir: Path) -> Tuple[bool, str]:
        """
        Execute TypeScript type checking using the local tsc binary
        """
        try:
            # Use the local tsc binary from node_modules
            tsc_binary = project_dir / "node_modules" / ".bin" / ("tsc.cmd" if os.name == 'nt' else "tsc")
            
            print_colored(f"   Running TypeScript type check...", Colors.BLUE)
            success, stdout, stderr = run_command([str(tsc_binary), "--noEmit"], cwd=str(project_dir))
            
            if success:
                return True, "No type errors found"
            else:
                # Combine stdout and stderr for complete error information
                full_output = f"{stdout}\n{stderr}".strip()
                return False, full_output
                
        except Exception as e:
            return False, f"Error running TypeScript check: {str(e)}"
    
    def run_typescript_typecheck(self, test_case: TestCase, output_dir: Path) -> bool:
        """
        Run TypeScript type checking on generated .ts files using the shared environment
        """
        # Only run TypeScript checking for relevant test cases
        if test_case.name not in self.typescript_test_cases:
            return True
            
        if not self.shared_ts_env:
            print_colored(f"   WARNING: Shared TypeScript environment not available", Colors.YELLOW)
            return True  # Don't fail the test, just skip TypeScript checking
        
        # Find TypeScript files in the output directory
        ts_files = list(output_dir.glob('**/*.ts'))
        if not ts_files:
            return True  # No TypeScript files to check
        
        print_colored(f"   Found {len(ts_files)} TypeScript files to check", Colors.BLUE)
        
        # Copy TypeScript files to shared environment (preserving directory structure)
        try:
            for ts_file in ts_files:
                rel_path = ts_file.relative_to(output_dir)
                dest_path = self.shared_ts_env / rel_path
                dest_path.parent.mkdir(parents=True, exist_ok=True)
                if dest_path.exists():
                    dest_path.unlink()  # Remove existing file
                shutil.copy2(ts_file, dest_path)
            
            # Run TypeScript type checking
            success, output = self.execute_typescript_check(self.shared_ts_env)
            
            # Clean up copied files for next test
            for ts_file in ts_files:
                rel_path = ts_file.relative_to(output_dir)
                dest_path = self.shared_ts_env / rel_path
                if dest_path.exists():
                    dest_path.unlink()
                # Also clean up empty directories
                try:
                    if dest_path.parent != self.shared_ts_env:
                        dest_path.parent.rmdir()
                except OSError:
                    pass  # Directory not empty, that's fine
            
            if success:
                print_colored(f"   SUCCESS: TypeScript type check passed", Colors.GREEN)
                return True
            else:
                print_colored(f"   ERROR: TypeScript type check failed:", Colors.RED)
                formatted_errors = self.format_typescript_errors(output)
                print_colored(formatted_errors, Colors.RED)
                self.test_details.append(f"{test_case.name}: TypeScript type check failed")
                return False
                
        except Exception as e:
            print_colored(f"   WARNING: TypeScript check failed due to error: {e}", Colors.YELLOW)
            return True  # Don't fail the test, just skip TypeScript checking
    
    def build_project(self) -> bool:
        """Build the dtolator project"""
        print_header("Building dtolator")
        
        print_colored("Building project with cargo...", Colors.BLUE)
        # Try MSVC target first on Windows, fallback to default
        success, stdout, stderr = run_command(["cargo", "build", "--release", "--target", "x86_64-pc-windows-msvc"])
        
        if not success and "target" in stderr.lower():
            print_colored("MSVC target not available, trying default target...", Colors.YELLOW)
            success, stdout, stderr = run_command(["cargo", "build", "--release"])
        
        if success:
            print_colored("SUCCESS: Build successful", Colors.GREEN)
            return True
        else:
            print_colored("ERROR: Build failed", Colors.RED)
            print_colored(f"stdout: {stdout}", Colors.WHITE)
            print_colored(f"stderr: {stderr}", Colors.RED)
            return False
    
    def run_single_test(self, test_case: TestCase) -> bool:
        """Run a single test case"""
        if self.refresh_mode:
            print_colored(f"\nRefreshing: {test_case.name}", Colors.MAGENTA)
        else:
            print_colored(f"\nRunning: {test_case.name}", Colors.BLUE)
        

        
        # Prepare command based on input type
        if "--from-json" in test_case.command_args:
            # JSON input test case
            cmd = [str(self.dtolator_binary), "--from-json", test_case.input_file] + [arg for arg in test_case.command_args if arg != "--from-json"]
        elif "--from-json-schema" in test_case.command_args:
            # JSON Schema input test case
            cmd = [str(self.dtolator_binary), "--from-json-schema", test_case.input_file] + [arg for arg in test_case.command_args if arg != "--from-json-schema"]
        elif "--from-openapi" in test_case.command_args:
            # OpenAPI test case that already specifies --from-openapi in command_args
            cmd = [str(self.dtolator_binary), "--from-openapi", test_case.input_file] + [arg for arg in test_case.command_args if arg != "--from-openapi"]
        else:
            # Default: assume OpenAPI input for backward compatibility
            cmd = [str(self.dtolator_binary), "--from-openapi", test_case.input_file] + test_case.command_args

        with tempfile.TemporaryDirectory() as temp_dir:
            temp_path = Path(temp_dir)
            
            # Determine if we should use directory output based on command args
            # For single file generators like --dotnet, --pydantic, --python-dict, --json-schema, we output to directory
            # if expected_dir is specified, even though the original command outputs to stdout
            should_use_dir_output = (test_case.expected_dir is not None and 
                                   any(arg in ["--dotnet", "--pydantic", "--python-dict", "--json-schema"] for arg in test_case.command_args))
            
            if test_case.expected_dir and not should_use_dir_output:
                # Test with directory output
                output_dir = temp_path / "output"
                cmd.extend(["-o", str(output_dir)])
                
                success, stdout, stderr = run_command(cmd)
                
                if not success:
                    print_colored(f"ERROR: Command failed: {' '.join(cmd)}", Colors.RED)
                    print_colored(f"stderr: {stderr}", Colors.RED)
                    self.test_details.append(f"{test_case.name}: Command failed - {stderr}")
                    return False
                
                if self.refresh_mode:
                    # Refresh mode: update expected directory
                    expected_path = self.project_root / test_case.expected_dir
                    
                    # Remove existing directory and recreate
                    if expected_path.exists():
                        shutil.rmtree(expected_path)
                    expected_path.mkdir(parents=True, exist_ok=True)
                    
                    # Copy generated files to expected directory
                    if output_dir.exists():
                        for file_path in output_dir.rglob('*'):
                            if file_path.is_file():
                                rel_path = file_path.relative_to(output_dir)
                                dest_path = expected_path / rel_path
                                dest_path.parent.mkdir(parents=True, exist_ok=True)
                                shutil.copy2(file_path, dest_path)
                    
                    print_colored(f"SUCCESS: Updated expected output in {test_case.expected_dir}", Colors.GREEN)
                    return True
                else:
                    # Compare with expected directory
                    expected_path = self.project_root / test_case.expected_dir
                    matches, errors = compare_directories(expected_path, output_dir)
                    
                    if matches:
                        print_colored("SUCCESS: Output matches expected", Colors.GREEN)
                        
                        # Run TypeScript type checking if enabled
                        if self.enable_typescript_check and not self.refresh_mode:
                            if not self.run_typescript_typecheck(test_case, output_dir):
                                return False
                        
                        return True
                    else:
                        print_colored("ERROR: Output differs from expected", Colors.RED)
                        for error in errors[:10]:  # Limit error output
                            print_colored(f"   {error}", Colors.YELLOW)
                        if len(errors) > 10:
                            print_colored(f"   ... and {len(errors) - 10} more errors", Colors.YELLOW)
                        self.test_details.append(f"{test_case.name}: Output differs - {len(errors)} errors")
                        return False
            
            elif should_use_dir_output:
                # Single file output to directory (for --dotnet, --pydantic, --python-dict)
                success, stdout, stderr = run_command(cmd)
                
                if not success:
                    print_colored(f"ERROR: Command failed: {' '.join(cmd)}", Colors.RED)
                    print_colored(f"stderr: {stderr}", Colors.RED)
                    self.test_details.append(f"{test_case.name}: Command failed - {stderr}")
                    return False
                
                if self.refresh_mode:
                    # Refresh mode: update expected directory with single file
                    if test_case.expected_dir is None:
                        print_colored("ERROR: No expected directory specified for refresh", Colors.RED)
                        return False
                    expected_path = self.project_root / test_case.expected_dir
                    
                    # Ensure directory exists
                    expected_path.mkdir(parents=True, exist_ok=True)
                    
                    # Determine the output filename based on the command
                    if "--dotnet" in test_case.command_args:
                        output_filename = "Models.cs"
                    elif "--pydantic" in test_case.command_args:
                        output_filename = "models.py"
                    elif "--python-dict" in test_case.command_args:
                        output_filename = "typed_dicts.py"
                    elif "--json-schema" in test_case.command_args:
                        output_filename = "schema.json"
                    else:
                        output_filename = "output.txt"
                    
                    output_file = expected_path / output_filename
                    with open(output_file, 'w', encoding='utf-8') as f:
                        f.write(stdout)
                    
                    print_colored(f"SUCCESS: Updated expected file: {output_file.relative_to(self.project_root)}", Colors.GREEN)
                    return True
                else:
                    # Compare with expected directory (which should contain a single file)
                    if test_case.expected_dir is None:
                        print_colored("ERROR: No expected directory specified for comparison", Colors.RED)
                        return False
                    expected_path = self.project_root / test_case.expected_dir
                    
                    # Determine the expected filename
                    if "--dotnet" in test_case.command_args:
                        expected_filename = "Models.cs"
                    elif "--pydantic" in test_case.command_args:
                        expected_filename = "models.py"
                    elif "--python-dict" in test_case.command_args:
                        expected_filename = "typed_dicts.py"
                    elif "--json-schema" in test_case.command_args:
                        expected_filename = "schema.json"
                    else:
                        print_colored(f"ERROR: Unknown command type for directory comparison", Colors.RED)
                        return False
                    
                    expected_file = expected_path / expected_filename
                    if expected_file.exists():
                        with open(expected_file, 'r', encoding='utf-8') as f:
                            expected_content = f.read()
                        
                        if stdout.strip() == expected_content.strip():
                            print_colored("SUCCESS: Output matches expected", Colors.GREEN)
                            
                            # Note: TypeScript checking not applicable for single file outputs to directory
                            # because we don't have the output_dir structure needed
                            
                            return True
                        else:
                            print_colored("ERROR: Output differs from expected", Colors.RED)
                            # Show a brief diff
                            diff = list(difflib.unified_diff(
                                expected_content.splitlines(keepends=True),
                                stdout.splitlines(keepends=True),
                                fromfile=str(expected_file),
                                tofile="actual",
                                n=3
                            ))
                            for line in diff[:20]:  # Show first 20 lines of diff
                                print_colored(f"   {line.rstrip()}", Colors.YELLOW)
                            if len(diff) > 20:
                                print_colored(f"   ... ({len(diff) - 20} more diff lines)", Colors.YELLOW)
                            self.test_details.append(f"{test_case.name}: Output content differs")
                            return False
                    else:
                        print_colored(f"ERROR: Expected file not found: {expected_file}", Colors.RED)
                        self.test_details.append(f"{test_case.name}: Expected file not found")
                        return False
                    
            else:
                # Test with stdout output and compare individual files
                success, stdout, stderr = run_command(cmd)
                
                if not success:
                    print_colored(f"ERROR: Command failed: {' '.join(cmd)}", Colors.RED)
                    print_colored(f"stderr: {stderr}", Colors.RED)
                    self.test_details.append(f"{test_case.name}: Command failed - {stderr}")
                    return False
                
                if self.refresh_mode:
                    # Refresh mode: update expected file
                    if test_case.expected_files:
                        expected_file_path = None
                        
                        # Find the expected file in output-samples
                        for root, dirs, files in os.walk(self.output_samples_dir):
                            for file in files:
                                if file in test_case.expected_files:
                                    expected_file_path = Path(root) / file
                                    break
                            if expected_file_path:
                                break
                        
                        if expected_file_path:
                            # Ensure directory exists
                            expected_file_path.parent.mkdir(parents=True, exist_ok=True)
                            
                            # Write new content
                            with open(expected_file_path, 'w', encoding='utf-8') as f:
                                f.write(stdout)
                            
                            print_colored(f"SUCCESS: Updated expected file: {expected_file_path.relative_to(self.project_root)}", Colors.GREEN)
                            return True
                        else:
                            print_colored(f"ERROR: Could not determine expected file path for: {test_case.expected_files}", Colors.RED)
                            return False
                    else:
                        print_colored("SUCCESS: Command executed successfully (no file to update)", Colors.GREEN)
                        return True
                else:
                    # For single file outputs, compare with expected file
                    if test_case.expected_files:
                        expected_file_path = None
                        
                        # Find the expected file in output-samples
                        for root, dirs, files in os.walk(self.output_samples_dir):
                            for file in files:
                                if file in test_case.expected_files:
                                    expected_file_path = Path(root) / file
                                    break
                            if expected_file_path:
                                break
                        
                        if expected_file_path and expected_file_path.exists():
                            with open(expected_file_path, 'r', encoding='utf-8') as f:
                                expected_content = f.read()
                            
                            if stdout.strip() == expected_content.strip():
                                print_colored("SUCCESS: Output matches expected", Colors.GREEN)
                                # TypeScript checking not applicable for stdout-based tests
                                return True
                            else:
                                print_colored("ERROR: Output differs from expected", Colors.RED)
                                # Show a brief diff
                                diff = list(difflib.unified_diff(
                                    expected_content.splitlines(keepends=True),
                                    stdout.splitlines(keepends=True),
                                    fromfile="expected",
                                    tofile="actual",
                                    n=3
                                ))
                                for line in diff[:20]:  # Show first 20 lines of diff
                                    print_colored(f"   {line.rstrip()}", Colors.YELLOW)
                                if len(diff) > 20:
                                    print_colored(f"   ... ({len(diff) - 20} more diff lines)", Colors.YELLOW)
                                self.test_details.append(f"{test_case.name}: Output content differs")
                                return False
                        else:
                            print_colored(f"ERROR: Expected file not found: {test_case.expected_files}", Colors.RED)
                            self.test_details.append(f"{test_case.name}: Expected file not found")
                            return False
                    else:
                        print_colored("SUCCESS: Command executed successfully", Colors.GREEN)
                        # TypeScript checking not applicable for stdout-based tests
                        return True
    
    def run_all_tests(self) -> bool:
        """Run all test cases"""
        if self.refresh_mode:
            print_header("Refreshing dtolator Test Expected Outputs")
        else:
            print_header("Running dtolator Test Suite")
        
        # First, build the project
        if not self.build_project():
            return False
        
        # Check if dtolator binary exists
        if not self.dtolator_binary.exists():
            print_colored(f"ERROR: dtolator binary not found at: {self.dtolator_binary}", Colors.RED)
            return False
        
        print_colored(f"Using dtolator binary: {self.dtolator_binary}", Colors.BLUE)
        
        # Run each test case
        for test_case in self.test_cases:
            if self.run_single_test(test_case):
                self.passed_tests += 1
            else:
                self.failed_tests += 1
        
        # Print summary
        self.print_summary()
        
        return self.failed_tests == 0
    
    def print_summary(self) -> None:
        """Print test results summary"""
        if self.refresh_mode:
            print_header("Refresh Results Summary")
            
            total_tests = self.passed_tests + self.failed_tests
            print_colored(f"Total test cases processed: {total_tests}", Colors.WHITE)
            print_colored(f"Successfully updated: {self.passed_tests}", Colors.GREEN)
            print_colored(f"Failed to update: {self.failed_tests}", Colors.RED if self.failed_tests > 0 else Colors.WHITE)
            
            if self.failed_tests > 0:
                print_colored(f"\nUPDATE FAILURES:", Colors.RED + Colors.BOLD)
                for detail in self.test_details:
                    print_colored(f"   • {detail}", Colors.RED)
            
            if self.failed_tests == 0:
                print_colored(f"\nALL EXPECTED OUTPUTS UPDATED!", Colors.GREEN + Colors.BOLD)
                print_colored(f"The output-samples directory has been refreshed with current dtolator output.", Colors.GREEN)
            else:
                print_colored(f"\n{self.failed_tests} UPDATE(S) FAILED!", Colors.RED + Colors.BOLD)
        else:
            print_header("Test Results Summary")
            
            total_tests = self.passed_tests + self.failed_tests
            print_colored(f"Total tests: {total_tests}", Colors.WHITE)
            print_colored(f"Passed: {self.passed_tests}", Colors.GREEN)
            print_colored(f"Failed: {self.failed_tests}", Colors.RED if self.failed_tests > 0 else Colors.WHITE)
            
            if self.failed_tests > 0:
                print_colored(f"\nFAILURE DETAILS:", Colors.RED + Colors.BOLD)
                for detail in self.test_details:
                    if "failed" in detail.lower() or "differs" in detail.lower() or "not found" in detail.lower():
                        print_colored(f"   • {detail}", Colors.RED)
            
            if self.failed_tests == 0:
                print_colored(f"\nALL TESTS PASSED!", Colors.GREEN + Colors.BOLD)
            else:
                print_colored(f"\n{self.failed_tests} TEST(S) FAILED!", Colors.RED + Colors.BOLD)

def main():
    """Main entry point"""
    test_suite = TestSuite()
    
    # Parse command line arguments
    args = sys.argv[1:] if len(sys.argv) > 1 else []
    
    # Handle help first
    if '-h' in args or '--help' in args:
        print("dtolator Test Suite")
        print("Usage: python test-suite.py [options]")
        print("Options:")
        print("  -h, --help    Show this help message")
        print("  --no-build    Skip building the project (use existing binary)")
        print("  --refresh     Regenerate expected output files (updates output-samples)")
        print("  --typecheck   Enable TypeScript type checking on generated .ts files")
        print("")
        print("Examples:")
        print("  python test-suite.py                    # Run all tests with build")
        print("  python test-suite.py --typecheck        # Run tests with TypeScript checking")
        print("  python test-suite.py --no-build --typecheck  # Skip build, run with TypeScript checking")
        print("  python test-suite.py --refresh          # Update expected output files")
        return
    
    # Parse flags
    no_build = '--no-build' in args
    refresh_mode = '--refresh' in args
    typecheck_enabled = '--typecheck' in args
    
    # Set flags on test suite
    test_suite.refresh_mode = refresh_mode
    test_suite.enable_typescript_check = typecheck_enabled
    
    # Check TypeScript checking prerequisites
    if typecheck_enabled:
        if not test_suite.check_typescript_environment():
            print_colored("ERROR: Static TypeScript environment not ready", Colors.RED)
            print_colored("INFO: Run 'npm install' in the project root to set up dependencies", Colors.YELLOW)
            print_colored("Disabling TypeScript checking for this run", Colors.YELLOW)
            test_suite.enable_typescript_check = False
            typecheck_enabled = False
    
    # Build project unless --no-build is specified
    if not no_build:
        if not test_suite.build_project():
            sys.exit(1)
    
    # Check if dtolator binary exists
    if not test_suite.dtolator_binary.exists():
        print_colored(f"ERROR: dtolator binary not found at: {test_suite.dtolator_binary}", Colors.RED)
        if no_build:
            print_colored("Please build the project first with: cargo build --release", Colors.YELLOW)
        sys.exit(1)
    
    # Print mode information
    if refresh_mode:
        print_header("Refreshing dtolator Test Expected Outputs")
        print_colored("WARNING: This will overwrite existing files in output-samples/", Colors.YELLOW + Colors.BOLD)
    elif typecheck_enabled:
        print_header("Running dtolator Test Suite with TypeScript Checking")
        print_colored("TypeScript type checking enabled for generated .ts files", Colors.YELLOW + Colors.BOLD)
    elif no_build:
        print_header("Running dtolator Test Suite (skipping build)")
    else:
        print_header("Running dtolator Test Suite")
    
    print_colored(f"Using dtolator binary: {test_suite.dtolator_binary}", Colors.BLUE)
    
    # Setup shared TypeScript environment if needed
    if typecheck_enabled:
        if not test_suite.setup_shared_typescript_environment():
            print_colored("ERROR: Failed to setup shared TypeScript environment", Colors.RED)
            print_colored("Disabling TypeScript checking for this run", Colors.YELLOW)
            test_suite.enable_typescript_check = False
    
    try:
        # Run all test cases
        for test_case in test_suite.test_cases:
            if test_suite.run_single_test(test_case):
                test_suite.passed_tests += 1
            else:
                test_suite.failed_tests += 1
    finally:
        # No cleanup needed for static TypeScript environment
        pass
    
    # Print summary and exit
    test_suite.print_summary()
    sys.exit(0 if test_suite.failed_tests == 0 else 1)

if __name__ == "__main__":
    main()
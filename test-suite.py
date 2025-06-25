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

Usage:
    python test-suite.py                # Build and run all tests
    python test-suite.py --no-build     # Run tests without building (faster for development)
    python test-suite.py --refresh      # Regenerate expected output files (updates output-samples)
    python test-suite.py --help         # Show help information

Test Cases:
- Angular Full Sample: full-sample.json -> Angular services with Zod
- Angular Simple Sample: simple-sample.json -> Angular services with Zod  
- Angular Nested: full-sample.json -> Angular services without Zod
- Comprehensive Nested: full-sample.json -> Angular services with Zod
- Nested Test: full-sample.json -> TypeScript + Zod schemas only
- DotNet Test: simple-sample.json -> C# classes
- Pydantic Test: simple-sample.json -> Python Pydantic models
- Python TypedDict Test: simple-sample.json -> Python TypedDict
- Python TypedDict Full: full-sample.json -> Python TypedDict

Each test runs the dtolator command with appropriate flags and compares the output
with the expected files in the output-samples directory. Any differences are
reported with detailed diff information.
"""

import os
import subprocess
import tempfile
import difflib
import shutil
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
            timeout=30
        )
        return result.returncode == 0, result.stdout, result.stderr
    except subprocess.TimeoutExpired:
        return False, "", "Command timed out after 30 seconds"
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
                expected_files=["Models.cs"]
            ),
            
            # Pydantic test
            TestCase(
                name="Pydantic Test",
                input_file="simple-sample.json", 
                command_args=["--pydantic"],
                expected_files=["models.py"]
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
        ]
    
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
            print_colored("✅ Build successful", Colors.GREEN)
            return True
        else:
            print_colored("❌ Build failed", Colors.RED)
            print_colored(f"stdout: {stdout}", Colors.WHITE)
            print_colored(f"stderr: {stderr}", Colors.RED)
            return False
    
    def run_single_test(self, test_case: TestCase) -> bool:
        """Run a single test case"""
        if self.refresh_mode:
            print_colored(f"\n🔄 Refreshing: {test_case.name}", Colors.MAGENTA)
        else:
            print_colored(f"\n🔍 Running: {test_case.name}", Colors.BLUE)
        
        # Prepare command
        cmd = [str(self.dtolator_binary), "-i", test_case.input_file] + test_case.command_args
        
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_path = Path(temp_dir)
            
            # Determine if we should use directory output based on command args
            # For single file generators like --dotnet, --pydantic, --python-dict, we output to directory
            # if expected_dir is specified, even though the original command outputs to stdout
            should_use_dir_output = (test_case.expected_dir is not None and 
                                   any(arg in ["--dotnet", "--pydantic", "--python-dict"] for arg in test_case.command_args))
            
            if test_case.expected_dir and not should_use_dir_output:
                # Test with directory output
                output_dir = temp_path / "output"
                cmd.extend(["-o", str(output_dir)])
                
                success, stdout, stderr = run_command(cmd)
                
                if not success:
                    print_colored(f"❌ Command failed: {' '.join(cmd)}", Colors.RED)
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
                    
                    print_colored(f"✅ Updated expected output in {test_case.expected_dir}", Colors.GREEN)
                    return True
                else:
                    # Compare with expected directory
                    expected_path = self.project_root / test_case.expected_dir
                    matches, errors = compare_directories(expected_path, output_dir)
                    
                    if matches:
                        print_colored("✅ Output matches expected", Colors.GREEN)
                        return True
                    else:
                        print_colored("❌ Output differs from expected", Colors.RED)
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
                    print_colored(f"❌ Command failed: {' '.join(cmd)}", Colors.RED)
                    print_colored(f"stderr: {stderr}", Colors.RED)
                    self.test_details.append(f"{test_case.name}: Command failed - {stderr}")
                    return False
                
                if self.refresh_mode:
                    # Refresh mode: update expected directory with single file
                    if test_case.expected_dir is None:
                        print_colored("❌ No expected directory specified for refresh", Colors.RED)
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
                    else:
                        output_filename = "output.txt"
                    
                    output_file = expected_path / output_filename
                    with open(output_file, 'w', encoding='utf-8') as f:
                        f.write(stdout)
                    
                    print_colored(f"✅ Updated expected file: {output_file.relative_to(self.project_root)}", Colors.GREEN)
                    return True
                else:
                    # Compare with expected directory (which should contain a single file)
                    if test_case.expected_dir is None:
                        print_colored("❌ No expected directory specified for comparison", Colors.RED)
                        return False
                    expected_path = self.project_root / test_case.expected_dir
                    
                    # Determine the expected filename
                    if "--dotnet" in test_case.command_args:
                        expected_filename = "Models.cs"
                    elif "--pydantic" in test_case.command_args:
                        expected_filename = "models.py"
                    elif "--python-dict" in test_case.command_args:
                        expected_filename = "typed_dicts.py"
                    else:
                        print_colored(f"❌ Unknown command type for directory comparison", Colors.RED)
                        return False
                    
                    expected_file = expected_path / expected_filename
                    if expected_file.exists():
                        with open(expected_file, 'r', encoding='utf-8') as f:
                            expected_content = f.read()
                        
                        if stdout.strip() == expected_content.strip():
                            print_colored("✅ Output matches expected", Colors.GREEN)
                            return True
                        else:
                            print_colored("❌ Output differs from expected", Colors.RED)
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
                        print_colored(f"❌ Expected file not found: {expected_file}", Colors.RED)
                        self.test_details.append(f"{test_case.name}: Expected file not found")
                        return False
                    
            else:
                # Test with stdout output and compare individual files
                success, stdout, stderr = run_command(cmd)
                
                if not success:
                    print_colored(f"❌ Command failed: {' '.join(cmd)}", Colors.RED)
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
                            
                            print_colored(f"✅ Updated expected file: {expected_file_path.relative_to(self.project_root)}", Colors.GREEN)
                            return True
                        else:
                            print_colored(f"❌ Could not determine expected file path for: {test_case.expected_files}", Colors.RED)
                            return False
                    else:
                        print_colored("✅ Command executed successfully (no file to update)", Colors.GREEN)
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
                                print_colored("✅ Output matches expected", Colors.GREEN)
                                return True
                            else:
                                print_colored("❌ Output differs from expected", Colors.RED)
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
                            print_colored(f"❌ Expected file not found: {test_case.expected_files}", Colors.RED)
                            self.test_details.append(f"{test_case.name}: Expected file not found")
                            return False
                    else:
                        print_colored("✅ Command executed successfully", Colors.GREEN)
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
            print_colored(f"❌ dtolator binary not found at: {self.dtolator_binary}", Colors.RED)
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
                print_colored(f"\n❌ UPDATE FAILURES:", Colors.RED + Colors.BOLD)
                for detail in self.test_details:
                    print_colored(f"   • {detail}", Colors.RED)
            
            if self.failed_tests == 0:
                print_colored(f"\n🎉 ALL EXPECTED OUTPUTS UPDATED! ✅", Colors.GREEN + Colors.BOLD)
                print_colored(f"The output-samples directory has been refreshed with current dtolator output.", Colors.GREEN)
            else:
                print_colored(f"\n💥 {self.failed_tests} UPDATE(S) FAILED! ❌", Colors.RED + Colors.BOLD)
        else:
            print_header("Test Results Summary")
            
            total_tests = self.passed_tests + self.failed_tests
            print_colored(f"Total tests: {total_tests}", Colors.WHITE)
            print_colored(f"Passed: {self.passed_tests}", Colors.GREEN)
            print_colored(f"Failed: {self.failed_tests}", Colors.RED if self.failed_tests > 0 else Colors.WHITE)
            
            if self.failed_tests > 0:
                print_colored(f"\n❌ FAILURE DETAILS:", Colors.RED + Colors.BOLD)
                for detail in self.test_details:
                    if "failed" in detail.lower() or "differs" in detail.lower() or "not found" in detail.lower():
                        print_colored(f"   • {detail}", Colors.RED)
            
            if self.failed_tests == 0:
                print_colored(f"\n🎉 ALL TESTS PASSED! ✅", Colors.GREEN + Colors.BOLD)
            else:
                print_colored(f"\n💥 {self.failed_tests} TEST(S) FAILED! ❌", Colors.RED + Colors.BOLD)

def main():
    """Main entry point"""
    test_suite = TestSuite()
    
    # Handle command line arguments
    if len(sys.argv) > 1:
        if sys.argv[1] in ['-h', '--help']:
            print("dtolator Test Suite")
            print("Usage: python test-suite.py [options]")
            print("Options:")
            print("  -h, --help    Show this help message")
            print("  --no-build    Skip building the project (use existing binary)")
            print("  --refresh     Regenerate expected output files (updates output-samples)")
            return
        elif sys.argv[1] == '--no-build':
            # Skip build and run tests directly
            if not test_suite.dtolator_binary.exists():
                print_colored(f"❌ dtolator binary not found at: {test_suite.dtolator_binary}", Colors.RED)
                print_colored("Please build the project first with: cargo build --release", Colors.YELLOW)
                sys.exit(1)
            
            print_header("Running dtolator Test Suite (skipping build)")
            
            for test_case in test_suite.test_cases:
                if test_suite.run_single_test(test_case):
                    test_suite.passed_tests += 1
                else:
                    test_suite.failed_tests += 1
            
            test_suite.print_summary()
            sys.exit(0 if test_suite.failed_tests == 0 else 1)
        elif sys.argv[1] == '--refresh':
            # Refresh mode: regenerate expected outputs
            test_suite.refresh_mode = True
            
            # Build first
            if not test_suite.build_project():
                sys.exit(1)
            
            if not test_suite.dtolator_binary.exists():
                print_colored(f"❌ dtolator binary not found at: {test_suite.dtolator_binary}", Colors.RED)
                sys.exit(1)
            
            print_colored(f"Using dtolator binary: {test_suite.dtolator_binary}", Colors.BLUE)
            print_colored("⚠️  WARNING: This will overwrite existing files in output-samples/", Colors.YELLOW + Colors.BOLD)
            
            # Run each test case in refresh mode
            for test_case in test_suite.test_cases:
                if test_suite.run_single_test(test_case):
                    test_suite.passed_tests += 1
                else:
                    test_suite.failed_tests += 1
            
            test_suite.print_summary()
            sys.exit(0 if test_suite.failed_tests == 0 else 1)
    
    # Run all tests (including build)
    success = test_suite.run_all_tests()
    sys.exit(0 if success else 1)

if __name__ == "__main__":
    main()
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
- Python TypedDict Test: simple-sample.json -> Python TypedDict
- Python TypedDict Full: full-sample.json -> Python TypedDict

Each test runs the dtolator command with appropriate flags and compares the output
with the expected files in the output-samples directory. Any differences are
reported with detailed diff information.

TypeScript Type Checking:
When --typecheck is enabled, generated TypeScript files are validated using the
TypeScript compiler. A temporary Node.js project is created with the required
dependencies (TypeScript, Zod, Angular) and tsc --noEmit is run to check for
type errors without generating output files.
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
        
        # TypeScript test cases that should be type-checked
        self.typescript_test_cases = {
            "Angular Full Sample",
            "Angular Simple Sample", 
            "Angular Nested Test",
            "Comprehensive Nested Test",
            "Nested Test"
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
    
    def check_npm_availability(self) -> bool:
        """
        Check if npm is available in the system
        """
        # Try both npm and npm.cmd (for Windows)
        npm_commands = ["npm", "npm.cmd"]
        
        for npm_cmd in npm_commands:
            try:
                success, stdout, stderr = run_command([npm_cmd, "--version"])
                if success:
                    return True
            except Exception:
                continue
        
        return False
    
    def setup_typescript_environment(self, temp_dir: Path, is_angular: bool = False) -> bool:
        """
        Setup a minimal TypeScript environment for type checking in a temporary directory
        """
        # Check if npm is available
        if not self.check_npm_availability():
            print_colored(f"   ❌ npm is not available. Please install Node.js and npm to enable TypeScript checking.", Colors.RED)
            print_colored(f"   💡 Download from: https://nodejs.org/", Colors.YELLOW)
            return False
        
        try:
            # Create package.json with required dependencies
            package_json = {
                "name": "dtolator-typecheck",
                "version": "1.0.0",
                "private": True,
                "dependencies": {
                    "typescript": "^5.3.0",
                    "zod": "^3.22.0"
                }
            }
            
            if is_angular:
                package_json["dependencies"].update({
                    "@angular/core": "^17.0.0",
                    "@angular/common": "^17.0.0", 
                    "rxjs": "^7.8.0"
                })
            
            # Write package.json
            package_json_path = temp_dir / "package.json"
            with open(package_json_path, 'w', encoding='utf-8') as f:
                json.dump(package_json, f, indent=2)
            
            # Create tsconfig.json for type checking
            tsconfig = {
                "compilerOptions": {
                    "target": "ES2020",
                    "module": "ESNext",
                    "moduleResolution": "node",
                    "strict": True,
                    "noEmit": True,  # Type check only, don't generate files
                    "skipLibCheck": True,
                    "esModuleInterop": True,
                    "allowSyntheticDefaultImports": True,
                    "forceConsistentCasingInFileNames": True,
                    "declaration": False,
                    "declarationMap": False,
                    "sourceMap": False
                },
                "include": ["**/*.ts"],
                "exclude": ["node_modules", "**/*.spec.ts", "**/*.test.ts"]
            }
            
            if is_angular:
                tsconfig["compilerOptions"].update({
                    "experimentalDecorators": True,
                    "emitDecoratorMetadata": True
                })
            
            # Write tsconfig.json
            tsconfig_path = temp_dir / "tsconfig.json"
            with open(tsconfig_path, 'w', encoding='utf-8') as f:
                json.dump(tsconfig, f, indent=2)
            
            # Install dependencies using npm
            print_colored(f"   📦 Installing TypeScript dependencies...", Colors.BLUE)
            
            # Try npm.cmd first (Windows), then npm
            npm_cmd = "npm.cmd" if os.name == 'nt' else "npm"
            success, stdout, stderr = run_command([npm_cmd, "install", "--silent"], cwd=str(temp_dir))
            
            if not success:
                print_colored(f"   ❌ Failed to install dependencies: {stderr}", Colors.RED)
                return False
                
            print_colored(f"   ✅ Dependencies installed successfully", Colors.GREEN)
            return True
            
        except Exception as e:
            print_colored(f"   ❌ Error setting up TypeScript environment: {str(e)}", Colors.RED)
            return False
    
    def format_typescript_errors(self, tsc_output: str) -> str:
        """
        Format TypeScript compiler errors for better readability
        """
        if not tsc_output.strip():
            return "No detailed error information available"
            
        lines = tsc_output.split('\n')
        formatted_errors = []
        error_count = 0
        
        for line in lines:
            line = line.strip()
            if not line:
                continue
                
            # TypeScript error format: file.ts(line,col): error TSxxxx: message
            if '.ts(' in line and 'error TS' in line:
                formatted_errors.append(f"   🚨 {line}")
                error_count += 1
            elif line and not line.startswith('npm') and not 'Found' in line:
                # Additional context lines
                formatted_errors.append(f"      {line}")
        
        if error_count > 0:
            summary = f"\n   📊 Total TypeScript errors: {error_count}\n"
            return summary + '\n'.join(formatted_errors)
        else:
            return '\n'.join(formatted_errors) if formatted_errors else "TypeScript compilation failed with unknown errors"
    
    def execute_typescript_check(self, project_dir: Path) -> Tuple[bool, str]:
        """
        Execute TypeScript type checking using tsc --noEmit
        """
        try:
            # Use npx.cmd on Windows, npx on other systems
            npx_cmd = "npx.cmd" if os.name == 'nt' else "npx"
            tsc_cmd = [npx_cmd, "tsc", "--noEmit", "--pretty"]
            
            success, stdout, stderr = run_command(tsc_cmd, cwd=str(project_dir))
            
            if success:
                return True, "No type errors found"
            else:
                # TypeScript errors are usually in stderr, but sometimes in stdout
                error_output = stderr if stderr.strip() else stdout
                formatted_errors = self.format_typescript_errors(error_output)
                return False, formatted_errors
                
        except Exception as e:
            return False, f"Error running TypeScript compiler: {str(e)}"
    
    def run_typescript_typecheck(self, test_case: TestCase, output_dir: Path) -> bool:
        """
        Run TypeScript type checking on generated .ts files
        """
        # Only run for test cases that generate TypeScript files
        if test_case.name not in self.typescript_test_cases:
            return True  # Skip non-TypeScript tests
        
        # Check if there are any .ts files to type check
        ts_files = list(output_dir.glob("**/*.ts"))
        if not ts_files:
            print_colored(f"   ⚠️  No TypeScript files found to check", Colors.YELLOW)
            return True
        
        print_colored(f"   🔍 Found {len(ts_files)} TypeScript files to check", Colors.BLUE)
        
        # Create temporary directory for TypeScript project
        with tempfile.TemporaryDirectory() as temp_dir_str:
            temp_dir = Path(temp_dir_str)
            
            # Determine if this is an Angular project
            is_angular = "--angular" in test_case.command_args
            
            # Setup TypeScript environment
            if not self.setup_typescript_environment(temp_dir, is_angular):
                self.test_details.append(f"{test_case.name}: Failed to setup TypeScript environment")
                return False
            
            # Copy generated TypeScript files to temp directory
            try:
                for ts_file in ts_files:
                    rel_path = ts_file.relative_to(output_dir)
                    dest_path = temp_dir / rel_path
                    dest_path.parent.mkdir(parents=True, exist_ok=True)
                    shutil.copy2(ts_file, dest_path)
            except Exception as e:
                print_colored(f"   ❌ Error copying TypeScript files: {str(e)}", Colors.RED)
                self.test_details.append(f"{test_case.name}: Failed to copy TypeScript files")
                return False
            
            # Run TypeScript type checking
            print_colored(f"   🔧 Running TypeScript type check...", Colors.BLUE)
            success, error_message = self.execute_typescript_check(temp_dir)
            
            if success:
                print_colored(f"   ✅ TypeScript type check passed", Colors.GREEN)
                return True
            else:
                print_colored(f"   ❌ TypeScript type check failed", Colors.RED)
                print_colored(error_message, Colors.YELLOW)
                self.test_details.append(f"{test_case.name}: TypeScript type errors found")
                return False
    
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
                        
                        # Run TypeScript type checking if enabled
                        if self.enable_typescript_check and not self.refresh_mode:
                            if not self.run_typescript_typecheck(test_case, output_dir):
                                return False
                        
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
                            
                            # Note: TypeScript checking not applicable for single file outputs to directory
                            # because we don't have the output_dir structure needed
                            
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
                                # TypeScript checking not applicable for stdout-based tests
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
        if not test_suite.check_npm_availability():
            print_colored("❌ TypeScript checking requires npm to be installed", Colors.RED)
            print_colored("💡 Please install Node.js and npm from: https://nodejs.org/", Colors.YELLOW)
            print_colored("🔄 Disabling TypeScript checking for this run", Colors.YELLOW)
            test_suite.enable_typescript_check = False
            typecheck_enabled = False
    
    # Build project unless --no-build is specified
    if not no_build:
        if not test_suite.build_project():
            sys.exit(1)
    
    # Check if dtolator binary exists
    if not test_suite.dtolator_binary.exists():
        print_colored(f"❌ dtolator binary not found at: {test_suite.dtolator_binary}", Colors.RED)
        if no_build:
            print_colored("Please build the project first with: cargo build --release", Colors.YELLOW)
        sys.exit(1)
    
    # Print mode information
    if refresh_mode:
        print_header("Refreshing dtolator Test Expected Outputs")
        print_colored("⚠️  WARNING: This will overwrite existing files in output-samples/", Colors.YELLOW + Colors.BOLD)
    elif typecheck_enabled:
        print_header("Running dtolator Test Suite with TypeScript Checking")
        print_colored("🧪 TypeScript type checking enabled for generated .ts files", Colors.YELLOW + Colors.BOLD)
    elif no_build:
        print_header("Running dtolator Test Suite (skipping build)")
    else:
        print_header("Running dtolator Test Suite")
    
    print_colored(f"Using dtolator binary: {test_suite.dtolator_binary}", Colors.BLUE)
    
    # Run all test cases
    for test_case in test_suite.test_cases:
        if test_suite.run_single_test(test_case):
            test_suite.passed_tests += 1
        else:
            test_suite.failed_tests += 1
    
    # Print summary and exit
    test_suite.print_summary()
    sys.exit(0 if test_suite.failed_tests == 0 else 1)

if __name__ == "__main__":
    main()
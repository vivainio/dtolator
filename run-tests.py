#!/usr/bin/env python3

import argparse
import subprocess
import os

def run_tests(refresh=False):
    """Run the test suite"""
    env = os.environ.copy()
    if refresh:
        env["DTOLATOR_TEST_REFRESH"] = "1"
        print("Running dtolator test suite (refresh mode)...")
    else:
        print("Running dtolator test suite...")
    
    subprocess.run(["cargo", "test", "--test", "integration_tests", "--"], env=env)

def run_coverage():
    """Run tests with coverage analysis"""
    print("Running tests with coverage analysis...")
    if subprocess.run(["cargo", "tarpaulin", "--version"], capture_output=True).returncode != 0:
        print("Installing cargo-tarpaulin...")
        subprocess.run(["cargo", "install", "cargo-tarpaulin"], check=True)
    
    subprocess.run([
        "cargo", "tarpaulin",
        "--timeout", "300",
        "--exclude-files", "tests/*",
        "--ignore-panics",
        "--ignore-timeouts"
    ])

def main():
    parser = argparse.ArgumentParser(
        description="dtolator Test Suite",
        prog="python run-tests.py"
    )
    parser.add_argument("--refresh", action="store_true", help="Regenerate expected output files")
    parser.add_argument("--coverage", action="store_true", help="Run tests with code coverage analysis")
    
    args = parser.parse_args()
    
    if args.coverage:
        run_coverage()
    elif args.refresh:
        run_tests(refresh=True)
    else:
        run_tests(refresh=False)

if __name__ == "__main__":
    main()

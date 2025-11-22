#!/usr/bin/env python3

import argparse
import subprocess
import os


def run_tests(refresh: bool = False) -> None:
    """Run the test suite"""
    env = os.environ.copy()
    if refresh:
        env["DTOLATOR_TEST_REFRESH"] = "1"
        print("Running dtolator test suite (refresh mode)...")
    else:
        print("Running dtolator test suite...")

    subprocess.run(
        ["cargo", "test", "--test", "integration_tests", "--", "--nocapture"], env=env
    )


def run_typecheck() -> None:
    """Run TypeScript type checking"""
    print("Running TypeScript type checking...")
    subprocess.run("npm run typecheck", shell=True, check=True)


def run_biome() -> None:
    """Run biome formatter and linter on output-samples"""
    print("Running biome on output-samples...")
    subprocess.run(
        ["biome", "check", "--write", "output-samples"], check=True, shell=True
    )


def run_coverage() -> None:
    """Run tests with coverage analysis"""
    print("Running tests with coverage analysis...")
    if (
        subprocess.run(
            ["cargo", "tarpaulin", "--version"], capture_output=True
        ).returncode
        != 0
    ):
        print("Installing cargo-tarpaulin...")
        subprocess.run(["cargo", "install", "cargo-tarpaulin"], check=True)

    subprocess.run(
        [
            "cargo",
            "tarpaulin",
            "--timeout",
            "300",
            "--exclude-files",
            "tests/*",
            "--ignore-panics",
            "--ignore-timeouts",
        ]
    )


def main() -> None:
    parser = argparse.ArgumentParser(
        description="dtolator Test Suite", prog="python run-tests.py"
    )
    parser.add_argument(
        "--refresh", action="store_true", help="Regenerate expected output files"
    )
    parser.add_argument(
        "--coverage", action="store_true", help="Run tests with code coverage analysis"
    )
    parser.add_argument(
        "--biome",
        action="store_true",
        help="Refresh tests and run biome formatter/linter",
    )
    parser.add_argument(
        "--typecheck", action="store_true", help="Run TypeScript type checking"
    )

    args = parser.parse_args()

    if args.coverage:
        run_coverage()
    elif args.biome:
        run_tests(refresh=True)
        run_biome()
    elif args.typecheck:
        run_typecheck()
    elif args.refresh:
        run_tests(refresh=True)
    else:
        run_tests(refresh=False)


if __name__ == "__main__":
    main()

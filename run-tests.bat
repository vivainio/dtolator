@echo off
REM Test Suite Runner for dtolator (Rust version)
REM Usage:
REM   run-tests.bat              - Build and run all tests
REM   run-tests.bat --refresh    - Regenerate expected output files (updates output-samples)
REM   run-tests.bat --coverage   - Run tests with code coverage analysis
REM   run-tests.bat --help       - Show help information
REM

if "%1"=="--help" (
    echo dtolator Test Suite (Rust)
    echo Usage: run-tests.bat [options]
    echo Options:
    echo   --refresh              Regenerate expected output files (updates output-samples^)
    echo   --coverage             Run tests with code coverage analysis
    echo   --coverage --html      Generate HTML coverage report
    echo   --coverage --json      Generate JSON coverage report
    echo   --coverage --lcov      Generate LCOV coverage report
    echo.
    echo Examples:
    echo   run-tests.bat                        - Build and run all tests
    echo   run-tests.bat --refresh              - Update expected output files
    echo   run-tests.bat --coverage             - Run tests and show coverage stats
    echo   run-tests.bat --coverage --html      - Run tests and generate HTML report
    goto :eof
)

REM Handle coverage mode
if "%1"=="--coverage" (
    echo.
    echo ========================================
    echo Running Tests with Coverage Analysis
    echo ========================================
    echo.
    
    REM Check if cargo-tarpaulin is installed
    cargo tarpaulin --version >nul 2>&1
    if errorlevel 1 (
        echo Installing cargo-tarpaulin...
        cargo install cargo-tarpaulin
        if errorlevel 1 (
            echo Error: Failed to install cargo-tarpaulin
            goto :eof
        )
    )
    
    REM Determine output format
    set COVERAGE_FORMAT=stdout
    if "%2"=="--html" set COVERAGE_FORMAT=html
    if "%2"=="--json" set COVERAGE_FORMAT=json
    if "%2"=="--lcov" set COVERAGE_FORMAT=lcov
    
    REM Create coverage directory for non-stdout formats
    if not "%COVERAGE_FORMAT%"=="stdout" (
        if not exist "coverage" mkdir coverage
    )
    
    REM Run coverage analysis
    if "%COVERAGE_FORMAT%"=="html" (
        echo Generating HTML coverage report...
        cargo tarpaulin --out Html --output-dir coverage --timeout 300 --exclude-files tests/* --ignore-panics --ignore-timeouts
        if exist "coverage\index.html" (
            echo.
            echo ========================================
            echo Coverage Report Generated
            echo ========================================
            echo HTML Report: coverage\index.html
            echo.
        )
    ) else if "%COVERAGE_FORMAT%"=="json" (
        echo Generating JSON coverage report...
        cargo tarpaulin --out Json --output-dir coverage --timeout 300 --exclude-files tests/* --ignore-panics --ignore-timeouts
        if exist "coverage\cobertura.json" (
            echo.
            echo ========================================
            echo Coverage Report Generated
            echo ========================================
            echo JSON Report: coverage\cobertura.json
            echo.
        )
    ) else if "%COVERAGE_FORMAT%"=="lcov" (
        echo Generating LCOV coverage report...
        cargo tarpaulin --out Lcov --output-dir coverage --timeout 300 --exclude-files tests/* --ignore-panics --ignore-timeouts
        if exist "coverage\lcov.info" (
            echo.
            echo ========================================
            echo Coverage Report Generated
            echo ========================================
            echo LCOV Report: coverage\lcov.info
            echo.
        )
    ) else (
        echo Running tests with coverage analysis...
        cargo tarpaulin --timeout 300 --exclude-files tests/* --ignore-panics --ignore-timeouts
        echo.
        echo ========================================
        echo To Generate HTML Report:
        echo ========================================
        echo   run-tests.bat --coverage --html
        echo.
    )
    goto :eof
)

REM Set environment variable if --refresh is specified
if "%1"=="--refresh" set "DTOLATOR_TEST_REFRESH=1"

echo Running dtolator test suite...
cargo test --test integration_tests --

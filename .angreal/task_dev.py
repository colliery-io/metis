import subprocess
import sys
import angreal


@angreal.command(
    name='test',
    about='Run all tests across workspace crates independently',
    when_to_use=['During development', 'Before committing changes', 'In CI/CD pipelines'],
    when_not_to_use=['When only checking syntax', 'For quick formatting fixes']
)
def run_tests():
    """Run all tests in the workspace by testing each crate independently."""
    crates = ['metis-docs-core', 'metis-docs-cli', 'metis-docs-mcp']
    
    for crate in crates:
        try:
            print(f"Running tests for {crate}...")
            result = subprocess.run(['cargo', 'test', '--package', crate, '--', '--test-threads=1'], check=True)
            print(f"✓ {crate} tests passed!")
        except subprocess.CalledProcessError as e:
            print(f"✗ {crate} tests failed with exit code {e.returncode}")
            sys.exit(e.returncode)
        except FileNotFoundError:
            print("Error: cargo command not found. Ensure Rust is installed.")
            sys.exit(1)
    
    print("✓ All crate tests completed successfully!")
    return 0


@angreal.command(
    name='build',
    about='Build all crates in workspace',
    when_to_use=['To compile the project', 'Before running tests', 'To check for compilation errors'],
    when_not_to_use=['For quick syntax checking (use check instead)', 'When only formatting code']
)
def build_project():
    """Build all crates in the workspace using cargo build."""
    try:
        result = subprocess.run(['cargo', 'build'], check=True)
        return result.returncode
    except subprocess.CalledProcessError as e:
        print(f"Build failed with exit code {e.returncode}")
        sys.exit(e.returncode)
    except FileNotFoundError:
        print("Error: cargo command not found. Ensure Rust is installed.")
        sys.exit(1)


@angreal.command(
    name='clean',
    about='Clean build artifacts',
    when_to_use=['To free up disk space', 'When build cache is corrupted', 'Before fresh builds'],
    when_not_to_use=['During active development', 'When dependencies are slow to rebuild']
)
def clean_artifacts():
    """Clean build artifacts using cargo clean."""
    try:
        result = subprocess.run(['cargo', 'clean'], check=True)
        print("Build artifacts cleaned successfully")
        return result.returncode
    except subprocess.CalledProcessError as e:
        print(f"Clean failed with exit code {e.returncode}")
        sys.exit(e.returncode)
    except FileNotFoundError:
        print("Error: cargo command not found. Ensure Rust is installed.")
        sys.exit(1)


@angreal.command(
    name='test-core',
    about='Run only the core library tests (metis-docs-core)',
    when_to_use=['When you want to test only core functionality', 'During core development', 'For quick validation'],
    when_not_to_use=['When testing CLI functionality', 'For comprehensive testing']
)
def run_core_tests():
    """Run only the core library tests."""
    try:
        print("Running metis-docs-core tests...")
        result = subprocess.run(['cargo', 'test', '--package', 'metis-docs-core', '--', '--test-threads=1'], check=True)
        print("✓ All metis-docs-core tests passed!")
        return result.returncode
    except subprocess.CalledProcessError as e:
        print(f"Tests failed with exit code {e.returncode}")
        sys.exit(e.returncode)
    except FileNotFoundError:
        print("Error: cargo command not found. Ensure Rust is installed.")
        sys.exit(1)


@angreal.command(
    name='coverage',
    about='Generate coverage report using tarpaulin for all workspace crates',
    when_to_use=['To measure test coverage', 'Before releases', 'To identify untested code'],
    when_not_to_use=['During rapid development cycles', 'When tarpaulin is not installed']
)
def generate_coverage():
    """Generate coverage report using cargo tarpaulin for all workspace crates."""
    try:
        # Run tarpaulin for the workspace including all crates
        result = subprocess.run([
            'cargo', 'tarpaulin', 
            '--out', 'Html', 
            '--workspace',  # Include all workspace crates (core, cli, mcp)
            '--exclude-files', 'target/*',  # Exclude build artifacts
            '--', '--test-threads=1'  # Ensure thread safety
        ], check=True)
        print("Coverage report generated successfully for all workspace crates")
        print("Report saved to tarpaulin-report.html")
        return result.returncode
    except subprocess.CalledProcessError as e:
        print(f"Coverage generation failed with exit code {e.returncode}")
        sys.exit(e.returncode)
    except FileNotFoundError:
        print("Error: cargo tarpaulin not found. Install with: cargo install cargo-tarpaulin")
        sys.exit(1)


@angreal.command(
    name='check',
    about='Run comprehensive quality checks (clippy + format + check)',
    when_to_use=['Before committing code', 'As pre-push hook', 'For comprehensive code quality'],
    when_not_to_use=['When making quick experimental changes', 'During initial development']
)
def run_checks():
    """Run clippy, format check, and cargo check."""
    commands = [
        (['cargo', 'clippy', '--lib','--bins'], 'Clippy linting'),
        (['cargo', 'fmt', '--check'], 'Format checking'),
        (['cargo', 'check'], 'Compilation checking')
    ]
    
    for cmd, description in commands:
        print(f"Running {description}...")
        try:
            result = subprocess.run(cmd, check=True)
        except subprocess.CalledProcessError as e:
            print(f"{description} failed with exit code {e.returncode}")
            sys.exit(e.returncode)
        except FileNotFoundError:
            print("Error: cargo command not found. Ensure Rust is installed.")
            sys.exit(1)
    
    print("All quality checks passed!")
    return 0



import subprocess
import sys
import angreal


@angreal.command(
    name='test',
    about='Run all tests across workspace using modular test structure',
    when_to_use=['During development', 'Before committing changes', 'In CI/CD pipelines'],
    when_not_to_use=['When only checking syntax', 'For quick formatting fixes']
)
def run_tests():
    """Run all tests in the workspace using our modular test structure."""
    try:
        # Run unit tests for all crates
        print("Running unit tests...")
        result = subprocess.run(['cargo', 'test', '--lib'], check=True)
        
        # Run our modular integration tests
        print("Running integration tests...")
        result = subprocess.run(['cargo', 'test', '--package', 'metis-docs-mcp', '--test', 'integration'], check=True)
        
        return result.returncode
    except subprocess.CalledProcessError as e:
        print(f"Tests failed with exit code {e.returncode}")
        sys.exit(e.returncode)
    except FileNotFoundError:
        print("Error: cargo command not found. Ensure Rust is installed.")
        sys.exit(1)


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
    name='coverage',
    about='Generate coverage report using tarpaulin with modular test structure',
    when_to_use=['To measure test coverage', 'Before releases', 'To identify untested code'],
    when_not_to_use=['During rapid development cycles', 'When tarpaulin is not installed']
)
def generate_coverage():
    """Generate coverage report using cargo tarpaulin with our modular tests."""
    try:
        # Run tarpaulin with our specific test structure
        result = subprocess.run([
            'cargo', 'tarpaulin', 
            '--out', 'Html', 
            '--lib',  # Include library tests
            '--package', 'metis-docs-mcp',
            '--test', 'integration'  # Include our modular integration tests
        ], check=True)
        print("Coverage report generated successfully")
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
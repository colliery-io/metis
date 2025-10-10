import subprocess
import sys
import angreal


@angreal.command(
    name='test',
    about='Run all tests across workspace crates independently with optimized test selection',
    when_to_use=['During development', 'Before committing changes', 'In CI/CD pipelines'],
    when_not_to_use=['When only checking syntax', 'For quick formatting fixes']
)
def run_tests():
    """Run all tests in the workspace by testing each crate independently."""
    # Define crates with their optimal test strategies
    crate_configs = [
        {
            'name': 'metis-docs-core',
            'strategy': 'full',  # Has comprehensive unit tests + integration tests
            'description': 'Running comprehensive tests (unit + integration + doc)'
        },
        {
            'name': 'metis-docs-cli', 
            'strategy': 'integration_only',  # Binary crate, only integration tests
            'description': 'Running integration tests only (binary crate)'
        },
        {
            'name': 'metis-docs-mcp',
            'strategy': 'integration_only',  # Library with only integration tests
            'description': 'Running integration tests only (no unit tests defined)'
        }
    ]
    
    for config in crate_configs:
        crate = config['name']
        strategy = config['strategy']
        description = config['description']
        
        try:
            print(f"Running tests for {crate}...")
            print(f"  Strategy: {description}")
            
            if strategy == 'full':
                # Run all test types
                result = subprocess.run(['cargo', 'test', '--package', crate, '--', '--test-threads=1'], check=True)
            elif strategy == 'integration_only':
                # Run only integration tests to avoid empty unit test runs
                result = subprocess.run(['cargo', 'test', '--package', crate, '--tests', '--', '--test-threads=1'], check=True)
            
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


@angreal.command(
    name='gui',
    about='Launch Tauri GUI in development mode with hot reload',
    when_to_use=['During GUI development', 'Testing GUI changes', 'Frontend/backend integration'],
    when_not_to_use=['In CI/CD environments', 'For production builds', 'Headless environments']
)
def run_gui_dev():
    """Launch the Tauri GUI application in development mode."""
    import os
    import signal
    import shutil
    
    gui_path = os.path.join(angreal.get_root(),'..', 'crates','metis-docs-gui')
    
    # Check if GUI crate exists
    if not os.path.exists(gui_path):
        print(f"Error: GUI crate not found at {gui_path}")
        print("Run this command from the workspace root directory.")
        return 1
    
    # Check if required tools are available before starting anything
    if not shutil.which('cargo'):
        print("Error: cargo command not found. Ensure Rust is installed.")
        return 1
    
    if not shutil.which('npm'):
        print("Error: npm command not found. Ensure Node.js is installed.")
        return 1
    
    # Check if tauri CLI is available
    try:
        subprocess.run(['cargo', 'tauri', '--version'], 
                      check=True, capture_output=True, text=True)
    except (subprocess.CalledProcessError, FileNotFoundError):
        print("Error: Tauri CLI not found.")
        print("Install with: cargo install tauri-cli")
        print("Or install a specific version: cargo install tauri-cli --version '^2.0.0'")
        return 1
    
    print("Starting Metis GUI in development mode...")
    print("This will:")
    print("  - Start the Vite frontend dev server")
    print("  - Compile and launch the Tauri backend")
    print("  - Enable hot reload for frontend changes")
    print("  - Open the GUI application window")
    print()
    print("Press Ctrl+C to stop the development server")
    print()
    
    original_cwd = os.getcwd()
    process = None
    
    try:
        # Change to GUI directory
        os.chdir(gui_path)
        
        # Build frontend first
        print("Building frontend...")
        result = subprocess.run(['npm', 'run', 'build'], check=True, timeout=120)
        
        # Then run tauri dev with proper process management
        print("Starting Tauri development server...")
        
        def signal_handler(sig, frame):
            print("\nShutting down development server...")
            if process:
                try:
                    process.terminate()
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait()
            sys.exit(0)
        
        signal.signal(signal.SIGINT, signal_handler)
        signal.signal(signal.SIGTERM, signal_handler)
        
        # Start tauri dev as a managed process
        process = subprocess.Popen(['cargo', 'tauri', 'dev'], 
                                 stdout=sys.stdout, 
                                 stderr=sys.stderr)
        
        # Wait for process to complete
        return_code = process.wait()
        return return_code
        
    except subprocess.CalledProcessError as e:
        print(f"GUI development server failed with exit code {e.returncode}")
        print("\nTroubleshooting:")
        print("  - Ensure Node.js and npm are installed")
        print("  - Run 'npm install' in the GUI directory")
        print("  - Check that Tauri CLI is installed: cargo install tauri-cli")
        return e.returncode
    except subprocess.TimeoutExpired:
        print("Error: Frontend build timed out after 2 minutes")
        return 1
    except FileNotFoundError as e:
        print(f"Error: Command not found: {e}")
        return 1
    except KeyboardInterrupt:
        print("\nDevelopment server stopped by user")
        return 0
    finally:
        # Always ensure we're back in the original directory
        os.chdir(original_cwd)
        
        # Clean up any remaining process
        if process and process.poll() is None:
            try:
                process.terminate()
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait()
            except ProcessLookupError:
                pass  # Process already terminated



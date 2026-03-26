# How to Install Metis

Metis can be installed as a desktop GUI application (which includes the CLI), as a standalone CLI, or as a Claude Code plugin. Choose the method that fits your workflow.

## Install the Desktop Application (Recommended)

The desktop app includes the GUI, CLI, and MCP server. The CLI is installed automatically on first launch.

### One-Line Install

```bash
curl -fsSL https://raw.githubusercontent.com/colliery-io/metis/main/scripts/install.sh | bash
```

This script:
1. Detects your OS and architecture
2. Downloads the latest release from GitHub
3. Installs the application (macOS: `/Applications/Metis.app`, Linux: `~/.local/bin/metis`)
4. Installs the Claude Code plugin (if Claude Code is available)

### Pin a Specific Version

```bash
METIS_VERSION=2.0.4 curl -fsSL https://raw.githubusercontent.com/colliery-io/metis/main/scripts/install.sh | bash
```

### Platform-Specific Details

**macOS:**
- Installs to `/Applications/Metis.app`
- Removes quarantine attributes automatically
- CLI is installed on first GUI launch to `~/Library/Application Support/io.colliery.metis/bin/metis` with a symlink at `~/.local/bin/metis`

**Linux:**
- Installs as AppImage to `~/.local/bin/metis`
- Ensure `~/.local/bin` is in your PATH:
  ```bash
  echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
  ```

**Windows:**
- Downloads the installer to `~/Downloads/Metis_Setup.exe`
- Run the installer manually to complete installation

### Manual Download

Download release assets directly from [GitHub Releases](https://github.com/colliery-io/metis/releases):

| Platform | Asset |
|----------|-------|
| macOS (Apple Silicon) | `Metis_VERSION_aarch64.dmg` |
| macOS (Intel) | `Metis_VERSION_x64.dmg` |
| Linux | `Metis_VERSION_amd64.AppImage` |
| Windows | `Metis_VERSION_x64-setup.exe` |

## Install the CLI Only (From Source)

If you only need the CLI without the GUI:

```bash
cargo install --path crates/metis-docs-cli
```

This installs the `metis` binary to your Cargo bin directory (usually `~/.cargo/bin/`).

## Install the Claude Code Plugin

The plugin provides methodology guidance, slash commands, and automatic hooks.

### Via Plugin Marketplace

```
/plugin marketplace add colliery-io/metis
/plugin install metis@colliery-io-metis
```

### Add the MCP Server

The MCP server lets Claude Code interact with Metis tools. Add it after installing the plugin:

```
!claude mcp add --scope user metis metis mcp
```

### Verify

```
/mcp
```

Expected output:
```
metis: metis mcp - Connected
```

## Update Metis

### GUI Updates

Re-run the install script to update:

```bash
curl -fsSL https://raw.githubusercontent.com/colliery-io/metis/main/scripts/install.sh | bash
```

After updating, launch the GUI at least once to update the CLI binary. Restart Claude Code to pick up the new MCP server.

### Plugin Updates

```
/plugin marketplace update colliery-io-metis
/plugin install metis@colliery-io-metis
```

## Verify Installation

Check that the CLI is available:

```bash
metis --help
```

Check the version:

```bash
metis --version
```

Initialize a test project:

```bash
mkdir /tmp/metis-test && cd /tmp/metis-test
metis init --name "Test" --prefix "TST"
metis status
```

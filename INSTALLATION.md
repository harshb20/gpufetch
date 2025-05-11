# Installing gpufetch

This guide will help you install gpufetch on your system.

## Prerequisites

### Required Dependencies

- Rust and Cargo (from [rustup.rs](https://rustup.rs))
- libpci development headers
- libglvnd development headers (optional but recommended)

Install the dependencies according to your distribution:

**Fedora/RHEL/CentOS:**
```bash
sudo dnf install pciutils-devel libglvnd-devel
```

**Debian/Ubuntu:**
```bash
sudo apt install libpci-dev libglvnd-dev
```

**Arch Linux:**
```bash
sudo pacman -S pciutils libglvnd
```

## Installation Methods

### Method 1: Using the build script (recommended)

1. Clone the repository:
```bash
git clone https://github.com/yourusername/gpufetch.git
cd gpufetch
```

2. Make the build script executable:
```bash
chmod +x build.sh
```

3. Run the build script to install:
   - For local installation (in ~/.local/bin):
     ```bash
     ./build.sh install
     ```
   - For system-wide installation (in /usr/local/bin):
     ```bash
     ./build.sh system-install
     ```

### Method 2: Building manually with Cargo

1. Clone the repository:
```bash
git clone https://github.com/yourusername/gpufetch.git
cd gpufetch
```

2. Build the project:
```bash
cargo build --release
```

3. Install the binary:
   - Local installation:
     ```bash
     mkdir -p ~/.local/bin
     cp target/release/gpufetch ~/.local/bin/
     ```
   - System-wide installation:
     ```bash
     sudo install -Dm755 target/release/gpufetch /usr/local/bin/gpufetch
     ```

## Verifying the Installation

After installation, you can verify that gpufetch works correctly:

```bash
gpufetch
```

If you installed to ~/.local/bin and it's not in your PATH, you can either:

1. Add it to your PATH by adding this to your ~/.bashrc or ~/.zshrc:
   ```bash
   export PATH="$HOME/.local/bin:$PATH"
   ```

2. Run it using the full path:
   ```bash
   ~/.local/bin/gpufetch
   ```

## Troubleshooting

If you encounter any problems during compilation, here are some common issues and solutions:

### Missing Dependencies

If you see errors about missing libraries or headers, make sure you have installed all the required dependencies for your distribution.

### API Compatibility Issues

If you see errors about incompatible versions or missing functions in the `pci-ids` crate, you might need to update your Cargo.toml file to specify the correct version:

```toml
pci-ids = "=0.2.5"  # Pin to a specific version
```

### Permission Issues

If you encounter permission issues when installing system-wide, make sure you're using `sudo` for the installation command.

## Uninstallation

To uninstall gpufetch:

- If installed locally:
  ```bash
  rm ~/.local/bin/gpufetch
  ```

- If installed system-wide:
  ```bash
  sudo rm /usr/local/bin/gpufetch
  ```

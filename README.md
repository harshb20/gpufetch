# gpufetch

<p align="center">
  <b>A modern GPU information displaying tool in Rust</b>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/license-GPL--3.0-blue.svg" alt="License: GPL-3.0">
  <img src="https://img.shields.io/badge/rust-stable-orange.svg" alt="Rust: stable">
</p>

`gpufetch` is a command-line tool that displays detailed information about your GPU(s) in an aesthetically pleasing way. Inspired by tools like `neofetch` and `cpufetch`, it aims to provide comprehensive GPU information while being fast and lightweight.

## Features

- Detect and display information for NVIDIA, AMD, and Intel GPUs
- Work across Linux distributions (potentially BSD in the future)
- Colorful ASCII art representations of GPU brands
- Detailed hardware information including:
  - Architecture, chip name, and manufacturing process
  - Memory size, type, and bus width
  - Clock speeds and compute units
  - Cache sizes
  - Peak theoretical performance
  - And more!
- Customizable color schemes
- Multiple display options (full/compact logo, text-only)

## Example Output

```
               ⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
               ⢸⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿
           .⣿⣿⣿.     ⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿  GeForce RTX 3080
      .⣿⣿⣿⣿.   ,⣿⣿⣿.     ⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿  --------------------
   ,⣿⣿⣿'      ⣿.   ⣿⣿⣿⣿:     ⣿⣿⣿⣿⣿⣿⣿⣿⣿  Vendor: NVIDIA
.⣿⣿⣿⣿    ⣿⣿⣿⣿⣿ .      .⣿⣿⣿.    ⣿⣿⣿⣿⣿⣿⣿⣿  Architecture: Ampere
⣿⣿⣿⣿   :⣿⣿,    ⣿⣿⣿.    ⣿⣿⣿⣿    :⣿⣿⣿⣿⣿⣿⣿  Chip: GA102
 ⣿⣿⣿⣿   ⣿⣿⣿.   ⣿⣿⣿⣿⣿.⣿⣿⣿⣿    :⣿⣿⣿⣿⣿⣿⣿⣿  Process: 8 nm
  :⣿⣿⣿   ,⣿⣿⣿.  ⣿⣿⣿⣿⣿⣿⣿.   .⣿⣿⣿⣿.     ⣿⣿⣿⣿  Memory: 10 GB GDDR6X
    ⣿⣿⣿⣿.   .⣿⣿⣿.       ,⣿⣿⣿⣿⣿        .⣿⣿⣿⣿  Memory Bus: 320 bit
      ⣿⣿⣿⣿⣿:.    ,⣿⣿⣿⣿::::::::::⣿⣿⣿.        :⣿⣿⣿⣿⣿⣿⣿⣿  Core Clock: 1440 MHz
         ⣿⣿⣿⣿⣿⣿⣿⣿⣿.            '⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿  Boost Clock: 1710 MHz
               ⢸⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿  CUDA Cores: 8704
                                          Streaming Multiprocessors: 68
######.  ##   ##  ##  ######   ##    ###      Tensor Cores: 272
##   ##  ##   ##  ##  ##   ##  ##   #: :#     RT Cores: 68
##   ##   ## ##   ##  ##   ##  ##  #######    L2 Cache: 5.0 MB
##   ##    ###    ##  ######   ## ##     ##   Peak Performance: 29.77 TFLOPS
                                          Driver: 470.103.01
```

## Installation

### From Source

1. Ensure you have Rust and Cargo installed:
   ```
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. Clone the repository:
   ```
   git clone https://github.com/harshb20/gpufetch.git
   cd gpufetch
   ```

3. Build and install:
   ```
   cargo build --release
   sudo install -Dm755 target/release/gpufetch /usr/local/bin/gpufetch
   ```

### Package Managers (coming soon)

```
# Arch Linux (AUR)
yay -S gpufetch

# Fedora/RHEL
sudo dnf install gpufetch

# Debian/Ubuntu
sudo apt install gpufetch
```

## Usage

Basic usage:
```
gpufetch
```

Show all available GPUs:
```
gpufetch -g -1
```

Use a specific color scheme:
```
gpufetch -c nvidia
gpufetch -c amd
gpufetch -c intel
```

Show detailed information:
```
gpufetch -d
```

Show help:
```
gpufetch -h
```

## Dependencies

- `pci-ids`: For PCI device identification
- `sysfs-class`: For accessing sysfs
- `procfs`: For accessing procfs
- `colored`: For terminal coloring

## System Requirements

- Linux-based operating system
- Root access not required for basic functionality, but may provide more detailed information

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the GNU General Public License v3.0 - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Original [gpufetch](https://github.com/Dr-Noob/gpufetch) by Dr-Noob for inspiration
- [cpufetch](https://github.com/Dr-Noob/cpufetch) for the idea and design inspiration
- The Rust community for the excellent libraries and tools

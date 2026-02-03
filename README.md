# Dim and Dimmer

A simple Linux GUI application for controlling external monitor brightness and contrast.

## Features

- **Monitor Control** - Adjust brightness and contrast on external monitors via DDC-CI
- **Software Dimming** - Additional gamma-based dimming for X11 sessions
- **Auto-Detection** - Automatically detects connected monitors
- **Simple Interface** - Clean, intuitive sliders for quick adjustments

## Installation

### Pre-built Binary (Recommended)

Download and install the latest release:

```bash
curl -fsSL https://codeberg.org/ryankilleen/dim-and-dimmer/raw/branch/main/install.sh | sudo bash
```

To uninstall:

```bash
curl -fsSL https://codeberg.org/ryankilleen/dim-and-dimmer/raw/branch/main/install.sh | sudo bash -s -- --uninstall
```

### From crates.io

```bash
cargo install dim-and-dimmer
```

### From Source

```bash
git clone https://codeberg.org/ryankilleen/dim-and-dimmer.git
cd dim-and-dimmer
cargo build --release
sudo cp target/release/dim-and-dimmer /usr/local/bin/
```

## Requirements

- Linux with X11 (Wayland not yet supported)
- `ddcutil` installed for monitor control
- `xrandr` for software dimming
- Membership in `i2c` group (or root) for DDC-CI access

### Installing Dependencies

**Debian/Ubuntu:**
```bash
sudo apt install ddcutil x11-xserver-utils
```

**Fedora:**
```bash
sudo dnf install ddcutil xrandr
```

**Arch Linux:**
```bash
sudo pacman -S ddcutil xorg-xrandr
```

### i2c Group Access

For DDC-CI to work without root, add yourself to the `i2c` group:

```bash
sudo usermod -aG i2c $USER
```

Log out and back in for the change to take effect.

## Usage

1. Launch the application
2. Select your monitor from the dropdown
3. Adjust brightness and contrast with the sliders
4. Optionally use software dimming for additional control

## Desktop Integration

If you installed via the install script, Dim and Dimmer will appear in your application menu. For manual installations, copy the desktop file:

```bash
sudo cp assets/dim-and-dimmer.desktop /usr/share/applications/
```

## License

MIT

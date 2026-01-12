# Dim and Dimmer

A simple Linux GUI application for controlling external monitor brightness and contrast.

## Features

- **Monitor Control** - Adjust brightness and contrast on external monitors via DDC-CI
- **Software Dimming** - Additional gamma-based dimming for X11 sessions
- **Auto-Detection** - Automatically detects connected monitors
- **Simple Interface** - Clean, intuitive sliders for quick adjustments

## Requirements

- Linux with X11 (Wayland not yet supported)
- `ddcutil` installed for monitor control
- `xrandr` for software dimming

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

## Usage

1. Launch the application
2. Select your monitor from the dropdown
3. Adjust brightness and contrast with the sliders
4. Optionally use software dimming for additional control

## Building from Source

```bash
cargo build --release
```

The binary will be located at `target/release/dim-and-dimmer`.

## License

MIT

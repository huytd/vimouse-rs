# Vimouse - Vim-like Mouse Control

A Rust application that provides vim-like mouse control with keyboard shortcuts, including the ability to find and list all clickable elements on the screen.

## Features

- **Vim-like mouse movement**: Use hjkl keys for directional movement
- **Quick screen navigation**: Jump to screen quadrants using qwer/asdf/zxcv keys
- **Smooth scrolling**: Use g+hjkl for smooth momentum-based scrolling
- **Clickable elements detection**: Press 'i' to find all clickable elements on screen
- **Multiple click modes**: Space for left click, Ctrl/CapsLock for right click
- **Variable speed**: Shift for slow movement, Alt for ultra-fast movement

## New Feature: Clickable Elements Detection

Press the **'i' key** while the application is running to scan the screen and print all clickable elements to the console. This feature:

- Enumerates all windows on the screen using X11
- Excludes the vimouse application window itself
- Displays window names, application names, locations, and sizes
- Works on Linux with X11 window manager

### Example Output

```
🔍 Searching for clickable elements on screen...
Found 5 clickable elements:
--------------------------------------------------------------------------------
1. Window
   Text: "Firefox: GitHub - Example Repository"
   Location: (100, 50)
   Size: 1200x800

2. Window
   Text: "Terminal: bash"
   Location: (50, 900)
   Size: 800x400

3. Window
   Text: "VSCode: main.rs"
   Location: (1300, 100)
   Size: 1000x900

--------------------------------------------------------------------------------
Total: 5 clickable elements
```

## Key Bindings

### Movement
- `h`, `j`, `k`, `l` - Move left, down, up, right
- `y`, `u`, `b`, `n` - Diagonal movement (↖, ↗, ↙, ↘)
- `q`, `w`, `e`, `r` - Jump to top row quadrants
- `a`, `s`, `d`, `f` - Jump to middle row quadrants  
- `z`, `x`, `c`, `v` - Jump to bottom row quadrants

### Clicking
- `Space` - Left mouse click
- `Ctrl` / `CapsLock` - Right mouse click

### Scrolling
- `g` + `h`, `j`, `k`, `l` - Smooth scroll left, down, up, right
- `t` - Toggle scroll mode on/off

### Speed Control
- `Shift` - Slow movement mode
- `Alt` - Ultra-fast movement mode

### Special Features
- `i` - **Find and list all clickable elements on screen**
- `Esc` - Exit application

## Installation

### Prerequisites (Linux)

```bash
sudo apt-get update
sudo apt-get install -y libx11-dev libxi-dev libxtst-dev libevdev-dev pkg-config autotools-dev autoconf libtool
```

### Build

```bash
cargo build --release
```

### Run

```bash
./target/release/vimouse
```

**Note**: You may need to run with appropriate permissions or add the application to your system's accessibility/input monitoring settings.

## Platform Support

- **Linux**: Full support with X11 window manager
- **macOS**: Original mouse control features (clickable elements detection not yet implemented)
- **Windows**: Original mouse control features (clickable elements detection not yet implemented)

## Technical Details

The clickable elements detection feature uses:
- X11 libraries for window enumeration
- Recursive window tree traversal
- Window property extraction (name, class, position, size)
- Safe memory management with proper cleanup

## License

This project maintains the same license as the original vimouse project.
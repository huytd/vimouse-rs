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

- Enumerates all windows on the screen using macOS Core Graphics APIs
- Excludes the vimouse application window itself
- Displays window names, application names, locations, and sizes
- Works on macOS with native window management APIs

### Example Output

```
🔍 Searching for clickable elements on screen...
Found 5 clickable elements:
--------------------------------------------------------------------------------
1. Window
   Text: "Safari: GitHub - Example Repository"
   Location: (100, 50)
   Size: 1200x800

2. Window
   Text: "Terminal: bash"
   Location: (50, 900)
   Size: 800x400

3. Window
   Text: "Visual Studio Code: main.rs"
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

### Prerequisites (macOS)

Make sure you have Rust installed:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Build

```bash
cargo build --release
```

### Run

```bash
./target/release/vimouse
```

**Note**: On macOS, you'll need to grant accessibility permissions to the application. When you first run it, macOS will prompt you to add it to System Preferences > Security & Privacy > Privacy > Accessibility.

## Platform Support

- **macOS**: Full support with Core Graphics and GPUI
- **Linux/Windows**: Original mouse control features (clickable elements detection not implemented)

## Technical Details

The clickable elements detection feature uses:
- Core Graphics `CGWindowListCopyWindowInfo` for window enumeration
- Core Foundation data types for safe memory management
- Window property extraction (name, owner, bounds)
- Filtering to exclude system and self windows

## Permissions

On macOS, this application requires:
- **Accessibility Access**: For mouse control and input monitoring
- **Input Monitoring**: For global key capture

The system will prompt you to grant these permissions when you first run the application.

## License

This project maintains the same license as the original vimouse project.
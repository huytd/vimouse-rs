# Vimouse - Vim-like Mouse Control

A Rust application that provides vim-like mouse control with keyboard shortcuts, including the ability to find and list all clickable elements on the screen.

## Features

- **Vim-like mouse movement**: Use hjkl keys for directional movement
- **Quick screen navigation**: Jump to screen quadrants using qwer/asdf/zxcv keys
- **Smooth scrolling**: Use g+hjkl for smooth momentum-based scrolling
- **Clickable elements detection**: Press 'i' to find all clickable elements on screen
- **Multiple click modes**: Space for left click, Ctrl/CapsLock for right click
- **Variable speed**: Shift for slow movement, Alt for ultra-fast movement
- **Cross-platform**: Works on macOS, Linux, and Windows

## New Feature: Clickable Elements Detection

Press the **'i' key** while the application is running to scan the screen and print all clickable elements to the console. This feature:

- **On macOS**: Uses Core Graphics APIs to enumerate actual windows
- **On other platforms**: Shows sample data for demonstration purposes
- Excludes the vimouse application window itself
- Displays window names, applications, locations, and sizes

### Example Output

```
🔍 Searching for clickable elements on screen...
Found 2 clickable elements:
--------------------------------------------------------------------------------
1. Window
   Text: "Sample Window 1"
   Location: (100, 100)
   Size: 800x600

2. Window
   Text: "Sample Window 2"
   Location: (200, 200)
   Size: 600x400

--------------------------------------------------------------------------------
Total: 2 clickable elements
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

### Prerequisites

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

The application will display helpful information in the console:

```
🐭 Vimouse - Vim-like Mouse Control (macOS)
Press 'i' to find clickable elements on screen
Press 'Esc' to exit
Use hjkl for movement, space for click, g+hjkl for scroll
Screen size: 2560x1440

🔑 Key Bindings:
   Movement: h/j/k/l (left/down/up/right)
   Click: Space (left), Ctrl (right)
   Speed: Shift (slow), Alt (fast)
   Scroll: g+hjkl, t (toggle)
   Detect: i (find clickable elements)
   Exit: Esc

⚠️  Note: You may need to grant accessibility permissions in System Preferences.
Starting mouse control...
```

## Platform Support

- **macOS**: Full support with Core Graphics APIs for real window detection
- **Linux/Windows**: Core mouse control features with sample clickable elements data

## Technical Details

### Architecture
- **Cross-platform compatibility**: Uses conditional compilation for platform-specific features
- **No unstable Rust features**: Compatible with stable Rust compiler
- **Simple dependencies**: Minimal dependency footprint for reliability
- **Console-based interface**: No GUI framework dependencies

### macOS Implementation
- Core Graphics `CGWindowListCopyWindowInfo` for window enumeration
- Core Foundation data types for safe memory management
- Platform-specific mouse position detection
- Native accessibility integration

### Error Handling
- Graceful degradation when permissions are not available
- Clear error messages with troubleshooting steps
- Platform-appropriate guidance for users

## Permissions

### macOS
On macOS, this application requires:
- **Accessibility Access**: For mouse control and input monitoring
- **Input Monitoring**: For global key capture

### Troubleshooting Permissions (macOS)

If you get a permissions error:

1. Go to **System Preferences** > **Security & Privacy** > **Privacy**
2. Select **Accessibility** from the left panel
3. Click the lock icon and enter your password
4. Click the **+** button and add the vimouse executable
5. Make sure the checkbox next to vimouse is enabled
6. Restart the application

### Other Platforms
Other platforms may require appropriate permissions for:
- Global key capture
- Mouse simulation
- Input monitoring

## Development Notes

This implementation was designed to:
- Avoid unstable Rust features and complex dependency chains
- Provide a working foundation that can be enhanced per platform
- Maintain the original vimouse functionality while adding new features
- Be easily buildable and deployable across different environments

## License

This project maintains the same license as the original vimouse project.
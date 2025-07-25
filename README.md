# Vimouse - Vim-like Mouse Control

A Rust application that provides vim-like mouse control with keyboard shortcuts, including **real clickable elements detection** using macOS Accessibility APIs.

## Features

- **Vim-like mouse movement**: Use hjkl keys for directional movement
- **Quick screen navigation**: Jump to screen quadrants using qwer/asdf/zxcv keys
- **Smooth scrolling**: Use g+hjkl for smooth momentum-based scrolling
- **🆕 Real clickable elements detection**: Press 'i' to find actual UI elements on screen
- **Multiple click modes**: Space for left click, Ctrl/CapsLock for right click
- **Variable speed**: Shift for slow movement, Alt for ultra-fast movement
- **Cross-platform**: Works on macOS with full features, other platforms with core functionality

## 🔍 Real Clickable Elements Detection

Press the **'i' key** while the application is running to perform a comprehensive scan of all interactive UI elements on your screen.

### What It Actually Detects

**Real UI Elements (macOS only):**
- 🔘 **Buttons** - All types (standard, menu, popup buttons)
- 📝 **Text Fields** - Input fields, text areas, search fields
- ☑️ **Checkboxes** - Interactive checkbox controls
- 🔵 **Radio Buttons** - Radio button selections
- 🔗 **Links** - Clickable hyperlinks
- 📋 **Menus** - Menu items and dropdown options
- 📂 **Tabs** - Tab controls and tab groups
- 🎚️ **Sliders** - Interactive slider controls
- 🖼️ **Images** - Clickable image elements
- 📊 **Tables/Lists** - Interactive table and list elements

### Technical Implementation

**macOS (Full Implementation):**
- Uses **macOS Accessibility APIs** (`AXUIElement`)
- Traverses the complete UI hierarchy of all applications
- Filters for genuinely clickable/interactive elements
- Extracts real properties: position, size, title, value, enabled state
- Excludes invisible, disabled, or non-interactive elements

**Other Platforms:**
- Displays appropriate "not supported" message
- No fake data - honest about platform limitations

### Example Output

```
🔍 Searching for clickable elements on screen...
Found 23 clickable elements in 45.2ms:
------------------------------------------------------------------------------------------
📊 Element types found:
   • AXButton: 8 elements
   • AXTextField: 3 elements  
   • AXCheckBox: 2 elements
   • AXLink: 4 elements
   • AXMenuItem: 6 elements

1. 🔘 AXButton
   Text: "Save Document"
   Location: (150, 200)
   Size: 80×24
   Click area: 1920 sq pixels

2. 📝 AXTextField
   Text: "Enter your name"
   Location: (200, 300)
   Size: 200×22
   Click area: 4400 sq pixels

3. ☑️ AXCheckBox
   Text: "Enable notifications"
   Location: (50, 450)
   Size: 16×16
   Click area: 256 sq pixels

------------------------------------------------------------------------------------------
✅ Total: 23 clickable elements | Scan time: 45.2ms
💡 Tip: Use mouse coordinates to click on these elements programmatically
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
- `i` - **🔍 Find and list all real clickable elements on screen**
- `Esc` - Exit application

## Installation & Usage

### Prerequisites (macOS)

1. **Rust Installation:**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Accessibility Permissions:**
   - The app will prompt you to grant accessibility permissions
   - This is **required** for both mouse control and UI element detection

### Build & Run

```bash
# Build
cargo build --release

# Run
./target/release/vimouse
```

### Example Console Output

```
🐭 Vimouse - Vim-like Mouse Control (macOS)
Press 'i' to find clickable elements on screen
Press 'Esc' to exit
Use hjkl for movement, space for click, g+hjkl for scroll
Screen size: 2560×1440

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

- **macOS**: ✅ Full support with real UI element detection via Accessibility APIs
- **Linux/Windows**: ⚠️ Core mouse control only (no clickable elements detection)

## Technical Architecture

### macOS Implementation
- **Accessibility APIs**: Direct integration with `AXUIElement` APIs
- **System-wide scanning**: Enumerates all applications and their UI hierarchies
- **Smart filtering**: Only returns genuinely interactive elements
- **Property extraction**: Real position, size, text, and state information
- **Performance optimized**: Efficient recursive traversal with depth limiting

### Safety & Reliability
- **Memory management**: Proper CFRelease for all Core Foundation objects
- **Error handling**: Graceful degradation when permissions are denied
- **Recursion protection**: Prevents infinite loops in complex UI hierarchies
- **Type safety**: Strong typing for all Accessibility API interactions

### Element Classification
The system identifies elements by their actual accessibility roles:
- Validates element is enabled and visible
- Extracts meaningful text (title, value, or role-based description)
- Calculates accurate screen coordinates and dimensions
- Filters out decorative or non-interactive elements

## Permissions & Troubleshooting

### Required Permissions (macOS)
- **Accessibility Access**: For UI element detection and mouse control
- **Input Monitoring**: For global key capture

### Setup Instructions
1. Run the application
2. When prompted, go to **System Preferences** → **Security & Privacy** → **Privacy**
3. Select **Accessibility** from the left panel
4. Click the lock icon and authenticate
5. Click **+** and add the vimouse executable
6. Ensure the checkbox is enabled
7. Restart the application

### Troubleshooting
If element detection returns no results:
- ✓ Verify accessibility permissions are granted
- ✓ Ensure applications with UI elements are open and visible
- ✓ Check that elements are not hidden behind other windows
- ✓ Try running with different applications in focus

## Use Cases

- **Accessibility Testing**: Identify all interactive elements for testing
- **UI Automation**: Get precise coordinates for automated clicking
- **Development Debugging**: Understand UI element hierarchy and properties
- **Screen Reading**: Programmatically access UI element information
- **Quality Assurance**: Verify all expected interactive elements are present

## License

This project maintains the same license as the original vimouse project.
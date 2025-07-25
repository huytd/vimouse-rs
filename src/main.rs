use lazy_static::lazy_static;
use rdev::{display_size, grab, simulate, Button, Event, EventType, Key};
use std::{collections::HashMap, thread, time};

#[cfg(target_os = "macos")]
use core_graphics::event::CGEvent;
#[cfg(target_os = "macos")]
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

const SLOW_SPEED: f64 = 5.0;
const FAST_SPEED: f64 = 40.0;
const ULTRA_FAST_SPEED: f64 = 150.0;

// Smooth scroll constants
const SCROLL_INITIAL_VELOCITY: f64 = 20.0;
const SCROLL_DECELERATION: f64 = 0.85; // Momentum decay factor
const SCROLL_MIN_VELOCITY: f64 = 0.5; // Minimum velocity before stopping
const SCROLL_FRAME_DELAY_MS: u64 = 16; // ~60 FPS for smooth animation

static mut MOUSE_POSITION: (f64, f64) = (0., 0.);
static mut MOUSE_SPEED: f64 = FAST_SPEED;
static mut G_KEY_HELD: bool = false;

static mut SCREEN_WIDTH: f64 = 0.;
static mut SCREEN_HEIGHT: f64 = 0.;

#[derive(Debug, Clone)]
pub struct ClickableElement {
    pub text: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub role: String,
}

#[cfg(target_os = "macos")]
mod clickable_detector {
    use super::*;
    use core_graphics::window::{
        kCGWindowListOptionOnScreenOnly, kCGWindowListExcludeDesktopElements,
        CGWindowListCopyWindowInfo, kCGNullWindowID
    };
    use core_foundation::{
        array::CFArray,
        base::{CFTypeRef, TCFType},
        dictionary::CFDictionary,
        string::CFString,
        number::CFNumber,
    };
    use accessibility_sys::*;
    use std::ptr;

    pub fn find_clickable_elements() -> Vec<ClickableElement> {
        let mut elements = Vec::new();
        
        unsafe {
            // First get the system-wide accessibility element
            let system_wide = AXUIElementCreateSystemWide();
            if system_wide.is_null() {
                println!("Failed to create system-wide accessibility element");
                return elements;
            }
            
            // Get all applications
            let mut apps_ref: CFTypeRef = ptr::null();
            let result = AXUIElementCopyAttributeValue(
                system_wide, 
                kAXApplicationsAttribute, 
                &mut apps_ref
            );
            
            if result != kAXErrorSuccess || apps_ref.is_null() {
                println!("Failed to get applications list. Error code: {}", result);
                CFRelease(system_wide as CFTypeRef);
                return elements;
            }
            
            let apps_array: CFArray<AXUIElementRef> = CFArray::wrap_under_create_rule(apps_ref as _);
            
            // Iterate through each application
            for i in 0..apps_array.len() {
                if let Some(app_ref) = apps_array.get(i) {
                    let app_element = *app_ref;
                    
                    // Skip our own application
                    if let Some(app_name) = get_app_name(app_element) {
                        if app_name.to_lowercase().contains("vimouse") {
                            continue;
                        }
                    }
                    
                    // Get windows for this application
                    collect_app_elements(app_element, &mut elements);
                }
            }
            
            CFRelease(system_wide as CFTypeRef);
        }
        
        elements
    }

    unsafe fn collect_app_elements(app_element: AXUIElementRef, elements: &mut Vec<ClickableElement>) {
        // Get windows for this application
        let mut windows_ref: CFTypeRef = ptr::null();
        let result = AXUIElementCopyAttributeValue(
            app_element,
            kAXWindowsAttribute,
            &mut windows_ref
        );
        
        if result != kAXErrorSuccess || windows_ref.is_null() {
            return;
        }
        
        let windows_array: CFArray<AXUIElementRef> = CFArray::wrap_under_create_rule(windows_ref as _);
        
        // Iterate through each window
        for i in 0..windows_array.len() {
            if let Some(window_ref) = windows_array.get(i) {
                let window_element = *window_ref;
                collect_window_elements(window_element, elements, 0);
            }
        }
    }

    unsafe fn collect_window_elements(element: AXUIElementRef, elements: &mut Vec<ClickableElement>, depth: i32) {
        // Prevent infinite recursion
        if depth > 10 {
            return;
        }
        
        // Check if this element is clickable
        if is_clickable_element(element) {
            if let Some(clickable_elem) = create_clickable_element(element) {
                elements.push(clickable_elem);
            }
        }
        
        // Get children and recurse
        let mut children_ref: CFTypeRef = ptr::null();
        let result = AXUIElementCopyAttributeValue(
            element,
            kAXChildrenAttribute,
            &mut children_ref
        );
        
        if result == kAXErrorSuccess && !children_ref.is_null() {
            let children_array: CFArray<AXUIElementRef> = CFArray::wrap_under_create_rule(children_ref as _);
            
            for i in 0..children_array.len() {
                if let Some(child_ref) = children_array.get(i) {
                    let child_element = *child_ref;
                    collect_window_elements(child_element, elements, depth + 1);
                }
            }
        }
    }

    unsafe fn is_clickable_element(element: AXUIElementRef) -> bool {
        // Get the role of the element
        if let Some(role) = get_element_role(element) {
            match role.as_str() {
                "AXButton" | "AXMenuButton" | "AXPopUpButton" | "AXCheckBox" | 
                "AXRadioButton" | "AXTextField" | "AXTextArea" | "AXSearchField" |
                "AXLink" | "AXMenuItem" | "AXTab" | "AXSlider" | "AXIncrementor" |
                "AXDecrementor" | "AXComboBox" | "AXDisclosureTriangle" |
                "AXStepper" | "AXSegmentedControl" | "AXTabGroup" | "AXScrollBar" |
                "AXTable" | "AXOutline" | "AXList" | "AXImage" => {
                    // Additional check: element should be enabled and visible
                    is_element_enabled(element) && is_element_visible(element)
                },
                _ => false
            }
        } else {
            false
        }
    }

    unsafe fn create_clickable_element(element: AXUIElementRef) -> Option<ClickableElement> {
        let role = get_element_role(element).unwrap_or("Unknown".to_string());
        let title = get_element_title(element).unwrap_or("".to_string());
        let value = get_element_value(element).unwrap_or("".to_string());
        let position = get_element_position(element).unwrap_or((0.0, 0.0));
        let size = get_element_size(element).unwrap_or((0.0, 0.0));
        
        // Create descriptive text
        let text = if !title.is_empty() {
            title
        } else if !value.is_empty() {
            value
        } else {
            format!("{} Element", role)
        };
        
        Some(ClickableElement {
            text,
            x: position.0,
            y: position.1,
            width: size.0,
            height: size.1,
            role,
        })
    }

    unsafe fn get_app_name(app_element: AXUIElementRef) -> Option<String> {
        get_element_attribute_string(app_element, kAXTitleAttribute)
    }

    unsafe fn get_element_role(element: AXUIElementRef) -> Option<String> {
        get_element_attribute_string(element, kAXRoleAttribute)
    }

    unsafe fn get_element_title(element: AXUIElementRef) -> Option<String> {
        get_element_attribute_string(element, kAXTitleAttribute)
    }

    unsafe fn get_element_value(element: AXUIElementRef) -> Option<String> {
        get_element_attribute_string(element, kAXValueAttribute)
    }

    unsafe fn is_element_enabled(element: AXUIElementRef) -> bool {
        let mut enabled_ref: CFTypeRef = ptr::null();
        let result = AXUIElementCopyAttributeValue(
            element,
            kAXEnabledAttribute,
            &mut enabled_ref
        );
        
        if result == kAXErrorSuccess && !enabled_ref.is_null() {
            let enabled_num = enabled_ref as *const CFNumber;
            if let Some(enabled_cf) = CFNumber::wrap_under_get_rule(enabled_num) {
                enabled_cf.to_i32().unwrap_or(0) != 0
            } else {
                true // Default to enabled if we can't determine
            }
        } else {
            true // Default to enabled
        }
    }

    unsafe fn is_element_visible(element: AXUIElementRef) -> bool {
        // Check if element has a position (visible elements should have position)
        get_element_position(element).is_some()
    }

    unsafe fn get_element_position(element: AXUIElementRef) -> Option<(f64, f64)> {
        let mut position_ref: CFTypeRef = ptr::null();
        let result = AXUIElementCopyAttributeValue(
            element,
            kAXPositionAttribute,
            &mut position_ref
        );
        
        if result == kAXErrorSuccess && !position_ref.is_null() {
            let mut point = CGPoint { x: 0.0, y: 0.0 };
            let success = AXValueGetValue(
                position_ref as AXValueRef,
                kAXValueCGPointType,
                &mut point as *mut _ as *mut _
            );
            
            CFRelease(position_ref);
            
            if success {
                Some((point.x, point.y))
            } else {
                None
            }
        } else {
            None
        }
    }

    unsafe fn get_element_size(element: AXUIElementRef) -> Option<(f64, f64)> {
        let mut size_ref: CFTypeRef = ptr::null();
        let result = AXUIElementCopyAttributeValue(
            element,
            kAXSizeAttribute,
            &mut size_ref
        );
        
        if result == kAXErrorSuccess && !size_ref.is_null() {
            let mut size = CGSize { width: 0.0, height: 0.0 };
            let success = AXValueGetValue(
                size_ref as AXValueRef,
                kAXValueCGSizeType,
                &mut size as *mut _ as *mut _
            );
            
            CFRelease(size_ref);
            
            if success {
                Some((size.width, size.height))
            } else {
                None
            }
        } else {
            None
        }
    }

    unsafe fn get_element_attribute_string(element: AXUIElementRef, attribute: CFStringRef) -> Option<String> {
        let mut attr_ref: CFTypeRef = ptr::null();
        let result = AXUIElementCopyAttributeValue(element, attribute, &mut attr_ref);
        
        if result == kAXErrorSuccess && !attr_ref.is_null() {
            let cf_string = CFString::wrap_under_create_rule(attr_ref as _);
            Some(cf_string.to_string())
        } else {
            None
        }
    }

    // Define CGPoint and CGSize for AXValue conversion
    #[repr(C)]
    struct CGPoint {
        x: f64,
        y: f64,
    }

    #[repr(C)]
    struct CGSize {
        width: f64,
        height: f64,
    }
}

#[cfg(not(target_os = "macos"))]
mod clickable_detector {
    use super::*;
    
    pub fn find_clickable_elements() -> Vec<ClickableElement> {
        println!("⚠️  Clickable element detection is only supported on macOS currently");
        println!("    The accessibility APIs required for this feature are platform-specific.");
        Vec::new()
    }
}

fn print_clickable_elements() {
    println!("🔍 Searching for clickable elements on screen...");
    let start_time = std::time::Instant::now();
    let elements = clickable_detector::find_clickable_elements();
    let duration = start_time.elapsed();
    
    if elements.is_empty() {
        println!("No clickable elements found.");
        #[cfg(target_os = "macos")]
        {
            println!("💡 This might be because:");
            println!("   • Accessibility permissions are not granted");
            println!("   • No applications with UI elements are currently open");
            println!("   • UI elements are not currently visible");
        }
        return;
    }
    
    println!("Found {} clickable elements in {:.2}ms:", elements.len(), duration.as_millis());
    println!("{:-<90}", "");
    
    // Group elements by role for better organization
    let mut role_counts = std::collections::HashMap::new();
    for element in &elements {
        *role_counts.entry(&element.role).or_insert(0) += 1;
    }
    
    // Print summary
    println!("📊 Element types found:");
    for (role, count) in &role_counts {
        println!("   • {}: {} elements", role, count);
    }
    println!();
    
    // Print detailed list
    for (i, element) in elements.iter().enumerate() {
        println!("{}. {} {}", i + 1, 
            match element.role.as_str() {
                "AXButton" => "🔘",
                "AXTextField" | "AXTextArea" | "AXSearchField" => "📝",
                "AXCheckBox" => "☑️",
                "AXRadioButton" => "🔵",
                "AXLink" => "🔗",
                "AXMenuItem" => "📋",
                "AXTab" => "📂",
                "AXSlider" => "🎚️",
                "AXComboBox" | "AXPopUpButton" => "📋",
                "AXImage" => "🖼️",
                "AXTable" | "AXOutline" | "AXList" => "📊",
                _ => "🔳"
            },
            element.role
        );
        
        if !element.text.is_empty() && element.text != format!("{} Element", element.role) {
            println!("   Text: \"{}\"", element.text);
        }
        
        println!("   Location: ({:.0}, {:.0})", element.x, element.y);
        println!("   Size: {:.0}×{:.0}", element.width, element.height);
        
        // Add clickability info
        if element.width > 0.0 && element.height > 0.0 {
            println!("   Click area: {:.0} sq pixels", element.width * element.height);
        }
        
        println!();
    }
    
    println!("{:-<90}", "");
    println!("✅ Total: {} clickable elements | Scan time: {:.2}ms", elements.len(), duration.as_millis());
    
    #[cfg(target_os = "macos")]
    {
        println!("💡 Tip: Use mouse coordinates to click on these elements programmatically");
    }
}

lazy_static! {
    static ref MOVEMENT_MAP: HashMap<Key, (f64, f64)> = HashMap::from([
        (Key::KeyH, (-1., 0.)),
        (Key::KeyL, (1., 0.)),
        (Key::KeyJ, (0., 1.)),
        (Key::KeyK, (0., -1.)),
        (Key::KeyY, (-1., -1.)),
        (Key::KeyU, (1., -1.)),
        (Key::KeyB, (-1., 1.)),
        (Key::KeyN, (1., 1.)),
    ]);
    static ref SCREEN_CELL_MAP: HashMap<Key, (f64, f64)> = HashMap::from([
        (Key::KeyQ, (0., 0.)),
        (Key::KeyW, (1., 0.)),
        (Key::KeyE, (2., 0.)),
        (Key::KeyR, (3., 0.)),
        (Key::KeyA, (0., 1.)),
        (Key::KeyS, (1., 1.)),
        (Key::KeyD, (2., 1.)),
        (Key::KeyF, (3., 1.)),
        (Key::KeyZ, (0., 2.)),
        (Key::KeyX, (1., 2.)),
        (Key::KeyC, (2., 2.)),
        (Key::KeyV, (3., 2.)),
    ]);
}

#[cfg(target_os = "macos")]
fn get_current_mouse_position() -> Option<(f64, f64)> {
    // Get the current mouse position using Core Graphics
    let event_source = CGEventSource::new(CGEventSourceStateID::CombinedSessionState).ok()?;
    let event = CGEvent::new(event_source).ok()?;
    let location = event.location();
    Some((location.x, location.y))
}

#[cfg(not(target_os = "macos"))]
fn get_current_mouse_position() -> Option<(f64, f64)> {
    // Fallback for non-macOS platforms
    None
}

fn send(event_type: &EventType) {
    let delay = time::Duration::from_millis(5);
    match simulate(event_type) {
        Ok(()) => (),
        Err(err) => {
            println!("We could not send {event_type:?}: {err:?}");
        }
    }
    // Let ths OS catchup (at least MacOS)
    thread::sleep(delay);
}

// Send smooth scroll with momentum and easing
fn send_smooth_scroll(direction_x: f64, direction_y: f64) {
    // Spawn a thread to handle the smooth scroll animation
    thread::spawn(move || {
        let mut velocity_x = direction_x * SCROLL_INITIAL_VELOCITY;
        let mut velocity_y = direction_y * SCROLL_INITIAL_VELOCITY;

        // Continue scrolling until velocity drops below minimum
        while velocity_x.abs() > SCROLL_MIN_VELOCITY || velocity_y.abs() > SCROLL_MIN_VELOCITY {
            // Send scroll event with current velocity
            let delta_x = velocity_x as i64;
            let delta_y = velocity_y as i64;

            if delta_x != 0 || delta_y != 0 {
                let _ = simulate(&EventType::Wheel { delta_x, delta_y });
            }

            // Apply deceleration
            velocity_x *= SCROLL_DECELERATION;
            velocity_y *= SCROLL_DECELERATION;

            // Stop if velocity is too small
            if velocity_x.abs() < SCROLL_MIN_VELOCITY {
                velocity_x = 0.0;
            }
            if velocity_y.abs() < SCROLL_MIN_VELOCITY {
                velocity_y = 0.0;
            }

            // Wait for next frame
            thread::sleep(time::Duration::from_millis(SCROLL_FRAME_DELAY_MS));
        }
    });
}

fn callback(event: Event) -> Option<Event> {
    unsafe {
        return match event.event_type {
            EventType::MouseMove { x, y } => {
                MOUSE_POSITION = (x, y);
                return Some(event);
            }
            EventType::KeyPress(key) => {
                return match key {
                    /* Movement directions:
                     *
                     *  y  k  u
                     *   ↖ ↑ ↗
                     * h ← . → l
                     *   ↙ ↓ ↘
                     *  b  j  n
                     *
                     */
                    Key::KeyH
                    | Key::KeyL
                    | Key::KeyK
                    | Key::KeyJ
                    | Key::KeyY
                    | Key::KeyU
                    | Key::KeyB
                    | Key::KeyN => {
                        if G_KEY_HELD {
                            // Scroll mode: only handle h, l, j, k for scrolling
                            match key {
                                Key::KeyH => {
                                    // Scroll left with smooth momentum
                                    send_smooth_scroll(-1.0, 0.0);
                                    return None;
                                }
                                Key::KeyL => {
                                    // Scroll right with smooth momentum
                                    send_smooth_scroll(1.0, 0.0);
                                    return None;
                                }
                                Key::KeyJ => {
                                    // Scroll down with smooth momentum
                                    send_smooth_scroll(0.0, -1.0);
                                    return None;
                                }
                                Key::KeyK => {
                                    // Scroll up with smooth momentum
                                    send_smooth_scroll(0.0, 1.0);
                                    return None;
                                }
                                _ => {
                                    // Other movement keys are ignored in scroll mode
                                    return None;
                                }
                            }
                        } else {
                            // Normal movement mode
                            if let Some(direction) = MOVEMENT_MAP.get(&key) {
                                send(&EventType::MouseMove {
                                    x: MOUSE_POSITION.0 + direction.0 * MOUSE_SPEED,
                                    y: MOUSE_POSITION.1 + direction.1 * MOUSE_SPEED,
                                });
                                return None;
                            }
                        }
                        return Some(event);
                    }
                    /* Mouse clicks:
                     * - Space: Left click
                     * - Ctrl: Right click
                     */
                    Key::Space => {
                        send(&EventType::ButtonPress(Button::Left));
                        return None;
                    }
                    Key::ControlLeft | Key::ControlRight | Key::CapsLock => {
                        send(&EventType::ButtonPress(Button::Right));
                        return None;
                    }
                    /* Quick jump to a specific
                     * area on the screen:
                     *  ┌─────┬─────┬─────┬─────┐
                     *  │  Q  │  W  │  E  │  R  │
                     *  ├─────┼─────┼─────┼─────┤
                     *  │  A  │  S  │  D  │  F  │
                     *  ├─────┼─────┼─────┼─────┤
                     *  │  Z  │  X  │  C  │  V  │
                     *  └─────┴─────┴─────┴─────┘
                     */
                    Key::KeyQ
                    | Key::KeyW
                    | Key::KeyE
                    | Key::KeyR
                    | Key::KeyA
                    | Key::KeyS
                    | Key::KeyD
                    | Key::KeyF
                    | Key::KeyZ
                    | Key::KeyX
                    | Key::KeyC
                    | Key::KeyV => {
                        if let Some((col, row)) = SCREEN_CELL_MAP.get(&key) {
                            let (x, y) = (
                                col * SCREEN_WIDTH / 4. + SCREEN_WIDTH / 8.,
                                row * SCREEN_HEIGHT / 3. + SCREEN_HEIGHT / 6.,
                            );
                            send(&EventType::MouseMove { x, y });
                            return None;
                        }
                        return Some(event);
                    }
                    /* Others:
                     * - Esc: Exit
                     * - Shift: Slow speed
                     * - Alt: Fast speed
                     */
                    Key::Escape => {
                        std::process::exit(0);
                    }
                    Key::ShiftLeft | Key::ShiftRight => {
                        MOUSE_SPEED = SLOW_SPEED;
                        return Some(event);
                    }
                    Key::Alt => {
                        MOUSE_SPEED = ULTRA_FAST_SPEED;
                        return Some(event);
                    }
                    Key::KeyG => {
                        G_KEY_HELD = true;
                        return None;
                    }
                    Key::KeyT => {
                        G_KEY_HELD = !G_KEY_HELD;
                        return None;
                    }
                    Key::KeyI => {
                        // Print clickable elements to console
                        thread::spawn(|| {
                            print_clickable_elements();
                        });
                        return None;
                    }
                    _ => Some(event),
                };
            }
            EventType::KeyRelease(key) => {
                return match key {
                    Key::Space => {
                        send(&EventType::ButtonRelease(Button::Left));
                        return None;
                    }
                    Key::ControlLeft | Key::ControlRight | Key::CapsLock => {
                        send(&EventType::ButtonRelease(Button::Right));
                        return None;
                    }
                    Key::ShiftLeft | Key::ShiftRight | Key::Alt => {
                        MOUSE_SPEED = FAST_SPEED;
                        return Some(event);
                    }
                    Key::KeyG => {
                        G_KEY_HELD = false;
                        return None;
                    }
                    _ => Some(event),
                }
            }
            _ => Some(event),
        };
    }
}

fn main() {
    let platform = if cfg!(target_os = "macos") {
        "macOS"
    } else if cfg!(target_os = "linux") {
        "Linux"
    } else if cfg!(target_os = "windows") {
        "Windows"
    } else {
        "Unknown"
    };

    println!("🐭 Vimouse - Vim-like Mouse Control ({})", platform);
    println!("Press 'i' to find clickable elements on screen");
    println!("Press 'Esc' to exit");
    println!("Use hjkl for movement, space for click, g+hjkl for scroll");
    
    if let Ok((w, h)) = display_size() {
        unsafe {
            SCREEN_WIDTH = w as f64;
            SCREEN_HEIGHT = h as f64;

            // Get current mouse position instead of defaulting to center
            if let Some(current_pos) = get_current_mouse_position() {
                MOUSE_POSITION = current_pos;
            } else {
                // Fallback to center if we can't get current position
                MOUSE_POSITION = (SCREEN_WIDTH / 2., SCREEN_HEIGHT / 2.);
            }
        }
        println!("Screen size: {}x{}", w, h);
    }

    println!("\n🔑 Key Bindings:");
    println!("   Movement: h/j/k/l (left/down/up/right)");
    println!("   Click: Space (left), Ctrl (right)");
    println!("   Speed: Shift (slow), Alt (fast)");
    println!("   Scroll: g+hjkl, t (toggle)");
    println!("   Detect: i (find clickable elements)");
    println!("   Exit: Esc");

    if cfg!(target_os = "macos") {
        println!("\n⚠️  Note: You may need to grant accessibility permissions in System Preferences.");
    }
    
    println!("Starting mouse control...\n");

    if let Err(error) = grab(callback) {
        println!("ERROR: {error:?}");
        if cfg!(target_os = "macos") {
            println!("\n💡 Troubleshooting:");
            println!("   1. Go to System Preferences > Security & Privacy > Privacy");
            println!("   2. Select 'Accessibility' from the left panel");
            println!("   3. Add this application to the list");
            println!("   4. Make sure the checkbox is enabled");
        }
    }
}

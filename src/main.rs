use lazy_static::lazy_static;
use rdev::{display_size, grab, simulate, Button, Event, EventType, Key};
use std::{collections::HashMap, thread, time};

#[cfg(target_os = "linux")]
use x11::xlib::{
    Display, Window, XCloseDisplay, XDefaultRootWindow, XOpenDisplay, XQueryTree,
    XFree, XGetWMName, XTextProperty, 
    XGetWindowAttributes, XWindowAttributes, XGetClassHint, XClassHint
};
#[cfg(target_os = "linux")]
use std::ffi::CStr;
#[cfg(target_os = "linux")]
use std::ptr;

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

#[cfg(target_os = "linux")]
mod clickable_detector {
    use super::*;
    use x11::xlib::*;
    use std::ffi::CStr;
    use std::ptr;

    pub fn find_clickable_elements() -> Vec<ClickableElement> {
        let mut elements = Vec::new();
        
        unsafe {
            let display = XOpenDisplay(ptr::null());
            if display.is_null() {
                println!("Failed to open X11 display");
                return elements;
            }
            
            let root = XDefaultRootWindow(display);
            collect_windows_recursive(display, root, &mut elements, 0);
            
            XCloseDisplay(display);
        }
        
        elements
    }

    unsafe fn collect_windows_recursive(
        display: *mut Display,
        window: Window,
        elements: &mut Vec<ClickableElement>,
        depth: i32
    ) {
        if depth > 10 { // Prevent infinite recursion
            return;
        }

        // Get window attributes
        let mut attrs: XWindowAttributes = std::mem::zeroed();
        if XGetWindowAttributes(display, window, &mut attrs) == 0 {
            return;
        }

        // Skip invisible windows
        if attrs.width <= 0 || attrs.height <= 0 || attrs.class == 2 { // InputOnly
            return;
        }

        // Get window name
        let window_name = get_window_name(display, window);
        let class_name = get_window_class(display, window);
        
        // Create clickable element if window has useful properties
        if !window_name.is_empty() || !class_name.is_empty() {
            let display_text = if !window_name.is_empty() {
                if !class_name.is_empty() {
                    format!("{}: {}", class_name, window_name)
                } else {
                    window_name
                }
            } else {
                class_name
            };

            if !display_text.is_empty() && display_text != "vimouse" {
                elements.push(ClickableElement {
                    text: display_text,
                    x: attrs.x as f64,
                    y: attrs.y as f64,
                    width: attrs.width as f64,
                    height: attrs.height as f64,
                    role: "Window".to_string(),
                });
            }
        }

        // Get child windows
        let mut root_return: Window = 0;
        let mut parent_return: Window = 0;
        let mut children: *mut Window = ptr::null_mut();
        let mut nchildren: u32 = 0;

        if XQueryTree(display, window, &mut root_return, &mut parent_return, 
                     &mut children, &mut nchildren) != 0 {
            if !children.is_null() && nchildren > 0 {
                let children_slice = std::slice::from_raw_parts(children, nchildren as usize);
                for &child in children_slice {
                    collect_windows_recursive(display, child, elements, depth + 1);
                }
                XFree(children as *mut _);
            }
        }
    }

    unsafe fn get_window_name(display: *mut Display, window: Window) -> String {
        let mut text_prop: XTextProperty = std::mem::zeroed();
        if XGetWMName(display, window, &mut text_prop) != 0 && !text_prop.value.is_null() {
            let name = CStr::from_ptr(text_prop.value as *const i8);
            let result = name.to_string_lossy().to_string();
            XFree(text_prop.value as *mut _);
            result
        } else {
            String::new()
        }
    }

    unsafe fn get_window_class(display: *mut Display, window: Window) -> String {
        let mut class_hint: XClassHint = std::mem::zeroed();
        if XGetClassHint(display, window, &mut class_hint) != 0 {
            let mut result = String::new();
            if !class_hint.res_class.is_null() {
                let class_name = CStr::from_ptr(class_hint.res_class);
                result = class_name.to_string_lossy().to_string();
                XFree(class_hint.res_class as *mut _);
            }
            if !class_hint.res_name.is_null() {
                XFree(class_hint.res_name as *mut _);
            }
            result
        } else {
            String::new()
        }
    }
}

#[cfg(not(target_os = "linux"))]
mod clickable_detector {
    use super::*;
    
    pub fn find_clickable_elements() -> Vec<ClickableElement> {
        println!("Clickable element detection is only supported on Linux currently");
        Vec::new()
    }
}

fn print_clickable_elements() {
    println!("🔍 Searching for clickable elements on screen...");
    let elements = clickable_detector::find_clickable_elements();
    
    if elements.is_empty() {
        println!("No clickable elements found.");
        return;
    }
    
    println!("Found {} clickable elements:", elements.len());
    println!("{:-<80}", "");
    
    for (i, element) in elements.iter().enumerate() {
        println!("{}. {}", i + 1, element.role);
        if !element.text.is_empty() {
            println!("   Text: \"{}\"", element.text);
        }
        println!("   Location: ({:.0}, {:.0})", element.x, element.y);
        println!("   Size: {:.0}x{:.0}", element.width, element.height);
        println!();
    }
    
    println!("{:-<80}", "");
    println!("Total: {} clickable elements", elements.len());
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
    println!("🐭 Vimouse - Vim-like mouse control");
    println!("Press 'i' to find clickable elements on screen");
    println!("Press 'Esc' to exit");
    println!("Use hjkl for movement, space for click, g+hjkl for scroll");
    
    if let Ok((w, h)) = display_size() {
        unsafe {
            SCREEN_WIDTH = w as f64;
            SCREEN_HEIGHT = h as f64;
            MOUSE_POSITION = (SCREEN_WIDTH / 2., SCREEN_HEIGHT / 2.);
        }
        println!("Screen size: {}x{}", w, h);
    }

    if let Err(error) = grab(callback) {
        println!("ERROR: {error:?}");
        println!("Note: You may need to run this program with appropriate permissions");
        println!("or add it to your accessibility/input monitoring settings.");
    }
}

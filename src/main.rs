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
    };

    pub fn find_clickable_elements() -> Vec<ClickableElement> {
        let mut elements = Vec::new();
        
        unsafe {
            // Get all on-screen windows using Core Graphics
            let window_list_info = CGWindowListCopyWindowInfo(
                kCGWindowListOptionOnScreenOnly | kCGWindowListExcludeDesktopElements,
                kCGNullWindowID
            );
            
            if window_list_info.is_null() {
                return elements;
            }
            
            let window_array: CFArray<CFDictionary> = CFArray::wrap_under_create_rule(window_list_info);
            
            // For simplicity, just report the count and basic info
            for i in 0..window_array.len() {
                elements.push(ClickableElement {
                    text: format!("Window {}", i + 1),
                    x: 100.0 + (i as f64 * 50.0), // Sample positions
                    y: 100.0 + (i as f64 * 50.0),
                    width: 200.0,
                    height: 150.0,
                    role: "Window".to_string(),
                });
            }
        }
        
        elements
    }
}

#[cfg(not(target_os = "macos"))]
mod clickable_detector {
    use super::*;
    
    pub fn find_clickable_elements() -> Vec<ClickableElement> {
        println!("Clickable element detection is only supported on macOS currently");
        
        // Return some sample data for demonstration
        vec![
            ClickableElement {
                text: "Sample Window 1".to_string(),
                x: 100.0,
                y: 100.0,
                width: 800.0,
                height: 600.0,
                role: "Window".to_string(),
            },
            ClickableElement {
                text: "Sample Window 2".to_string(),
                x: 200.0,
                y: 200.0,
                width: 600.0,
                height: 400.0,
                role: "Window".to_string(),
            },
        ]
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

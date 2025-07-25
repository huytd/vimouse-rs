use core_graphics::event::CGEvent;
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use gpui::{
    div, px, rgb, size, App, AppContext, Application, Bounds, Context, FontWeight,
    IntoElement, ParentElement, Pixels, Point, Render, SharedString, Styled, Timer,
    TitlebarOptions, Window, WindowBounds, WindowDecorations, WindowOptions,
};
use lazy_static::lazy_static;
use rdev::{display_size, grab, simulate, Button, Event, EventType, Key};
use std::time::Duration;
use std::{collections::HashMap, thread, time};

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
    use core_foundation::{
        array::{CFArray, CFArrayRef},
        base::{CFTypeRef, TCFType},
        dictionary::{CFDictionary, CFDictionaryRef},
        number::{CFNumber, CFNumberRef},
        string::{CFString, CFStringRef},
    };
    use core_graphics::window::{
        kCGWindowListOptionOnScreenOnly, kCGWindowListExcludeDesktopElements,
        CGWindowListCopyWindowInfo, kCGNullWindowID
    };
    use std::ptr;

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
            
            let window_array = CFArray::wrap_under_create_rule(window_list_info);
            
            for i in 0..window_array.len() {
                if let Some(window_info_ref) = window_array.get(i) {
                    if let Some(window_dict) = CFDictionary::from_void(window_info_ref as *const _) {
                        if let Some(element) = create_clickable_element_from_window(&window_dict) {
                            // Skip our own application window
                            if let Some(owner_name) = get_window_owner_name(&window_dict) {
                                if owner_name.to_lowercase().contains("vimouse") {
                                    continue;
                                }
                            }
                            elements.push(element);
                        }
                    }
                }
            }
        }
        
        elements
    }

    fn create_clickable_element_from_window(window_dict: &CFDictionary) -> Option<ClickableElement> {
        // Get window bounds
        let bounds = get_window_bounds(window_dict)?;
        
        // Get window name/title
        let window_name = get_window_name(window_dict);
        let owner_name = get_window_owner_name(window_dict).unwrap_or("Unknown".to_string());
        
        // Create a display text combining owner and window name
        let display_text = if window_name.is_empty() {
            format!("{} Window", owner_name)
        } else {
            format!("{}: {}", owner_name, window_name)
        };
        
        Some(ClickableElement {
            text: display_text,
            x: bounds.0,
            y: bounds.1,
            width: bounds.2,
            height: bounds.3,
            role: "Window".to_string(),
        })
    }

    fn get_window_bounds(window_dict: &CFDictionary) -> Option<(f64, f64, f64, f64)> {
        // Get kCGWindowBounds
        let bounds_key = CFString::new("kCGWindowBounds");
        if let Some(bounds_ref) = window_dict.find(&bounds_key) {
            if let Some(bounds_dict) = CFDictionary::from_void(bounds_ref as *const _) {
                let x_key = CFString::new("X");
                let y_key = CFString::new("Y");
                let width_key = CFString::new("Width");
                let height_key = CFString::new("Height");
                
                let x = get_number_from_dict(&bounds_dict, &x_key).unwrap_or(0.0);
                let y = get_number_from_dict(&bounds_dict, &y_key).unwrap_or(0.0);
                let width = get_number_from_dict(&bounds_dict, &width_key).unwrap_or(0.0);
                let height = get_number_from_dict(&bounds_dict, &height_key).unwrap_or(0.0);
                
                return Some((x, y, width, height));
            }
        }
        None
    }

    fn get_window_name(window_dict: &CFDictionary) -> String {
        let name_key = CFString::new("kCGWindowName");
        if let Some(name_ref) = window_dict.find(&name_key) {
            if let Some(name_str) = CFString::from_void(name_ref as *const _) {
                return name_str.to_string();
            }
        }
        "".to_string()
    }

    fn get_window_owner_name(window_dict: &CFDictionary) -> Option<String> {
        let owner_key = CFString::new("kCGWindowOwnerName");
        if let Some(owner_ref) = window_dict.find(&owner_key) {
            if let Some(owner_str) = CFString::from_void(owner_ref as *const _) {
                return Some(owner_str.to_string());
            }
        }
        None
    }

    fn get_number_from_dict(dict: &CFDictionary, key: &CFString) -> Option<f64> {
        if let Some(number_ref) = dict.find(key) {
            if let Some(number) = CFNumber::from_void(number_ref as *const _) {
                if let Some(value) = number.to_f64() {
                    return Some(value);
                }
            }
        }
        None
    }
}

#[cfg(not(target_os = "macos"))]
mod clickable_detector {
    use super::*;
    
    pub fn find_clickable_elements() -> Vec<ClickableElement> {
        println!("Clickable element detection is only supported on macOS currently");
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

fn get_current_mouse_position() -> Option<(f64, f64)> {
    // Get the current mouse position using Core Graphics
    let event_source = CGEventSource::new(CGEventSourceStateID::CombinedSessionState).ok()?;
    let event = CGEvent::new(event_source).ok()?;
    let location = event.location();
    Some((location.x, location.y))
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

struct ApplicationUI {
    is_mouse_mode: bool,
    _ticker: gpui::Task<()>,
}

impl ApplicationUI {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let handle = cx.entity().downgrade();

        let task = cx.spawn_in(window, async move |_, cx| loop {
            Timer::after(Duration::from_millis(250)).await;

            let _ = cx.update(|_, cx| {
                if let Some(entity) = handle.upgrade() {
                    entity.update(cx, |app: &mut ApplicationUI, cx| unsafe {
                        app.is_mouse_mode = !G_KEY_HELD;
                        cx.notify();
                    });
                }
            });
        });

        Self {
            is_mouse_mode: true,
            _ticker: task, // store it so it keeps running
        }
    }
}

impl Render for ApplicationUI {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let status_text = if self.is_mouse_mode {
            "Mouse"
        } else {
            "Scroll"
        };

        div()
            .text_sm()
            .font_weight(FontWeight::MEDIUM)
            .text_align(gpui::TextAlign::Center)
            .flex()
            .flex_col()
            .justify_center()
            .items_center()
            .size_full()
            .text_color(rgb(0xffffff))
            .bg(rgb(if self.is_mouse_mode {
                0x10c476
            } else {
                0x7544c9
            }))
            .child(div().shadow_sm().child(format!("{status_text}")))
    }
}

fn main() {
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
    }

    Application::new().run(|cx: &mut App| unsafe {
        let bounds = Bounds::from_corner_and_size(
            gpui::Corner::TopLeft,
            Point::new(
                Pixels(SCREEN_WIDTH as f32 - 90.0),
                Pixels(SCREEN_HEIGHT as f32 - 50.0),
            ),
            size(px(80.), px(32.0)),
        );
        cx.open_window(
            WindowOptions {
                kind: gpui::WindowKind::PopUp,
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                window_decorations: Some(WindowDecorations::Client),
                titlebar: Some(TitlebarOptions {
                    appears_transparent: true,
                    title: Some(SharedString::from("Vimouse")),
                    traffic_light_position: Some(gpui::Point {
                        x: Pixels(-100.0),
                        y: Pixels(-100.0),
                    }),
                }),
                ..Default::default()
            },
            |win, cx| cx.new(|cx| ApplicationUI::new(win, cx)),
        )
        .unwrap();
        cx.activate(true);

        if let Err(error) = grab(callback) {
            println!("ERROR: {error:?}");
        }
    });
}

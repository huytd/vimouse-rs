use std::{cell::OnceCell, collections::HashMap, thread, time};
use lazy_static::lazy_static;
use rdev::{Event, EventType, simulate, Key, display_size, grab, Button};
use gpui::{
    div, px, rgb, size, App, AppContext, Application, Background, Bounds, Context, FontWeight, IntoElement, ParentElement, Pixels, Point, Render, SharedString, Styled, TitlebarOptions, Window, WindowBounds, WindowDecorations, WindowOptions
};
use core_graphics::event::CGEvent;
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

const SLOW_SPEED: f64 = 5.0;
const FAST_SPEED: f64 = 40.0;
const ULTRA_FAST_SPEED: f64 = 150.0;

// Smooth scroll constants
const SCROLL_INITIAL_VELOCITY: f64 = 20.0;
const SCROLL_DECELERATION: f64 = 0.85;  // Momentum decay factor
const SCROLL_MIN_VELOCITY: f64 = 0.5;   // Minimum velocity before stopping
const SCROLL_FRAME_DELAY_MS: u64 = 16;  // ~60 FPS for smooth animation

static mut MOUSE_POSITION: (f64, f64) = (0., 0.);
static mut MOUSE_SPEED: f64 = FAST_SPEED;
static mut G_KEY_HELD: bool = false;

static mut SCREEN_WIDTH: f64 = 0.;
static mut SCREEN_HEIGHT: f64 = 0.;

lazy_static! {
    static ref MOVEMENT_MAP: HashMap<Key, (f64, f64)> = HashMap::from([
        (Key::KeyH, (-1.,  0.)),
        (Key::KeyL, ( 1.,  0.)),
        (Key::KeyJ, ( 0.,  1.)),
        (Key::KeyK, ( 0., -1.)),
        (Key::KeyY, (-1., -1.)),
        (Key::KeyU, ( 1., -1.)),
        (Key::KeyB, (-1.,  1.)),
        (Key::KeyN, ( 1.,  1.)),
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
                let _ = simulate(&EventType::Wheel {
                    delta_x,
                    delta_y
                });
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
            },
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
                    Key::KeyH | Key::KeyL | Key::KeyK | Key::KeyJ | Key::KeyY | Key::KeyU | Key::KeyB | Key::KeyN => {
                        if G_KEY_HELD {
                            // Scroll mode: only handle h, l, j, k for scrolling
                            match key {
                                Key::KeyH => {
                                    // Scroll left with smooth momentum
                                    send_smooth_scroll(-1.0, 0.0);
                                    return None;
                                },
                                Key::KeyL => {
                                    // Scroll right with smooth momentum
                                    send_smooth_scroll(1.0, 0.0);
                                    return None;
                                },
                                Key::KeyJ => {
                                    // Scroll down with smooth momentum
                                    send_smooth_scroll(0.0, -1.0);
                                    return None;
                                },
                                Key::KeyK => {
                                    // Scroll up with smooth momentum
                                    send_smooth_scroll(0.0, 1.0);
                                    return None;
                                },
                                _ => {
                                    // Other movement keys are ignored in scroll mode
                                    return None;
                                }
                            }
                        } else {
                            // Normal movement mode
                            if let Some(direction) = MOVEMENT_MAP.get(&key) {
                                send(&EventType::MouseMove { x: MOUSE_POSITION.0 + direction.0 * MOUSE_SPEED, y: MOUSE_POSITION.1 + direction.1 * MOUSE_SPEED });
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
                    },
                    Key::ControlLeft | Key::ControlRight | Key::CapsLock => {
                        send(&EventType::ButtonPress(Button::Right));
                        return None;
                    },
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
                    Key::KeyQ | Key::KeyW | Key::KeyE | Key::KeyR | Key::KeyA | Key::KeyS | Key::KeyD | Key::KeyF | Key::KeyZ | Key::KeyX | Key::KeyC | Key::KeyV => {
                        if let Some((col, row)) = SCREEN_CELL_MAP.get(&key) {
                            let (x, y) = (
                                col * SCREEN_WIDTH / 4. + SCREEN_WIDTH / 8.,
                                row * SCREEN_HEIGHT / 3. + SCREEN_HEIGHT / 6.
                            );
                            send(&EventType::MouseMove { x, y });
                            return None;
                        }
                        return Some(event);
                    },
                    /* Others:
                     * - Esc: Exit
                     * - Shift: Slow speed
                     * - Alt: Fast speed
                     */
                    Key::Escape => {
                        std::process::exit(0);
                    },
                    Key::ShiftLeft | Key::ShiftRight => {
                        MOUSE_SPEED = SLOW_SPEED;
                        return Some(event);
                    },
                    Key::Alt => {
                        MOUSE_SPEED = ULTRA_FAST_SPEED;
                        return Some(event);
                    },
                    Key::KeyG => {
                        G_KEY_HELD = true;
                        return None;
                    }
                    Key::KeyT => {
                        G_KEY_HELD = !G_KEY_HELD;
                        return None;
                    }
                    _ => Some(event)
                }
            },
            EventType::KeyRelease(key) => {
                return match key {
                    Key::Space => {
                        send(&EventType::ButtonRelease(Button::Left));
                        return None;
                    },
                    Key::ControlLeft | Key::ControlRight | Key::CapsLock => {
                        send(&EventType::ButtonRelease(Button::Right));
                        return None;
                    },
                    Key::ShiftLeft | Key::ShiftRight | Key::Alt => {
                        MOUSE_SPEED = FAST_SPEED;
                        return Some(event);
                    },
                    Key::KeyG => {
                        G_KEY_HELD = false;
                        return None;
                    },
                    _ => Some(event)
                }
            }
            _ => Some(event)
        };
    }
}

struct ApplicationUI;

impl Render for ApplicationUI {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let status_text = unsafe {
            if G_KEY_HELD {
                "Scroll"
            } else {
                "Mouse"
            }
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
            .text_color(rgb(0x03361c))
            .bg(rgb(0x03fc7f))
            .child(format!("{status_text}"))
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
            Point::new(Pixels(SCREEN_WIDTH as f32 - 90.0), Pixels(SCREEN_HEIGHT as f32 - 50.0)),
            size(px(80.), px(32.0))
        );
        cx.open_window(
            WindowOptions {
                kind: gpui::WindowKind::PopUp,
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                window_decorations: Some(WindowDecorations::Client),
                titlebar: Some(TitlebarOptions {
                    appears_transparent: true,
                    title: Some(SharedString::from("Vimouse")),
                    traffic_light_position: Some(gpui::Point { x: Pixels(-100.0), y: Pixels(-100.0) })
                }),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|_| ApplicationUI {})
            },
        )
            .unwrap();
        cx.activate(true);

        if let Err(error) = grab(callback) {
            println!("ERROR: {error:?}");
        }
    });
}

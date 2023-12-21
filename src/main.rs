use std::{time, thread};
use rdev::{Event, EventType, simulate, Key, display_size, grab, Button};

const SLOW_SPEED: f64 = 5.0;
const FAST_SPEED: f64 = 40.0;
const ULTRA_FAST_SPEED: f64 = 150.0;

static mut MOUSE_POSITION: (f64, f64) = (0., 0.);
static mut MOUSE_SPEED: f64 = FAST_SPEED;

static mut SCREEN_WIDTH: f64 = 0.;
static mut SCREEN_HEIGHT: f64 = 0.;

fn main() {
    if let Ok((w, h)) = display_size() {
        unsafe {
            SCREEN_WIDTH = w as f64;
            SCREEN_HEIGHT = h as f64;
            MOUSE_POSITION = (SCREEN_WIDTH / 2., SCREEN_HEIGHT / 2.);
        }
    }

    if let Err(error) = grab(callback) {
        println!("ERROR: {error:?}");
    }
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
                    Key::KeyH => {
                        send(&EventType::MouseMove { x: MOUSE_POSITION.0 - MOUSE_SPEED, y: MOUSE_POSITION.1 });
                        return None;
                    },
                    Key::KeyL => {
                        send(&EventType::MouseMove { x: MOUSE_POSITION.0 + MOUSE_SPEED, y: MOUSE_POSITION.1 });
                        return None;
                    },
                    Key::KeyK => {
                        send(&EventType::MouseMove { x: MOUSE_POSITION.0, y: MOUSE_POSITION.1 - MOUSE_SPEED });
                        return None;
                    },
                    Key::KeyJ => {
                        send(&EventType::MouseMove { x: MOUSE_POSITION.0, y: MOUSE_POSITION.1 + MOUSE_SPEED });
                        return None;
                    },
                    Key::KeyY => {
                        send(&EventType::MouseMove { x: MOUSE_POSITION.0 - MOUSE_SPEED, y: MOUSE_POSITION.1 - MOUSE_SPEED });
                        return None;
                    },
                    Key::KeyU => {
                        send(&EventType::MouseMove { x: MOUSE_POSITION.0 + MOUSE_SPEED, y: MOUSE_POSITION.1 - MOUSE_SPEED });
                        return None;
                    },
                    Key::KeyB => {
                        send(&EventType::MouseMove { x: MOUSE_POSITION.0 - MOUSE_SPEED, y: MOUSE_POSITION.1 + MOUSE_SPEED });
                        return None;
                    },
                    Key::KeyN => {
                        send(&EventType::MouseMove { x: MOUSE_POSITION.0 + MOUSE_SPEED, y: MOUSE_POSITION.1 + MOUSE_SPEED });
                        return None;
                    },
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
                     *  ┌─────┬─────┬─────┐
                     *  │  Q  │  W  │  E  │
                     *  ├─────┼─────┼─────┤
                     *  │  A  │  S  │  D  │
                     *  ├─────┼─────┼─────┤
                     *  │  Z  │  X  │  C  │
                     *  └─────┴─────┴─────┘
                     */
                    Key::KeyQ => {
                        let (x, y) = (
                            0. * SCREEN_WIDTH / 3. + SCREEN_WIDTH / 6.,
                            0. * SCREEN_HEIGHT / 3. + SCREEN_HEIGHT / 6.
                        );
                        send(&EventType::MouseMove { x, y });
                        return None;
                    },
                    Key::KeyW => {
                        let (x, y) = (
                            1. * SCREEN_WIDTH / 3. + SCREEN_WIDTH / 6.,
                            0. * SCREEN_HEIGHT / 3. + SCREEN_HEIGHT / 6.
                        );
                        send(&EventType::MouseMove { x, y });
                        return None;
                    }
                    Key::KeyE => {
                        let (x, y) = (
                            2. * SCREEN_WIDTH / 3. + SCREEN_WIDTH / 6.,
                            0. * SCREEN_HEIGHT / 3. + SCREEN_HEIGHT / 6.
                        );
                        send(&EventType::MouseMove { x, y });
                        return None;
                    },
                    Key::KeyA => {
                        let (x, y) = (
                            0. * SCREEN_WIDTH / 3. + SCREEN_WIDTH / 6.,
                            1. * SCREEN_HEIGHT / 3. + SCREEN_HEIGHT / 6.
                        );
                        send(&EventType::MouseMove { x, y });
                        return None;
                    },
                    Key::KeyS => {
                        let (x, y) = (
                            1. * SCREEN_WIDTH / 3. + SCREEN_WIDTH / 6.,
                            1. * SCREEN_HEIGHT / 3. + SCREEN_HEIGHT / 6.
                        );
                        send(&EventType::MouseMove { x, y });
                        return None;
                    }
                    Key::KeyD => {
                        let (x, y) = (
                            2. * SCREEN_WIDTH / 3. + SCREEN_WIDTH / 6.,
                            1. * SCREEN_HEIGHT / 3. + SCREEN_HEIGHT / 6.
                        );
                        send(&EventType::MouseMove { x, y });
                        return None;
                    },
                    Key::KeyZ => {
                        let (x, y) = (
                            0. * SCREEN_WIDTH / 3. + SCREEN_WIDTH / 6.,
                            2. * SCREEN_HEIGHT / 3. + SCREEN_HEIGHT / 6.
                        );
                        send(&EventType::MouseMove { x, y });
                        return None;
                    },
                    Key::KeyX => {
                        let (x, y) = (
                            1. * SCREEN_WIDTH / 3. + SCREEN_WIDTH / 6.,
                            2. * SCREEN_HEIGHT / 3. + SCREEN_HEIGHT / 6.
                        );
                        send(&EventType::MouseMove { x, y });
                        return None;
                    }
                    Key::KeyC => {
                        let (x, y) = (
                            2. * SCREEN_WIDTH / 3. + SCREEN_WIDTH / 6.,
                            2. * SCREEN_HEIGHT / 3. + SCREEN_HEIGHT / 6.
                        );
                        send(&EventType::MouseMove { x, y });
                        return None;
                    }
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
                    _ => Some(event)
                }
            }
            _ => Some(event)
        };
    }
}

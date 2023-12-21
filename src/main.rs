
use std::{time, thread, collections::HashMap};
use lazy_static::lazy_static;
use rdev::{Event, EventType, simulate, Key, display_size, grab, Button};

const SLOW_SPEED: f64 = 5.0;
const FAST_SPEED: f64 = 40.0;
const ULTRA_FAST_SPEED: f64 = 150.0;

static mut MOUSE_POSITION: (f64, f64) = (0., 0.);
static mut MOUSE_SPEED: f64 = FAST_SPEED;

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
       (Key::KeyA, (0., 1.)),
       (Key::KeyS, (1., 1.)),
       (Key::KeyD, (2., 1.)),
       (Key::KeyZ, (0., 2.)),
       (Key::KeyX, (1., 2.)),
       (Key::KeyC, (2., 2.)),
    ]);
}

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
                    Key::KeyH | Key::KeyL | Key::KeyK | Key::KeyJ | Key::KeyY | Key::KeyU | Key::KeyB | Key::KeyN => {
                        if let Some(direction) = MOVEMENT_MAP.get(&key) {
                            send(&EventType::MouseMove { x: MOUSE_POSITION.0 + direction.0 * MOUSE_SPEED, y: MOUSE_POSITION.1 + direction.1 * MOUSE_SPEED });
                            return None;
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
                     *  ┌─────┬─────┬─────┐
                     *  │  Q  │  W  │  E  │
                     *  ├─────┼─────┼─────┤
                     *  │  A  │  S  │  D  │
                     *  ├─────┼─────┼─────┤
                     *  │  Z  │  X  │  C  │
                     *  └─────┴─────┴─────┘
                     */
                    Key::KeyQ | Key::KeyW | Key::KeyE | Key::KeyA | Key::KeyS | Key::KeyD | Key::KeyZ | Key::KeyX | Key::KeyC => {
                        if let Some((col, row)) = SCREEN_CELL_MAP.get(&key) {
                            let (x, y) = (
                                col * SCREEN_WIDTH / 3. + SCREEN_WIDTH / 6.,
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


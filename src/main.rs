use input_event_codes::*;
use libc;
use std::io::{self, Read, Write};

const EVENT_SIZE: usize = core::mem::size_of::<libc::input_event>();

fn main() {
    let mut buf: [u8; EVENT_SIZE] = [0; EVENT_SIZE];

    let mut engine = Engine::new(send_event);

    loop {
        io::stdin()
            .read_exact(&mut buf)
            .expect("Could not read input event");

        let evt = unsafe { std::mem::transmute::<[u8; EVENT_SIZE], libc::input_event>(buf) };
        if evt.type_ != EV_KEY!() {
            write_buffer(&buf);
            continue;
        }

        let mut evt = KeyEvent::new(&evt);
        eprintln!("{:?}", evt);

        engine.handle(&mut evt);
        if evt.bubble {
            write_buffer(&buf);
        }
    }
}

fn send_event(event: KeyEvent) {
    let input_event = libc::input_event {
        time: libc::timeval {
            tv_sec: 0,
            tv_usec: 0,
        },
        type_: EV_KEY!(),
        code: event.code,
        value: event.value.as_i32(),
    };
    let buf = unsafe { std::mem::transmute::<libc::input_event, [u8; EVENT_SIZE]>(input_event) };
    write_buffer(&buf);
}

fn write_buffer(buf: &[u8; EVENT_SIZE]) {
    io::stdout().write_all(buf).unwrap();
    io::stdout().flush().unwrap();
}

struct Engine {
    l_ctrl: KeyState,
    capslock: KeyState,
    send_event: fn(KeyEvent),
}

impl Engine {
    pub fn new(send_event: fn(KeyEvent)) -> Engine {
        Engine {
            l_ctrl: KeyState::Released,
            capslock: KeyState::Released,
            send_event,
        }
    }

    fn handle(&mut self, evt: &mut KeyEvent) {
        self.update_modifiers(evt);

        if matches!(self.capslock, KeyState::Pressed) && evt.code == KEY_L!() {
            let new_event = KeyEvent {
                code: KEY_RIGHT!(),
                value: evt.value.clone(),
                bubble: true,
            };
            (self.send_event)(new_event);
            evt.bubble = false;
        } else if matches!(self.capslock, KeyState::Pressed) && evt.code == KEY_J!() {
            let new_event = KeyEvent {
                code: KEY_LEFT!(),
                value: evt.value.clone(),
                bubble: true,
            };
            (self.send_event)(new_event);
            evt.bubble = false;
        } else if matches!(self.capslock, KeyState::Pressed) && evt.code == KEY_I!() {
            let new_event = KeyEvent {
                code: KEY_UP!(),
                value: evt.value.clone(),
                bubble: true,
            };
            (self.send_event)(new_event);
            evt.bubble = false;
        } else if matches!(self.capslock, KeyState::Pressed) && evt.code == KEY_K!() {
            let new_event = KeyEvent {
                code: KEY_DOWN!(),
                value: evt.value.clone(),
                bubble: true,
            };
            (self.send_event)(new_event);
            evt.bubble = false;
        }
    }

    pub fn update_modifiers(&mut self, event: &mut KeyEvent) {
        match event.code {
            KEY_LEFTCTRL!() => {
                match event.value {
                    KeyValue::Release => self.l_ctrl = KeyState::Released,
                    _ => self.l_ctrl = KeyState::Pressed,
                };
                event.bubble = true;
            }
            KEY_CAPSLOCK!() => {
                match event.value {
                    KeyValue::Release => self.capslock = KeyState::Released,
                    _ => self.capslock = KeyState::Pressed,
                };
                event.bubble = false
            }
            _ => event.bubble = true,
        }
    }
}

#[derive(Debug)]
enum KeyState {
    Pressed,
    Released,
}

#[derive(Debug)]
struct KeyEvent {
    code: u16,
    value: KeyValue,
    bubble: bool,
}

impl KeyEvent {
    pub fn new(input_event: &libc::input_event) -> KeyEvent {
        if input_event.type_ != EV_KEY!() {
            panic!("Constructing KeyEvent from wrong event type");
        }

        KeyEvent {
            code: input_event.code,
            value: KeyValue::from(input_event.value),
            bubble: true,
        }
    }
}

#[derive(Debug, Clone)]
enum KeyValue {
    Press,
    Repeat,
    Release,
}

impl KeyValue {
    pub fn from(value: i32) -> KeyValue {
        match value {
            0 => KeyValue::Release,
            1 => KeyValue::Press,
            2 => KeyValue::Repeat,
            _ => panic!("Invalid key value"),
        }
    }

    pub fn as_i32(&self) -> i32 {
        match self {
            KeyValue::Release => 0,
            KeyValue::Press => 1,
            KeyValue::Repeat => 2,
        }
    }
}

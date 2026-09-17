use std::io::{self, Cursor, Read, Write};

use input_event_codes::{EV_KEY, EV_SYN, SYN_REPORT};

const KEY_RELEASE: i32 = 0;
const KEY_PRESS: i32 = 1;
const KEY_REPEAT: i32 = 2;

pub fn create_event_streams(input_events: &[InputEvent]) -> (InputBuffer, OutputBuffer) {
    let mut input = InputBuffer::default();
    for event in input_events {
        input.add_event(event);
    }

    (input, OutputBuffer::default())
}

#[derive(Default)]
pub struct InputBuffer(Cursor<Vec<u8>>);

impl InputBuffer {
    pub fn add_event(&mut self, event: &InputEvent) {
        let raw = unsafe { std::mem::transmute::<libc::input_event, [u8; 24]>(event.0) };
        self.0.get_mut().extend_from_slice(&raw);
    }
}

impl Read for InputBuffer {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

#[derive(Default)]
pub struct OutputBuffer(Cursor<Vec<u8>>);

impl OutputBuffer {
    pub fn extract_events(&mut self) -> Vec<InputEvent> {
        let mut res = Vec::new();
        let mut buf = [0 as u8; 24];
        // position was set when writing, start reading from beginning
        self.0.set_position(0);
        while self.0.read_exact(&mut buf).is_ok() {
            let event = unsafe { std::mem::transmute::<[u8; 24], libc::input_event>(buf) };
            res.push(InputEvent(event));
        }
        // clear underlying data after reading, so subsequent calls to this function only return newly written events
        self.0.get_mut().clear();

        res
    }
}

impl Write for OutputBuffer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

#[derive(Debug)]
pub struct InputEvent(libc::input_event);

impl InputEvent {
    pub fn key_press(code: u16) -> Self {
        Self(libc::input_event {
            time: libc::timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
            type_: EV_KEY!(),
            code,
            value: KEY_PRESS,
        })
    }
    pub fn key_repeat(code: u16) -> Self {
        Self(libc::input_event {
            time: libc::timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
            type_: EV_KEY!(),
            code,
            value: KEY_REPEAT,
        })
    }
    pub fn key_release(code: u16) -> Self {
        Self(libc::input_event {
            time: libc::timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
            type_: EV_KEY!(),
            code,
            value: KEY_RELEASE,
        })
    }
    pub fn syn_report() -> Self {
        Self(libc::input_event {
            time: libc::timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
            type_: EV_SYN!(),
            code: SYN_REPORT!(),
            value: 0,
        })
    }
}

impl PartialEq for InputEvent {
    fn eq(&self, other: &Self) -> bool {
        self.0.time.tv_sec == other.0.time.tv_sec
            && self.0.time.tv_usec == other.0.time.tv_usec
            && self.0.code == other.0.code
            && self.0.type_ == other.0.type_
            // value is not specified for syn_report events
            && (self.0.value == other.0.value
                || self.0.type_ == EV_SYN!() && self.0.code == SYN_REPORT!())
    }
}

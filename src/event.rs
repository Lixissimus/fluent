use std::array::TryFromSliceError;

use input_event_codes::EV_KEY;

use crate::keys::Key;

const LIBC_INPUT_EVENT_SIZE: usize = 24;

#[derive(Default, Debug)]
pub struct EventBuffer([u8; LIBC_INPUT_EVENT_SIZE]);

impl EventBuffer {
    pub fn raw(&self) -> &[u8; core::mem::size_of::<Self>()] {
        &self.0
    }

    pub fn raw_mut(&mut self) -> &mut [u8; core::mem::size_of::<Self>()] {
        &mut self.0
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct InputEvent {
    pub r#type: EventType,
    pub code: Key,
    pub value: KeyValue,
}

impl InputEvent {
    pub fn key_press(key: Key) -> Self {
        Self {
            r#type: EventType::Key,
            code: key,
            value: KeyValue::Press,
        }
    }

    pub fn key_repeat(key: Key) -> Self {
        Self {
            r#type: EventType::Key,
            code: key,
            value: KeyValue::Repeat,
        }
    }

    pub fn key_release(key: Key) -> Self {
        Self {
            r#type: EventType::Key,
            code: key,
            value: KeyValue::Release,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum EventType {
    Key,
    Other(u16),
}

impl From<u16> for EventType {
    fn from(value: u16) -> Self {
        if value == EV_KEY!() {
            return Self::Key;
        }
        Self::Other(value)
    }
}

impl From<EventType> for u16 {
    fn from(value: EventType) -> Self {
        match value {
            EventType::Key => EV_KEY!(),
            EventType::Other(value) => value,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum KeyValue {
    Release,
    Press,
    Repeat,
    Other(i32),
}

impl From<i32> for KeyValue {
    fn from(value: i32) -> Self {
        match value {
            0 => KeyValue::Release,
            1 => KeyValue::Press,
            2 => KeyValue::Repeat,
            value => KeyValue::Other(value),
        }
    }
}

impl From<KeyValue> for i32 {
    fn from(value: KeyValue) -> Self {
        match value {
            KeyValue::Release => 0,
            KeyValue::Press => 1,
            KeyValue::Repeat => 2,
            KeyValue::Other(value) => value,
        }
    }
}

impl TryFrom<&EventBuffer> for InputEvent {
    type Error = TryFromSliceError;

    fn try_from(buffer: &EventBuffer) -> Result<Self, Self::Error> {
        Ok(InputEvent {
            // TODO: first 16 bits contain timestamp, add to InputEvent
            r#type: u16::from_ne_bytes(buffer.0[16..18].try_into()?).into(),
            code: u16::from_ne_bytes(buffer.0[18..20].try_into()?).into(),
            value: i32::from_ne_bytes(buffer.0[20..24].try_into()?).into(),
        })
    }
}

impl From<&InputEvent> for EventBuffer {
    fn from(event: &InputEvent) -> Self {
        let mut buffer = EventBuffer::default();
        // TODO: add timestamp to first 2 bytes
        buffer.0[0..16].copy_from_slice(&[0; 16]);
        let event_type: u16 = event.r#type.into();
        let event_code: u16 = event.code.into();
        let event_value: i32 = event.value.into();
        buffer.0[16..18].copy_from_slice(&event_type.to_ne_bytes());
        buffer.0[18..20].copy_from_slice(&event_code.to_ne_bytes());
        buffer.0[20..24].copy_from_slice(&event_value.to_ne_bytes());
        buffer
    }
}

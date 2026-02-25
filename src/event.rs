use std::array::TryFromSliceError;

use anyhow::anyhow;
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

#[derive(Debug, Default, Hash, PartialEq, Eq, Clone)]
pub struct Modifiers {
    pub ctrl_left: bool,
    pub ctrl_right: bool,
    pub alt_left: bool,
    pub alt_right: bool,
    pub shift_left: bool,
    pub shift_right: bool,
    pub capslock: bool,
}

impl Modifiers {
    pub fn is_modifier(event: &InputEvent) -> bool {
        match event.code {
            Key::CtrlLeft
            | Key::CtrlRight
            | Key::AltLeft
            | Key::AltRight
            | Key::ShiftLeft
            | Key::ShiftRight
            | Key::Capslock => true,
            _ => false,
        }
    }

    pub fn update_from(&mut self, event: &InputEvent) {
        let is_pressed = match event.value {
            KeyValue::Press | KeyValue::Repeat => true,
            _ => false,
        };
        match event.code {
            Key::CtrlLeft => self.ctrl_left = is_pressed,
            Key::CtrlRight => self.ctrl_right = is_pressed,
            Key::AltLeft => self.alt_left = is_pressed,
            Key::AltRight => self.alt_right = is_pressed,
            Key::ShiftLeft => self.shift_left = is_pressed,
            Key::ShiftRight => self.shift_right = is_pressed,
            Key::Capslock => self.capslock = is_pressed,
            _ => (),
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Combination {
    pub modifiers: Modifiers,
    pub key: Key,
}

impl TryFrom<&Vec<Key>> for Combination {
    // TODO: custom errors?
    type Error = anyhow::Error;

    fn try_from(keys: &Vec<Key>) -> Result<Self, Self::Error> {
        let mut modifiers = Modifiers::default();
        // destructure to ensure update on new modifiers
        let Modifiers {
            ctrl_left,
            ctrl_right,
            alt_left,
            alt_right,
            shift_left,
            shift_right,
            capslock,
        } = &mut modifiers;
        let mut trigger = Option::None;

        for key in keys {
            match key {
                Key::CtrlLeft => *ctrl_left = true,
                Key::CtrlRight => *ctrl_right = true,
                Key::AltLeft => *alt_left = true,
                Key::AltRight => *alt_right = true,
                Key::ShiftLeft => *shift_left = true,
                Key::ShiftRight => *shift_right = true,
                Key::Capslock => *capslock = true,
                key => {
                    trigger = {
                        if trigger.is_some() {
                            return Err(anyhow!(
                                "multiple non-modifier keys found in key sequence {:?}",
                                keys
                            ));
                        }
                        Some(*key)
                    }
                }
            }
        }

        let Some(trigger) = trigger else {
            return Err(anyhow!(
                "no non-modifier key found in key sequence {:?}",
                keys
            ));
        };

        Ok(Self {
            modifiers,
            key: trigger,
        })
    }
}

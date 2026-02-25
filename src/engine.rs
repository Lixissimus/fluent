use std::collections::HashMap;

use crate::{
    config::Config,
    event::{Combination, EventType, InputEvent, KeyValue, Modifiers},
    keys::Key,
};

#[derive(Default)]
pub struct Engine {
    modifier_state: Modifiers,
    mappings: HashMap<Combination, Vec<Key>>,
}

impl Engine {
    pub fn new(config: &Config) -> anyhow::Result<Self> {
        let mut mappings = HashMap::new();
        for mapping in &config.mappings {
            mappings.insert((&mapping.on).try_into()?, mapping.send.clone());
        }
        Ok(Self {
            modifier_state: Modifiers::default(),
            mappings,
        })
    }

    pub fn handle(&mut self, event: InputEvent) -> Vec<InputEvent> {
        let mut output_events = Vec::new();
        if Modifiers::is_modifier(&event) {
            self.modifier_state.update_from(&event);
            // consume capslock, forward all other modifiers
            if event.code != Key::Capslock {
                output_events.push(event);
            }
        } else {
            let active_combination = Combination {
                modifiers: self.modifier_state.clone(),
                key: event.code,
            };
            let Some(keys) = self.mappings.get(&active_combination) else {
                output_events.push(event);
                return output_events;
            };
            let evts = keys.iter().map(|key| InputEvent {
                code: *key,
                ..event
            });
            output_events.append(&mut self.release_modifiers());
            output_events.extend(evts);
        }

        output_events
    }

    fn release_modifiers(&self) -> Vec<InputEvent> {
        let mut events = Vec::new();

        // destructure to ensure this function is updated on new modifiers
        let Modifiers {
            ctrl_left,
            ctrl_right,
            alt_left,
            alt_right,
            shift_left,
            shift_right,
            // no need to release capslock, it is not forwarded anyways
            capslock: _,
        } = self.modifier_state;

        // I'm not sure if it is a good idea to blindly release all modifiers that the system assumes are pressed.
        // This way, we release them multiple times. However, until this creates a problem, let's leave it like that
        // in order to not have to track which were already released.
        // This e.g. conflicts in VS Code the alt focusing on the menu bar. Alternatively, we have to consume all
        // modifiers until we have a non-modifier event. If it is a hotkey, we just send the hotkey. If not, we forward
        // the modifiers and the non-modifier.

        if ctrl_left {
            events.push(InputEvent {
                r#type: EventType::Key,
                code: Key::CtrlLeft,
                value: KeyValue::Release,
            });
        }
        if ctrl_right {
            events.push(InputEvent {
                r#type: EventType::Key,
                code: Key::CtrlRight,
                value: KeyValue::Release,
            });
        }
        if alt_left {
            events.push(InputEvent {
                r#type: EventType::Key,
                code: Key::AltLeft,
                value: KeyValue::Release,
            });
        }
        if alt_right {
            events.push(InputEvent {
                r#type: EventType::Key,
                code: Key::AltRight,
                value: KeyValue::Release,
            });
        }
        if shift_left {
            events.push(InputEvent {
                r#type: EventType::Key,
                code: Key::ShiftLeft,
                value: KeyValue::Release,
            });
        }
        if shift_right {
            events.push(InputEvent {
                r#type: EventType::Key,
                code: Key::ShiftRight,
                value: KeyValue::Release,
            });
        }

        events
    }
}

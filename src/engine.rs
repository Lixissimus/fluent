use std::collections::HashMap;

use crate::{
    event::{Combination, InputEvent, Modifiers},
    keys::Key,
};

#[derive(Default)]
pub struct Engine {
    modifier_state: Modifiers,
    mappings: HashMap<Combination, Vec<Key>>,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            mappings: HashMap::from([
                (
                    Combination {
                        modifiers: Modifiers {
                            capslock: true,
                            ..Default::default()
                        },
                        key: Key::J,
                    },
                    vec![Key::ArrowLeft],
                ),
                (
                    Combination {
                        modifiers: Modifiers {
                            capslock: true,
                            alt_left: true,
                            ..Default::default()
                        },
                        key: Key::J,
                    },
                    vec![Key::ShiftLeft, Key::ArrowLeft],
                ),
                (
                    Combination {
                        modifiers: Modifiers {
                            capslock: true,
                            ..Default::default()
                        },
                        key: Key::K,
                    },
                    vec![Key::ArrowDown],
                ),
                (
                    Combination {
                        modifiers: Modifiers {
                            capslock: true,
                            ..Default::default()
                        },
                        key: Key::L,
                    },
                    vec![Key::ArrowRight],
                ),
                (
                    Combination {
                        modifiers: Modifiers {
                            capslock: true,
                            ..Default::default()
                        },
                        key: Key::I,
                    },
                    vec![Key::ArrowUp],
                ),
                (
                    Combination {
                        modifiers: Modifiers {
                            capslock: true,
                            ..Default::default()
                        },
                        key: Key::U,
                    },
                    vec![Key::Home],
                ),
                (
                    Combination {
                        modifiers: Modifiers {
                            capslock: true,
                            ..Default::default()
                        },
                        key: Key::O,
                    },
                    vec![Key::End],
                ),
                (
                    Combination {
                        modifiers: Modifiers {
                            capslock: true,
                            ..Default::default()
                        },
                        key: Key::H,
                    },
                    vec![Key::CtrlLeft, Key::ArrowLeft],
                ),
                (
                    Combination {
                        modifiers: Modifiers {
                            capslock: true,
                            ..Default::default()
                        },
                        key: Key::Semicolon,
                    },
                    vec![Key::CtrlLeft, Key::ArrowRight],
                ),
            ]),
            ..Default::default()
        }
    }

    pub fn handle(&mut self, event: InputEvent) -> Vec<InputEvent> {
        let mut output_events = Vec::new();
        if Modifiers::is_modifier(&event) {
            // TODO: improve modifier handling, we cannot always bubble them, see caps+alt+j
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
            let mut evts: Vec<InputEvent> = keys
                .iter()
                .map(|key| InputEvent {
                    code: *key,
                    ..event
                })
                .collect();
            output_events.append(&mut evts);
        }

        output_events
    }
}

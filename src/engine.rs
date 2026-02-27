use std::collections::{BTreeSet, HashMap};

use crate::{
    config::Config,
    event::{InputEvent, KeyValue},
    keys::Key,
};

pub struct Engine {
    hotkeys: Hotkeys,
    state: State,
}

#[derive(Clone)]
enum State {
    /// `Idle` state is the state where all the keys that are in the state's vector have been forwarded as key presses.
    /// The state's vector can be considered a list of pending release events.
    /// Whenever we leave `Idle`, we must ensure that we send matching key release events for all key press events
    /// that were forwarded.
    /// This also means that when we enter `Idle` with a non-empty list of keys, we must send key presses for
    /// all those events.
    /// Whatever key event happens while we stay in idle must be forwarded. This is in contrast to all other states
    /// where we consume all key events and not forward them directly. We only forward them once they bring us back into
    /// `Idle`.
    Idle(Vec<Key>),
    /// `PartialHotkey` state is the state where we have a sequence of events that _could_ become a hotkey match.
    /// The sequence of keys is stored in the state's vector. Incoming key events modify that sequence until it eiter
    /// results in a complete hotkey match and a transition into `CompleteHotkey` or a match becomes impossible and it
    /// leads back to `Idle`.
    PartialHotkey(Vec<Key>),
    /// `CompleteHotkey` state is the state where we had a matching hotkey pressed. When we enter this state, we send a
    /// key press for the configured hotkey. When a key repeat is sent in this state, it send a matching key repeat of
    /// the hotkeys non-modifier key. Key press events are ignored in this state. A key release event results in key
    /// release events of the configured hotkey and transitions either to `ReleasingHotkey` when there are still keys
    /// pressed, or to `Idle`. `pressed` contains the sequence of keys that activated the hotkey, `triggered` contains
    /// the key sequence that was triggered by the hotkey.
    CompleteHotkey {
        pressed: Vec<Key>,
        triggered: Vec<Key>,
    },
    /// `ReleasingHotkey` state is the state where we enter once a hotkey was released but there are still some keys of
    /// it pressed. If a new key press comes it that results in a new hotkey match, we transition back to
    /// `CompleteHotkey`. If a new key press comes in that _could_ become a hotkey match, we transition to
    /// `PartialHotkey`. If a new key press comes in that makes a match impossible, we got to `Idle`, sending key press
    /// events for all pressed events.
    /// Key releases will make us stay here, until no keys are pressed anymore. Then we transition back to `Idle`.
    ReleasingHotkey(Vec<Key>),
}

impl Engine {
    pub fn new(config: &Config) -> anyhow::Result<Self> {
        Ok(Self {
            hotkeys: Hotkeys::new(config)?,
            state: State::Idle(Vec::new()),
        })
    }

    pub fn handle(&mut self, event: InputEvent) -> Vec<InputEvent> {
        let (new_state, send_event) = match (&self.state, event.value) {
            (State::Idle(pressed), KeyValue::Press) => {
                let mut now_pressed = pressed.clone();
                now_pressed.push(event.code);
                match self.hotkeys.query(&KeySet::from_iter(now_pressed.clone())) {
                    Match::Impossible => (
                        State::Idle(now_pressed.clone()),
                        key_press_sequence(&now_pressed),
                    ),
                    Match::Possible => (
                        State::PartialHotkey(now_pressed),
                        key_release_sequence(&pressed),
                    ),
                    Match::Complete(trigger_keys) => {
                        let mut send_keys = key_release_sequence(&pressed);
                        send_keys.extend(key_press_sequence(&trigger_keys));
                        (
                            State::CompleteHotkey {
                                pressed: now_pressed,
                                triggered: trigger_keys.clone(),
                            },
                            send_keys,
                        )
                    }
                }
            }
            (State::PartialHotkey(pressed), KeyValue::Press)
            | (State::ReleasingHotkey(pressed), KeyValue::Press) => {
                let mut now_pressed = pressed.clone();
                now_pressed.push(event.code);
                match self.hotkeys.query(&KeySet::from_iter(now_pressed.clone())) {
                    Match::Impossible => (
                        State::Idle(now_pressed.clone()),
                        key_press_sequence(&now_pressed),
                    ),
                    Match::Possible => (State::PartialHotkey(now_pressed), Vec::new()),
                    Match::Complete(trigger_keys) => (
                        State::CompleteHotkey {
                            pressed: now_pressed,
                            triggered: trigger_keys.clone(),
                        },
                        key_press_sequence(&trigger_keys),
                    ),
                }
            }
            (
                State::CompleteHotkey {
                    pressed: trigger,
                    triggered: send,
                },
                KeyValue::Repeat,
            ) => (
                State::CompleteHotkey {
                    pressed: trigger.clone(),
                    triggered: send.clone(),
                },
                key_repeat_sequence(&send),
            ),
            (
                State::CompleteHotkey {
                    pressed: trigger,
                    triggered: send,
                },
                KeyValue::Release,
            ) => {
                let remaining_keys: Vec<Key> = trigger
                    .iter()
                    .filter(|key| key != &&event.code)
                    .cloned()
                    .collect();

                let send_keys = key_release_sequence(&send);

                if remaining_keys.is_empty() {
                    (State::Idle(Vec::new()), send_keys)
                } else {
                    (State::ReleasingHotkey(remaining_keys), send_keys)
                }
            }
            (State::PartialHotkey(pressed), KeyValue::Release) => {
                let remaining_keys: Vec<Key> = pressed
                    .iter()
                    .filter(|key| key != &&event.code)
                    .cloned()
                    .collect();

                if remaining_keys.is_empty() {
                    (State::Idle(Vec::new()), Vec::new())
                } else {
                    (State::PartialHotkey(remaining_keys), Vec::new())
                }
            }
            (State::Idle(pressed), KeyValue::Release) => {
                let remaining_keys: Vec<Key> = pressed
                    .iter()
                    .filter(|key| key != &&event.code)
                    .cloned()
                    .collect();

                (
                    State::Idle(remaining_keys),
                    key_release_sequence(&vec![event.code]),
                )
            }
            (State::Idle(pressed), KeyValue::Repeat) => (
                State::Idle(pressed.clone()),
                key_repeat_sequence(&vec![event.code]),
            ),
            (State::ReleasingHotkey(pressed), KeyValue::Release) => {
                let remaining_keys: Vec<Key> = pressed
                    .iter()
                    .filter(|key| key != &&event.code)
                    .cloned()
                    .collect();
                if remaining_keys.is_empty() {
                    (State::Idle(Vec::new()), Vec::new())
                } else {
                    (State::ReleasingHotkey(remaining_keys), Vec::new())
                }
            }
            (state, _) => (state.clone(), Vec::new()),
        };
        self.state = new_state;
        send_event
    }
}

fn key_press_sequence(keys: &Vec<Key>) -> Vec<InputEvent> {
    keys.iter().map(|key| InputEvent::key_press(*key)).fold(
        Vec::with_capacity(keys.len() * 2),
        |mut res, evt| {
            res.push(evt);
            res.push(InputEvent::syn_report());
            res
        },
    )
}

fn key_repeat_sequence(keys: &Vec<Key>) -> Vec<InputEvent> {
    keys.iter()
        .filter(|key| !is_modifier(&key))
        .map(|key| InputEvent::key_repeat(*key))
        .fold(Vec::with_capacity(keys.len() * 2), |mut res, evt| {
            res.push(evt);
            res.push(InputEvent::syn_report());
            res
        })
}

fn key_release_sequence(keys: &Vec<Key>) -> Vec<InputEvent> {
    keys.iter()
        .map(|key| InputEvent::key_release(*key))
        .rev()
        .fold(Vec::with_capacity(keys.len() * 2), |mut res, evt| {
            res.push(evt);
            res.push(InputEvent::syn_report());
            res
        })
}

#[derive(Default)]
struct Hotkeys {
    mappings: HashMap<KeySet, Vec<Key>>,
}

impl Hotkeys {
    fn new(config: &Config) -> anyhow::Result<Self> {
        let mut mappings = HashMap::new();
        for mapping in &config.mappings {
            mappings.insert(KeySet::from_iter(mapping.on.clone()), mapping.send.clone());
        }
        Ok(Self { mappings })
    }

    fn query(&self, combination: &KeySet) -> Match {
        for (trigger, send) in &self.mappings {
            if trigger == combination {
                return Match::Complete(send.clone());
            }
            // match is only still possible if there are only modifers pressed yet, otherwise it must be complete
            if trigger.is_superset(combination) && combination.iter().all(is_modifier) {
                return Match::Possible;
            }
        }
        Match::Impossible
    }
}

fn is_modifier(key: &Key) -> bool {
    match key {
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

type KeySet = BTreeSet<Key>;

#[derive(Debug, PartialEq, Eq)]
enum Match {
    Impossible,
    Possible,
    Complete(Vec<Key>),
}

#[cfg(test)]
mod hotkeys_test {
    use crate::{
        config::{Config, Mapping},
        engine::{Hotkeys, KeySet, Match},
        keys::Key,
    };

    #[test]
    fn match_impossible_with_empty_config_when_modifier_is_pressed() {
        let sut = Hotkeys::new(&Config::default()).unwrap();

        let result = sut.query(&KeySet::from([Key::CtrlLeft]));

        assert_eq!(result, Match::Impossible);
    }

    #[test]
    fn match_impossible_with_empty_config_when_non_modifier_is_pressed() {
        let sut = Hotkeys::new(&Config::default()).unwrap();

        let result = sut.query(&KeySet::from([Key::A]));

        assert_eq!(result, Match::Impossible);
    }

    #[test]
    fn match_impossible_with_empty_config_when_nothing_is_pressed() {
        let sut = Hotkeys::new(&Config::default()).unwrap();

        let result = sut.query(&KeySet::from([]));

        assert_eq!(result, Match::Impossible);
    }

    #[test]
    fn match_impossible_when_uncofigured_combination_pressed() {
        let sut = Hotkeys::new(&Config {
            mappings: vec![Mapping {
                on: vec![Key::CtrlLeft, Key::ShiftLeft, Key::A],
                send: vec![Key::B],
            }],
        })
        .unwrap();

        let result = sut.query(&KeySet::from([Key::AltLeft, Key::C]));

        assert_eq!(result, Match::Impossible);
    }

    #[test]
    fn match_impossible_when_non_modifier_pressed_and_not_complete() {
        let sut = Hotkeys::new(&Config {
            mappings: vec![Mapping {
                on: vec![Key::CtrlLeft, Key::ShiftLeft, Key::A],
                send: vec![Key::B],
            }],
        })
        .unwrap();

        let result = sut.query(&KeySet::from([Key::CtrlLeft, Key::A]));

        assert_eq!(result, Match::Impossible);
    }

    #[test]
    fn match_impossible_when_wrong_modifier_pressed() {
        let sut = Hotkeys::new(&Config {
            mappings: vec![Mapping {
                on: vec![Key::CtrlLeft, Key::ShiftLeft, Key::A],
                send: vec![Key::B],
            }],
        })
        .unwrap();

        let result = sut.query(&KeySet::from([Key::AltRight]));

        assert_eq!(result, Match::Impossible);
    }

    #[test]
    fn match_impossible_when_modifier_pressed_but_none_is_configured() {
        let sut = Hotkeys::new(&Config {
            mappings: vec![Mapping {
                on: vec![Key::A],
                send: vec![Key::B],
            }],
        })
        .unwrap();

        let result = sut.query(&KeySet::from([Key::AltRight]));

        assert_eq!(result, Match::Impossible);
    }

    #[test]
    fn match_possible_when_nothing_is_pressed() {
        let sut = Hotkeys::new(&Config {
            mappings: vec![Mapping {
                on: vec![Key::CtrlLeft, Key::A],
                send: vec![Key::B],
            }],
        })
        .unwrap();

        let result = sut.query(&KeySet::from([]));

        assert_eq!(result, Match::Possible);
    }

    #[test]
    fn match_possible_when_single_matching_modifier_is_pressed() {
        let sut = Hotkeys::new(&Config {
            mappings: vec![Mapping {
                on: vec![Key::CtrlLeft, Key::A],
                send: vec![Key::B],
            }],
        })
        .unwrap();

        let result = sut.query(&KeySet::from([Key::CtrlLeft]));

        assert_eq!(result, Match::Possible);
    }

    #[test]
    fn match_possible_when_one_of_multiple_matching_modifiers_are_pressed() {
        let sut = Hotkeys::new(&Config {
            mappings: vec![Mapping {
                on: vec![Key::CtrlLeft, Key::ShiftLeft, Key::AltLeft, Key::A],
                send: vec![Key::B],
            }],
        })
        .unwrap();

        let result = sut.query(&KeySet::from([Key::ShiftLeft]));

        assert_eq!(result, Match::Possible);
    }

    #[test]
    fn match_possible_when_some_of_multiple_matching_modifiers_are_pressed() {
        let sut = Hotkeys::new(&Config {
            mappings: vec![Mapping {
                on: vec![Key::CtrlLeft, Key::ShiftLeft, Key::AltLeft, Key::A],
                send: vec![Key::B],
            }],
        })
        .unwrap();

        let result = sut.query(&KeySet::from([Key::ShiftLeft, Key::AltLeft]));

        assert_eq!(result, Match::Possible);
    }

    #[test]
    fn match_possible_when_all_of_multiple_matching_modifiers_are_pressed() {
        let sut = Hotkeys::new(&Config {
            mappings: vec![Mapping {
                on: vec![Key::CtrlLeft, Key::ShiftLeft, Key::AltLeft, Key::A],
                send: vec![Key::B],
            }],
        })
        .unwrap();

        let result = sut.query(&KeySet::from([Key::CtrlLeft, Key::ShiftLeft, Key::AltLeft]));

        assert_eq!(result, Match::Possible);
    }

    #[test]
    fn match_complete_when_no_modifier_configured() {
        let sut = Hotkeys::new(&Config {
            mappings: vec![Mapping {
                on: vec![Key::A],
                send: vec![Key::B],
            }],
        })
        .unwrap();

        let result = sut.query(&KeySet::from([Key::A]));

        assert_eq!(result, Match::Complete(vec![Key::B]));
    }

    #[test]
    fn match_complete_with_single_modifier() {
        let sut = Hotkeys::new(&Config {
            mappings: vec![Mapping {
                on: vec![Key::CtrlLeft, Key::A],
                send: vec![Key::B],
            }],
        })
        .unwrap();

        let result = sut.query(&KeySet::from([Key::CtrlLeft, Key::A]));

        assert_eq!(result, Match::Complete(vec![Key::B]));
    }

    #[test]
    fn match_complete_with_multiple_modifiers() {
        let sut = Hotkeys::new(&Config {
            mappings: vec![Mapping {
                on: vec![Key::CtrlLeft, Key::AltLeft, Key::A],
                send: vec![Key::B],
            }],
        })
        .unwrap();

        let result = sut.query(&KeySet::from([Key::CtrlLeft, Key::AltLeft, Key::A]));

        assert_eq!(result, Match::Complete(vec![Key::B]));
    }

    #[test]
    fn incremental_with_multiple_hotkeys_when_match_is_found() {
        let sut = Hotkeys::new(&Config {
            mappings: vec![
                Mapping {
                    on: vec![Key::CtrlLeft, Key::AltLeft, Key::A],
                    send: vec![Key::B],
                },
                Mapping {
                    on: vec![Key::D],
                    send: vec![Key::E],
                },
                Mapping {
                    on: vec![Key::Capslock, Key::K],
                    send: vec![Key::CtrlLeft, Key::K],
                },
                Mapping {
                    on: vec![Key::CtrlLeft, Key::Capslock, Key::C],
                    send: vec![Key::CtrlLeft, Key::V],
                },
            ],
        })
        .unwrap();

        assert_eq!(sut.query(&KeySet::from([Key::CtrlLeft])), Match::Possible);
        assert_eq!(
            sut.query(&KeySet::from([Key::CtrlLeft, Key::Capslock])),
            Match::Possible
        );
        assert_eq!(
            sut.query(&KeySet::from([Key::CtrlLeft, Key::Capslock, Key::C])),
            Match::Complete(vec![Key::CtrlLeft, Key::V])
        );
    }

    #[test]
    fn incremental_with_multiple_hotkeys_when_no_match_is_found() {
        let sut = Hotkeys::new(&Config {
            mappings: vec![
                Mapping {
                    on: vec![Key::CtrlLeft, Key::AltLeft, Key::A],
                    send: vec![Key::B],
                },
                Mapping {
                    on: vec![Key::D],
                    send: vec![Key::E],
                },
                Mapping {
                    on: vec![Key::Capslock, Key::K],
                    send: vec![Key::CtrlLeft, Key::K],
                },
                Mapping {
                    on: vec![Key::CtrlLeft, Key::Capslock, Key::C],
                    send: vec![Key::CtrlLeft, Key::V],
                },
            ],
        })
        .unwrap();

        assert_eq!(sut.query(&KeySet::from([Key::CtrlLeft])), Match::Possible);
        assert_eq!(
            sut.query(&KeySet::from([Key::CtrlLeft, Key::Capslock])),
            Match::Possible
        );
        assert_eq!(
            sut.query(&KeySet::from([Key::CtrlLeft, Key::Capslock, Key::AltLeft])),
            Match::Impossible
        );
    }
}

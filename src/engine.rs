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
    Idle(Vec<Key>),
    PartialHotkey(Vec<Key>),
    CompleteHotkey { trigger: Vec<Key>, send: Vec<Key> },
    ReleasingHotkey(Vec<Key>),
}

impl Engine {
    pub fn new(config: &Config) -> anyhow::Result<Self> {
        Ok(Self {
            hotkeys: Hotkeys::new(config)?,
            state: State::Idle(Vec::new()),
        })
    }

    // TODO: describe in picture, find duplications and maybe refactor them
    pub fn handle(&mut self, event: InputEvent) -> Vec<InputEvent> {
        let (new_state, send_event) = match (&self.state, event.value) {
            (State::Idle(pressed), KeyValue::Press) => {
                let mut now_pressed = pressed.clone();
                now_pressed.push(event.code);
                match self.hotkeys.query(&KeySet::from_iter(now_pressed.clone())) {
                    Match::Impossible => (
                        // stay idle, forward key press
                        State::Idle(now_pressed.clone()),
                        now_pressed
                            .iter()
                            .map(|key| InputEvent::key_press(*key))
                            .collect(),
                    ),
                    Match::Possible => {
                        let send_keys = pressed
                            .iter()
                            .map(|key| InputEvent::key_release(*key))
                            .rev()
                            .collect();

                        (State::PartialHotkey(now_pressed), send_keys)
                    }
                    Match::Complete(mapped_keys) => {
                        let mut send_keys: Vec<InputEvent> = pressed
                            .iter()
                            .map(|key| InputEvent::key_release(*key))
                            .rev()
                            .collect();
                        send_keys.extend(mapped_keys.iter().map(|key| InputEvent::key_press(*key)));
                        (
                            State::CompleteHotkey {
                                trigger: now_pressed,
                                send: mapped_keys.clone(),
                            },
                            send_keys,
                        )
                    }
                }
            }
            (State::PartialHotkey(pressed), KeyValue::Press) => {
                let mut now_pressed = pressed.clone();
                now_pressed.push(event.code);
                match self.hotkeys.query(&KeySet::from_iter(now_pressed.clone())) {
                    Match::Impossible => (
                        State::Idle(now_pressed.clone()),
                        now_pressed
                            .iter()
                            .map(|key| InputEvent::key_press(*key))
                            .collect(),
                    ),
                    Match::Possible => (State::PartialHotkey(now_pressed), Vec::new()),
                    Match::Complete(mapped_keys) => (
                        State::CompleteHotkey {
                            trigger: now_pressed,
                            send: mapped_keys.clone(),
                        },
                        mapped_keys
                            .iter()
                            .map(|key| InputEvent::key_press(*key))
                            .collect(),
                    ),
                }
            }
            (State::ReleasingHotkey(pressed), KeyValue::Press) => {
                let mut now_pressed = pressed.clone();
                now_pressed.push(event.code);
                match self.hotkeys.query(&KeySet::from_iter(now_pressed.clone())) {
                    Match::Impossible => (
                        State::Idle(now_pressed.clone()),
                        now_pressed
                            .iter()
                            .map(|key| InputEvent::key_press(*key))
                            .collect(),
                    ),
                    Match::Possible => (State::PartialHotkey(now_pressed), Vec::new()),
                    Match::Complete(mapped_keys) => (
                        State::CompleteHotkey {
                            trigger: now_pressed,
                            send: mapped_keys.clone(),
                        },
                        mapped_keys
                            .iter()
                            .map(|key| InputEvent::key_press(*key))
                            .collect(),
                    ),
                }
            }
            (State::CompleteHotkey { trigger, send }, KeyValue::Repeat) => (
                // trigger hotkey repeat
                State::CompleteHotkey {
                    trigger: trigger.clone(),
                    send: send.clone(),
                },
                send.iter()
                    .filter(|key| !is_modifier(&key))
                    .map(|key| InputEvent::key_repeat(*key))
                    .collect(),
            ),
            (State::CompleteHotkey { trigger, send }, KeyValue::Release) => {
                let remaining_keys: Vec<Key> = trigger
                    .iter()
                    .filter(|key| key != &&event.code)
                    .cloned()
                    .collect();

                let send_keys: Vec<InputEvent> = send
                    .iter()
                    .map(|key| InputEvent::key_release(*key))
                    .rev()
                    .collect();

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
                    vec![InputEvent::key_release(event.code)],
                )
            }
            (State::Idle(pressed), KeyValue::Repeat) => (
                State::Idle(pressed.clone()),
                vec![InputEvent::key_repeat(event.code)],
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

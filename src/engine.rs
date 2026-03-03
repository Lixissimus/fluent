use std::collections::{BTreeSet, HashMap};

use crate::{
    config::{Config, Mapping},
    event::{InputEvent, KeyValue},
    keys::Key,
};

pub struct Engine {
    hotkeys: HashMap<String, Hotkeys>,
    modifiers: HashMap<String, Vec<Key>>,
    state: State,
    mode: String,
    previously_pressed: Vec<Key>,
    now_pressed: Vec<Key>,
}

#[derive(Clone, Debug)]
enum State {
    Idle,
    PartialHotkey,
    CompleteHotkey(Vec<Key>),
}

#[derive(Debug)]
enum KeyEvent {
    Press(Match),
    Repeat,
    Release,
    Nothing,
}

#[derive(Debug, PartialEq, Eq)]
enum Match {
    Impossible,
    Possible,
    Complete(Vec<Key>),
}

impl Engine {
    pub fn new(config: &Config) -> Self {
        Self {
            hotkeys: config
                .modes
                .iter()
                .map(|mode| {
                    (
                        mode.name.clone(),
                        Hotkeys::new(mode.mappings.clone(), mode.modifiers.clone()),
                    )
                })
                .collect(),
            modifiers: config
                .modes
                .iter()
                .map(|mode| (mode.name.clone(), mode.modifiers.clone()))
                .collect(),
            state: State::Idle,
            mode: config
                .modes
                .first()
                .and_then(|mode| Some(mode.name.clone()))
                .unwrap_or_default(),
            previously_pressed: Vec::new(),
            now_pressed: Vec::new(),
        }
    }

    pub fn handle(&mut self, event: InputEvent) -> Vec<InputEvent> {
        let action = self.handle_input(&event);
        let (new_state, output) = self.state_transition(event.code, action);
        self.state = new_state;
        output
    }

    fn handle_input(&mut self, event: &InputEvent) -> KeyEvent {
        match event.value {
            KeyValue::Release => {
                self.previously_pressed = self.now_pressed.clone();
                self.now_pressed.retain(|key| key != &event.code);
                KeyEvent::Release
            }
            KeyValue::Press => {
                self.previously_pressed = self.now_pressed.clone();
                self.now_pressed.push(event.code);
                match self
                    .hotkeys
                    .get(&self.mode)
                    .and_then(|hotkey| {
                        Some(hotkey.query(&KeySet::from_iter(self.now_pressed.clone())))
                    })
                    .unwrap_or(Match::Impossible)
                {
                    Match::Impossible => KeyEvent::Press(Match::Impossible),
                    Match::Possible => KeyEvent::Press(Match::Possible),
                    Match::Complete(trigger_keys) => KeyEvent::Press(Match::Complete(trigger_keys)),
                }
            }
            KeyValue::Repeat => KeyEvent::Repeat,
            KeyValue::Other(_) => KeyEvent::Nothing,
        }
    }

    fn state_transition(&self, key: Key, action: KeyEvent) -> (State, Vec<InputEvent>) {
        match (&self.state, action) {
            (State::Idle, KeyEvent::Press(Match::Impossible)) => {
                (State::Idle, key_press_sequence(&vec![key]))
            }
            (State::Idle, KeyEvent::Press(Match::Possible)) => (
                State::PartialHotkey,
                key_release_sequence(&self.previously_pressed),
            ),
            (State::Idle, KeyEvent::Press(Match::Complete(triggered))) => {
                let mut send_keys = key_release_sequence(&self.previously_pressed);
                send_keys.extend(key_press_sequence(&triggered));
                (State::CompleteHotkey(triggered.clone()), send_keys)
            }
            (State::Idle, KeyEvent::Repeat) => (
                State::Idle,
                if self.is_modifier(key) {
                    Vec::new()
                } else {
                    key_repeat_sequence(&vec![key])
                },
            ),
            (State::Idle, KeyEvent::Release) => (State::Idle, key_release_sequence(&vec![key])),

            (State::PartialHotkey, KeyEvent::Press(Match::Impossible)) => {
                (State::Idle, key_press_sequence(&self.now_pressed))
            }
            (State::PartialHotkey, KeyEvent::Press(Match::Possible)) => {
                (State::PartialHotkey, Vec::new())
            }
            (State::PartialHotkey, KeyEvent::Press(Match::Complete(triggered))) => (
                State::CompleteHotkey(triggered.clone()),
                key_press_sequence(&triggered),
            ),
            (State::PartialHotkey, KeyEvent::Repeat) => (State::PartialHotkey, Vec::new()),
            (State::PartialHotkey, KeyEvent::Release) => {
                if self.now_pressed.is_empty() {
                    (State::Idle, Vec::new())
                } else {
                    (State::PartialHotkey, Vec::new())
                }
            }

            (State::CompleteHotkey(triggered), KeyEvent::Press(_)) => {
                (State::CompleteHotkey(triggered.clone()), Vec::new())
            }
            (State::CompleteHotkey(triggered), KeyEvent::Repeat) => (
                State::CompleteHotkey(triggered.clone()),
                key_repeat_sequence(
                    &triggered
                        .iter()
                        .filter(|key| !self.is_modifier(**key))
                        .cloned()
                        .collect(),
                ),
            ),
            (State::CompleteHotkey(triggered), KeyEvent::Release) => {
                let hotkey_release = key_release_sequence(&triggered);
                if self.now_pressed.is_empty() {
                    (State::Idle, hotkey_release)
                } else {
                    (State::PartialHotkey, hotkey_release)
                }
            }

            (state, KeyEvent::Nothing) => (state.clone(), Vec::new()),
        }
    }

    fn is_modifier(&self, key: Key) -> bool {
        self.modifiers
            .get(&self.mode)
            .and_then(|modifiers| Some(modifiers.contains(&key)))
            .unwrap_or(false)
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
    keys.iter().map(|key| InputEvent::key_repeat(*key)).fold(
        Vec::with_capacity(keys.len() * 2),
        |mut res, evt| {
            res.push(evt);
            res.push(InputEvent::syn_report());
            res
        },
    )
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
    modifiers: Vec<Key>,
}

type KeySet = BTreeSet<Key>;

impl Hotkeys {
    fn new(mappings: Vec<Mapping>, modifiers: Vec<Key>) -> Self {
        Self {
            mappings: mappings
                .into_iter()
                .map(|m| (KeySet::from_iter(m.on), m.send))
                .collect(),
            modifiers,
        }
    }

    fn query(&self, combination: &KeySet) -> Match {
        for (trigger, send) in &self.mappings {
            if trigger == combination {
                return Match::Complete(send.clone());
            }
            // match is only still possible if there are only modifers pressed yet, otherwise it must be complete
            if trigger.is_superset(combination)
                && combination.iter().all(|key| self.modifiers.contains(key))
            {
                return Match::Possible;
            }
        }
        Match::Impossible
    }
}

#[cfg(test)]
mod hotkeys_test {
    use crate::{
        config::Mapping,
        engine::{Hotkeys, KeySet, Match},
        keys::Key,
    };

    #[test]
    fn match_impossible_with_empty_config_when_modifier_is_pressed() {
        let sut = Hotkeys::new(vec![], vec![]);

        let result = sut.query(&KeySet::from([Key::CtrlLeft]));

        assert_eq!(result, Match::Impossible);
    }

    #[test]
    fn match_impossible_with_empty_config_when_non_modifier_is_pressed() {
        let sut = Hotkeys::new(vec![], vec![]);

        let result = sut.query(&KeySet::from([Key::A]));

        assert_eq!(result, Match::Impossible);
    }

    #[test]
    fn match_impossible_with_empty_config_when_nothing_is_pressed() {
        let sut = Hotkeys::new(vec![], vec![]);

        let result = sut.query(&KeySet::from([]));

        assert_eq!(result, Match::Impossible);
    }

    #[test]
    fn match_impossible_when_uncofigured_combination_pressed() {
        let sut = Hotkeys::new(
            vec![Mapping {
                on: vec![Key::CtrlLeft, Key::ShiftLeft, Key::A],
                send: vec![Key::B],
            }],
            vec![],
        );

        let result = sut.query(&KeySet::from([Key::AltLeft, Key::C]));

        assert_eq!(result, Match::Impossible);
    }

    #[test]
    fn match_impossible_when_non_modifier_pressed_and_not_complete() {
        let sut = Hotkeys::new(
            vec![Mapping {
                on: vec![Key::CtrlLeft, Key::ShiftLeft, Key::A],
                send: vec![Key::B],
            }],
            vec![],
        );

        let result = sut.query(&KeySet::from([Key::CtrlLeft, Key::A]));

        assert_eq!(result, Match::Impossible);
    }

    #[test]
    fn match_impossible_when_wrong_modifier_pressed() {
        let sut = Hotkeys::new(
            vec![Mapping {
                on: vec![Key::CtrlLeft, Key::ShiftLeft, Key::A],
                send: vec![Key::B],
            }],
            vec![],
        );

        let result = sut.query(&KeySet::from([Key::AltRight]));

        assert_eq!(result, Match::Impossible);
    }

    #[test]
    fn match_impossible_when_modifier_pressed_but_none_is_configured() {
        let sut = Hotkeys::new(
            vec![Mapping {
                on: vec![Key::A],
                send: vec![Key::B],
            }],
            vec![],
        );

        let result = sut.query(&KeySet::from([Key::AltRight]));

        assert_eq!(result, Match::Impossible);
    }

    #[test]
    fn match_possible_when_nothing_is_pressed() {
        let sut = Hotkeys::new(
            vec![Mapping {
                on: vec![Key::CtrlLeft, Key::A],
                send: vec![Key::B],
            }],
            vec![],
        );

        let result = sut.query(&KeySet::from([]));

        assert_eq!(result, Match::Possible);
    }

    #[test]
    fn match_possible_when_single_matching_modifier_is_pressed() {
        let sut = Hotkeys::new(
            vec![Mapping {
                on: vec![Key::CtrlLeft, Key::A],
                send: vec![Key::B],
            }],
            vec![Key::CtrlLeft],
        );

        let result = sut.query(&KeySet::from([Key::CtrlLeft]));

        assert_eq!(result, Match::Possible);
    }

    #[test]
    fn match_possible_when_one_of_multiple_matching_modifiers_are_pressed() {
        let sut = Hotkeys::new(
            vec![Mapping {
                on: vec![Key::CtrlLeft, Key::ShiftLeft, Key::AltLeft, Key::A],
                send: vec![Key::B],
            }],
            vec![Key::CtrlLeft, Key::ShiftLeft, Key::AltLeft],
        );

        let result = sut.query(&KeySet::from([Key::ShiftLeft]));

        assert_eq!(result, Match::Possible);
    }

    #[test]
    fn match_possible_when_some_of_multiple_matching_modifiers_are_pressed() {
        let sut = Hotkeys::new(
            vec![Mapping {
                on: vec![Key::CtrlLeft, Key::ShiftLeft, Key::AltLeft, Key::A],
                send: vec![Key::B],
            }],
            vec![Key::CtrlLeft, Key::ShiftLeft, Key::AltLeft],
        );

        let result = sut.query(&KeySet::from([Key::ShiftLeft, Key::AltLeft]));

        assert_eq!(result, Match::Possible);
    }

    #[test]
    fn match_possible_when_all_of_multiple_matching_modifiers_are_pressed() {
        let sut = Hotkeys::new(
            vec![Mapping {
                on: vec![Key::CtrlLeft, Key::ShiftLeft, Key::AltLeft, Key::A],
                send: vec![Key::B],
            }],
            vec![Key::CtrlLeft, Key::ShiftLeft, Key::AltLeft],
        );

        let result = sut.query(&KeySet::from([Key::CtrlLeft, Key::ShiftLeft, Key::AltLeft]));

        assert_eq!(result, Match::Possible);
    }

    #[test]
    fn match_complete_when_no_modifier_configured() {
        let sut = Hotkeys::new(
            vec![Mapping {
                on: vec![Key::A],
                send: vec![Key::B],
            }],
            vec![Key::CtrlLeft],
        );

        let result = sut.query(&KeySet::from([Key::A]));

        assert_eq!(result, Match::Complete(vec![Key::B]));
    }

    #[test]
    fn match_complete_with_single_modifier() {
        let sut = Hotkeys::new(
            vec![Mapping {
                on: vec![Key::CtrlLeft, Key::A],
                send: vec![Key::B],
            }],
            vec![Key::CtrlLeft],
        );

        let result = sut.query(&KeySet::from([Key::CtrlLeft, Key::A]));

        assert_eq!(result, Match::Complete(vec![Key::B]));
    }

    #[test]
    fn match_complete_with_multiple_modifiers() {
        let sut = Hotkeys::new(
            vec![Mapping {
                on: vec![Key::CtrlLeft, Key::AltLeft, Key::A],
                send: vec![Key::B],
            }],
            vec![Key::CtrlLeft, Key::AltLeft],
        );

        let result = sut.query(&KeySet::from([Key::CtrlLeft, Key::AltLeft, Key::A]));

        assert_eq!(result, Match::Complete(vec![Key::B]));
    }

    #[test]
    fn incremental_with_multiple_hotkeys_when_match_is_found() {
        let sut = Hotkeys::new(
            vec![
                Mapping {
                    on: vec![Key::CtrlLeft, Key::AltLeft, Key::A],
                    send: vec![Key::B],
                },
                Mapping {
                    on: vec![Key::D],
                    send: vec![Key::E],
                },
                Mapping {
                    on: vec![Key::AltLeft, Key::K],
                    send: vec![Key::CtrlLeft, Key::K],
                },
                Mapping {
                    on: vec![Key::CtrlLeft, Key::AltLeft, Key::C],
                    send: vec![Key::CtrlLeft, Key::V],
                },
            ],
            vec![Key::CtrlLeft, Key::AltLeft],
        );

        assert_eq!(sut.query(&KeySet::from([Key::CtrlLeft])), Match::Possible);
        assert_eq!(
            sut.query(&KeySet::from([Key::CtrlLeft, Key::AltLeft])),
            Match::Possible
        );
        assert_eq!(
            sut.query(&KeySet::from([Key::CtrlLeft, Key::AltLeft, Key::C])),
            Match::Complete(vec![Key::CtrlLeft, Key::V])
        );
    }

    #[test]
    fn incremental_with_multiple_hotkeys_when_no_match_is_found() {
        let sut = Hotkeys::new(
            vec![
                Mapping {
                    on: vec![Key::CtrlLeft, Key::AltLeft, Key::A],
                    send: vec![Key::B],
                },
                Mapping {
                    on: vec![Key::D],
                    send: vec![Key::E],
                },
                Mapping {
                    on: vec![Key::ShiftLeft, Key::K],
                    send: vec![Key::CtrlLeft, Key::K],
                },
                Mapping {
                    on: vec![Key::CtrlLeft, Key::ShiftLeft, Key::C],
                    send: vec![Key::CtrlLeft, Key::V],
                },
            ],
            vec![Key::CtrlLeft, Key::ShiftLeft, Key::AltLeft],
        );

        assert_eq!(sut.query(&KeySet::from([Key::CtrlLeft])), Match::Possible);
        assert_eq!(
            sut.query(&KeySet::from([Key::CtrlLeft, Key::ShiftLeft])),
            Match::Possible
        );
        assert_eq!(
            sut.query(&KeySet::from([Key::CtrlLeft, Key::ShiftLeft, Key::AltLeft])),
            Match::Impossible
        );
    }
}

use std::collections::{BTreeSet, HashMap};

use crate::{
    config::Config,
    event::{InputEvent, KeyValue},
    keys::Key,
};

pub struct Engine {
    hotkeys: Hotkeys,
    modifiers: Vec<Key>,
    state: State,
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
enum Action {
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
            hotkeys: Hotkeys::new(config),
            modifiers: config.modifiers.clone(),
            state: State::Idle,
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

    fn handle_input(&mut self, event: &InputEvent) -> Action {
        match event.value {
            KeyValue::Release => {
                self.previously_pressed = self.now_pressed.clone();
                self.now_pressed.retain(|key| key != &event.code);
                Action::Release
            }
            KeyValue::Press => {
                self.previously_pressed = self.now_pressed.clone();
                self.now_pressed.push(event.code);
                match self
                    .hotkeys
                    .query(&KeySet::from_iter(self.now_pressed.clone()))
                {
                    Match::Impossible => Action::Press(Match::Impossible),
                    Match::Possible => Action::Press(Match::Possible),
                    Match::Complete(trigger_keys) => Action::Press(Match::Complete(trigger_keys)),
                }
            }
            KeyValue::Repeat => Action::Repeat,
            KeyValue::Other(_) => Action::Nothing,
        }
    }

    fn state_transition(&self, key: Key, action: Action) -> (State, Vec<InputEvent>) {
        match (&self.state, action) {
            (State::Idle, Action::Press(Match::Impossible)) => {
                (State::Idle, key_press_sequence(&vec![key]))
            }
            (State::Idle, Action::Press(Match::Possible)) => (
                State::PartialHotkey,
                key_release_sequence(&self.previously_pressed),
            ),
            (State::Idle, Action::Press(Match::Complete(triggered))) => {
                let mut send_keys = key_release_sequence(&self.previously_pressed);
                send_keys.extend(key_press_sequence(&triggered));
                (State::CompleteHotkey(triggered.clone()), send_keys)
            }
            (State::Idle, Action::Repeat) => (
                State::Idle,
                if self.modifiers.contains(&key) {
                    Vec::new()
                } else {
                    key_repeat_sequence(&vec![key])
                },
            ),
            (State::Idle, Action::Release) => (State::Idle, key_release_sequence(&vec![key])),

            (State::PartialHotkey, Action::Press(Match::Impossible)) => {
                (State::Idle, key_press_sequence(&self.now_pressed))
            }
            (State::PartialHotkey, Action::Press(Match::Possible)) => {
                (State::PartialHotkey, Vec::new())
            }
            (State::PartialHotkey, Action::Press(Match::Complete(triggered))) => (
                State::CompleteHotkey(triggered.clone()),
                key_press_sequence(&triggered),
            ),
            (State::PartialHotkey, Action::Repeat) => (State::PartialHotkey, Vec::new()),
            (State::PartialHotkey, Action::Release) => {
                if self.now_pressed.is_empty() {
                    (State::Idle, Vec::new())
                } else {
                    (State::PartialHotkey, Vec::new())
                }
            }

            (State::CompleteHotkey(triggered), Action::Press(_)) => {
                (State::CompleteHotkey(triggered.clone()), Vec::new())
            }
            (State::CompleteHotkey(triggered), Action::Repeat) => (
                State::CompleteHotkey(triggered.clone()),
                key_repeat_sequence(
                    &triggered
                        .iter()
                        .filter(|key| !self.modifiers.contains(key))
                        .cloned()
                        .collect(),
                ),
            ),
            (State::CompleteHotkey(triggered), Action::Release) => {
                let hotkey_release = key_release_sequence(&triggered);
                if self.now_pressed.is_empty() {
                    (State::Idle, hotkey_release)
                } else {
                    (State::PartialHotkey, hotkey_release)
                }
            }

            (state, Action::Nothing) => (state.clone(), Vec::new()),
        }
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
    fn new(config: &Config) -> Self {
        Self {
            mappings: config
                .mappings
                .iter()
                .map(|m| (KeySet::from_iter(m.on.clone()), m.send.clone()))
                .collect(),
            modifiers: config.modifiers.clone(),
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
        config::{Config, Mapping},
        engine::{Hotkeys, KeySet, Match},
        keys::Key,
    };

    #[test]
    fn match_impossible_with_empty_config_when_modifier_is_pressed() {
        let sut = Hotkeys::new(&Config::default());

        let result = sut.query(&KeySet::from([Key::CtrlLeft]));

        assert_eq!(result, Match::Impossible);
    }

    #[test]
    fn match_impossible_with_empty_config_when_non_modifier_is_pressed() {
        let sut = Hotkeys::new(&Config::default());

        let result = sut.query(&KeySet::from([Key::A]));

        assert_eq!(result, Match::Impossible);
    }

    #[test]
    fn match_impossible_with_empty_config_when_nothing_is_pressed() {
        let sut = Hotkeys::new(&Config::default());

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
            ..Default::default()
        });

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
            ..Default::default()
        });

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
            ..Default::default()
        });

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
            ..Default::default()
        });

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
            ..Default::default()
        });

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
            ..Default::default()
        });

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
            ..Default::default()
        });

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
            ..Default::default()
        });

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
            ..Default::default()
        });

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
            ..Default::default()
        });

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
            ..Default::default()
        });

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
            ..Default::default()
        });

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
                    on: vec![Key::AltLeft, Key::K],
                    send: vec![Key::CtrlLeft, Key::K],
                },
                Mapping {
                    on: vec![Key::CtrlLeft, Key::AltLeft, Key::C],
                    send: vec![Key::CtrlLeft, Key::V],
                },
            ],
            ..Default::default()
        });

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
                    on: vec![Key::ShiftLeft, Key::K],
                    send: vec![Key::CtrlLeft, Key::K],
                },
                Mapping {
                    on: vec![Key::CtrlLeft, Key::ShiftLeft, Key::C],
                    send: vec![Key::CtrlLeft, Key::V],
                },
            ],
            ..Default::default()
        });

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

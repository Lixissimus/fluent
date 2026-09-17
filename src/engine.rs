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
    CompleteHotkey(Vec<TriggeredHotkey>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TriggeredHotkey {
    trigger: KeySet,
    send: Vec<Key>,
}

#[derive(Debug)]
enum Action {
    Pressed(PressOutcome),
    Repeated,
    Released,
    Ignored,
}

#[derive(Debug, PartialEq, Eq)]
enum PressOutcome {
    RegularKey,
    PossibleHotkey,
    CompletedHotkey(TriggeredHotkey),
}

impl From<Match> for PressOutcome {
    fn from(value: Match) -> Self {
        match value {
            Match::Impossible => Self::RegularKey,
            Match::Possible => Self::PossibleHotkey,
            Match::Complete(triggered) => Self::CompletedHotkey(triggered),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Match {
    Impossible,
    Possible,
    Complete(TriggeredHotkey),
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
                Action::Released
            }
            KeyValue::Press => {
                self.previously_pressed = self.now_pressed.clone();
                self.now_pressed.push(event.code);
                let match_result = self
                    .hotkeys
                    .query(&KeySet::from_iter(self.now_pressed.clone()));
                Action::Pressed(match_result.into())
            }
            KeyValue::Repeat => Action::Repeated,
            KeyValue::Other(_) => Action::Ignored,
        }
    }

    fn state_transition(&self, key: Key, action: Action) -> (State, Vec<InputEvent>) {
        match (&self.state, action) {
            (State::Idle, Action::Pressed(outcome)) => self.idle_press_transition(key, outcome),
            (State::Idle, Action::Repeated) => self.idle_repeat_transition(key),
            (State::Idle, Action::Released) => (State::Idle, key_release_sequence(&vec![key])),
            (State::Idle, Action::Ignored) => (State::Idle, Vec::new()),

            (State::PartialHotkey, Action::Pressed(outcome)) => {
                self.partial_press_transition(outcome)
            }
            (State::PartialHotkey, Action::Repeated) => (State::PartialHotkey, Vec::new()),
            (State::PartialHotkey, Action::Released) => self.partial_release_transition(),
            (State::PartialHotkey, Action::Ignored) => (State::PartialHotkey, Vec::new()),

            (State::CompleteHotkey(active), Action::Pressed(_)) => {
                self.complete_press_transition(active)
            }
            (State::CompleteHotkey(active), Action::Repeated) => {
                self.complete_repeat_transition(key, active)
            }
            (State::CompleteHotkey(active), Action::Released) => {
                self.complete_release_transition(key, active)
            }
            (State::CompleteHotkey(active), Action::Ignored) => {
                (State::CompleteHotkey(active.to_vec()), Vec::new())
            }
        }
    }

    fn idle_press_transition(&self, key: Key, outcome: PressOutcome) -> (State, Vec<InputEvent>) {
        match outcome {
            PressOutcome::RegularKey => (State::Idle, key_press_sequence(&vec![key])),
            PressOutcome::PossibleHotkey => (
                State::PartialHotkey,
                key_release_sequence(&self.previously_pressed),
            ),
            PressOutcome::CompletedHotkey(triggered) => {
                let mut send_keys = key_release_sequence(&self.previously_pressed);
                send_keys.extend(key_press_sequence(&triggered.send));
                (State::CompleteHotkey(vec![triggered.clone()]), send_keys)
            }
        }
    }

    fn idle_repeat_transition(&self, key: Key) -> (State, Vec<InputEvent>) {
        (
            State::Idle,
            if self.modifiers.contains(&key) {
                Vec::new()
            } else {
                key_repeat_sequence(&vec![key])
            },
        )
    }

    fn partial_press_transition(&self, outcome: PressOutcome) -> (State, Vec<InputEvent>) {
        match outcome {
            PressOutcome::RegularKey => (State::Idle, key_press_sequence(&self.now_pressed)),
            PressOutcome::PossibleHotkey => (State::PartialHotkey, Vec::new()),
            PressOutcome::CompletedHotkey(triggered) => (
                State::CompleteHotkey(vec![triggered.clone()]),
                key_press_sequence(&triggered.send),
            ),
        }
    }

    fn partial_release_transition(&self) -> (State, Vec<InputEvent>) {
        if self.now_pressed.is_empty() {
            (State::Idle, Vec::new())
        } else {
            (State::PartialHotkey, Vec::new())
        }
    }

    fn complete_press_transition(&self, active: &[TriggeredHotkey]) -> (State, Vec<InputEvent>) {
        let mut active = active.to_vec();
        let combination = KeySet::from_iter(self.now_pressed.clone());
        let output = match self
            .hotkeys
            .query_additional(&combination, &active.as_slice())
        {
            Some(triggered) => {
                let output = key_press_sequence(&triggered.send);
                active.push(triggered);
                output
            }
            None => Vec::new(),
        };
        (State::CompleteHotkey(active), output)
    }

    fn complete_repeat_transition(
        &self,
        key: Key,
        active: &[TriggeredHotkey],
    ) -> (State, Vec<InputEvent>) {
        let repeat_keys = active
            .iter()
            .filter(|hotkey| hotkey.trigger.contains(&key))
            .flat_map(|hotkey| hotkey.send.iter())
            .filter(|key| !self.modifiers.contains(key))
            .cloned()
            .collect();
        (
            State::CompleteHotkey(active.to_vec()),
            key_repeat_sequence(&repeat_keys),
        )
    }

    fn complete_release_transition(
        &self,
        key: Key,
        active: &[TriggeredHotkey],
    ) -> (State, Vec<InputEvent>) {
        let mut remaining = Vec::new();
        let mut released = Vec::new();
        for hotkey in active {
            if hotkey.trigger.contains(&key) {
                released.extend(key_release_sequence(&hotkey.send));
            } else {
                remaining.push(hotkey.clone());
            }
        }

        if remaining.is_empty() {
            if self.now_pressed.is_empty() {
                (State::Idle, released)
            } else {
                (State::PartialHotkey, released)
            }
        } else {
            (State::CompleteHotkey(remaining), released)
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
                return Match::Complete(TriggeredHotkey {
                    trigger: trigger.clone(),
                    send: send.clone(),
                });
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

    fn query_additional(
        &self,
        combination: &KeySet,
        active: &[TriggeredHotkey],
    ) -> Option<TriggeredHotkey> {
        self.mappings.iter().find_map(|(trigger, send)| {
            (trigger.is_subset(combination)
                && !active.iter().any(|hotkey| hotkey.trigger == *trigger))
            .then(|| TriggeredHotkey {
                trigger: trigger.clone(),
                send: send.clone(),
            })
        })
    }
}

#[cfg(test)]
mod hotkeys_test {
    use crate::{
        config::{Config, Mapping},
        engine::{Hotkeys, KeySet, Match, TriggeredHotkey},
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

        assert_eq!(
            result,
            Match::Complete(TriggeredHotkey {
                trigger: KeySet::from([Key::A]),
                send: vec![Key::B],
            })
        );
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

        assert_eq!(
            result,
            Match::Complete(TriggeredHotkey {
                trigger: KeySet::from([Key::CtrlLeft, Key::A]),
                send: vec![Key::B],
            })
        );
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

        assert_eq!(
            result,
            Match::Complete(TriggeredHotkey {
                trigger: KeySet::from([Key::CtrlLeft, Key::AltLeft, Key::A]),
                send: vec![Key::B],
            })
        );
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
            Match::Complete(TriggeredHotkey {
                trigger: KeySet::from([Key::CtrlLeft, Key::AltLeft, Key::C]),
                send: vec![Key::CtrlLeft, Key::V],
            })
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

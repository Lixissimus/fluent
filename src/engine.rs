use std::collections::{BTreeSet, HashMap};

use crate::{
    config::{self, Config},
    event::{InputEvent, KeyValue},
    keys::Key,
};

pub struct Engine {
    hotkeys_by_mode: HashMap<String, HotkeyStore>,
    current_state: State,
    current_mode: String,
    previously_pressed: Vec<Key>,
    now_pressed: Vec<Key>,
}

#[derive(Clone, Debug)]
enum State {
    Idle,
    PartialHotkey,
    CompleteHotkey(config::Action),
}

#[derive(Debug)]
enum KeyEvent {
    Press(Match),
    Repeat,
    Release,
    Nothing,
}

pub enum Action {
    SendKeyEvent(Vec<InputEvent>),
}

impl Engine {
    pub fn new(config: &Config) -> Self {
        Self {
            hotkeys_by_mode: config
                .modes
                .iter()
                .map(|mode| {
                    (
                        mode.name.clone(),
                        HotkeyStore::new(mode.hotkeys.clone(), mode.modifiers.clone()),
                    )
                })
                .collect(),
            current_state: State::Idle,
            current_mode: config
                .modes
                .first()
                .and_then(|mode| Some(mode.name.clone()))
                .unwrap_or_default(),
            previously_pressed: Vec::new(),
            now_pressed: Vec::new(),
        }
    }

    pub fn handle(&mut self, event: InputEvent) -> Vec<Action> {
        let trigger = self.handle_input(&event);
        let (new_state, output) = self.state_transition(event.code, trigger);
        self.current_state = new_state;
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
                    .hotkeys_by_mode
                    .get(&self.current_mode)
                    .and_then(|hotkeys| {
                        Some(hotkeys.query(&KeySet::from_iter(self.now_pressed.clone())))
                    })
                    .unwrap_or(Match::Impossible)
                {
                    Match::Impossible => KeyEvent::Press(Match::Impossible),
                    Match::Possible => KeyEvent::Press(Match::Possible),
                    Match::Complete(action) => KeyEvent::Press(Match::Complete(action)),
                }
            }
            KeyValue::Repeat => KeyEvent::Repeat,
            KeyValue::Other(_) => KeyEvent::Nothing,
        }
    }

    fn state_transition(&mut self, key: Key, trigger: KeyEvent) -> (State, Vec<Action>) {
        match (&self.current_state, trigger) {
            (State::Idle, KeyEvent::Press(Match::Impossible)) => (
                State::Idle,
                vec![Action::SendKeyEvent(key_press_sequence(&vec![key]))],
            ),
            (State::Idle, KeyEvent::Press(Match::Possible)) => (
                State::PartialHotkey,
                vec![Action::SendKeyEvent(key_release_sequence(
                    &self.previously_pressed,
                ))],
            ),
            (State::Idle, KeyEvent::Press(Match::Complete(action))) => {
                let mut send_actions = vec![Action::SendKeyEvent(key_release_sequence(
                    &self.previously_pressed,
                ))];
                match &action {
                    config::Action::KeyCombination(keys) => {
                        send_actions.push(Action::SendKeyEvent(key_press_sequence(&keys)));
                    }
                    config::Action::ModeChange(mode) => {
                        eprintln!("mode: {mode}");
                        self.current_mode = mode.clone()
                    }
                }

                (State::CompleteHotkey(action), send_actions)
            }
            (State::Idle, KeyEvent::Repeat) => (
                State::Idle,
                if self.is_modifier(key) {
                    Vec::new()
                } else {
                    vec![Action::SendKeyEvent(key_repeat_sequence(&vec![key]))]
                },
            ),
            (State::Idle, KeyEvent::Release) => (
                State::Idle,
                vec![Action::SendKeyEvent(key_release_sequence(&vec![key]))],
            ),

            (State::PartialHotkey, KeyEvent::Press(Match::Impossible)) => (
                State::Idle,
                vec![Action::SendKeyEvent(key_press_sequence(&self.now_pressed))],
            ),
            (State::PartialHotkey, KeyEvent::Press(Match::Possible)) => {
                (State::PartialHotkey, Vec::new())
            }
            (State::PartialHotkey, KeyEvent::Press(Match::Complete(action))) => {
                let send_actions = match &action {
                    config::Action::KeyCombination(keys) => {
                        vec![Action::SendKeyEvent(key_press_sequence(&keys))]
                    }
                    config::Action::ModeChange(_) => vec![],
                };

                (State::CompleteHotkey(action), send_actions)
            }
            (State::PartialHotkey, KeyEvent::Repeat) => (State::PartialHotkey, Vec::new()),
            (State::PartialHotkey, KeyEvent::Release) => {
                if self.now_pressed.is_empty() {
                    (State::Idle, Vec::new())
                } else {
                    (State::PartialHotkey, Vec::new())
                }
            }

            (State::CompleteHotkey(action), KeyEvent::Press(_)) => {
                (State::CompleteHotkey(action.clone()), Vec::new())
            }
            (State::CompleteHotkey(action), KeyEvent::Repeat) => {
                let send_actions = match action {
                    config::Action::KeyCombination(keys) => {
                        vec![Action::SendKeyEvent(key_repeat_sequence(
                            &keys
                                .iter()
                                .filter(|key| !self.is_modifier(**key))
                                .cloned()
                                .collect(),
                        ))]
                    }
                    config::Action::ModeChange(_) => vec![],
                };

                (State::CompleteHotkey(action.clone()), send_actions)
            }
            (State::CompleteHotkey(action), KeyEvent::Release) => {
                let send_actions = match action {
                    config::Action::KeyCombination(keys) => {
                        vec![Action::SendKeyEvent(key_release_sequence(&keys))]
                    }
                    config::Action::ModeChange(_) => vec![],
                };

                if self.now_pressed.is_empty() {
                    (State::Idle, send_actions)
                } else {
                    (State::PartialHotkey, send_actions)
                }
            }

            (state, KeyEvent::Nothing) => (state.clone(), Vec::new()),
        }
    }

    fn is_modifier(&self, key: Key) -> bool {
        self.hotkeys_by_mode
            .get(&self.current_mode)
            .and_then(|hotkeys| Some(hotkeys.is_modifier(key)))
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
struct HotkeyStore {
    hotkeys: HashMap<KeySet, config::Action>,
    modifiers: Vec<Key>,
}

type KeySet = BTreeSet<Key>;

#[derive(Debug, PartialEq, Eq)]
enum Match {
    Impossible,
    Possible,
    Complete(config::Action),
}

impl HotkeyStore {
    pub fn new(hotkeys: Vec<config::Hotkey>, modifiers: Vec<Key>) -> Self {
        Self {
            hotkeys: hotkeys
                .into_iter()
                .map(|m| (KeySet::from_iter(m.on), m.action))
                .collect(),
            modifiers,
        }
    }

    pub fn query(&self, chord: &KeySet) -> Match {
        for (trigger, action) in &self.hotkeys {
            if trigger == chord {
                return Match::Complete(action.clone());
            }
            // match is only still possible if there are only modifers pressed yet, otherwise it must be complete
            if trigger.is_superset(chord) && chord.iter().all(|key| self.modifiers.contains(key)) {
                return Match::Possible;
            }
        }
        Match::Impossible
    }

    pub fn is_modifier(&self, key: Key) -> bool {
        self.modifiers.contains(&key)
    }
}

#[cfg(test)]
mod test {

    use super::*;

    use crate::{
        config::{Action, Hotkey},
        keys::Key,
    };

    #[test]
    fn match_impossible_with_empty_config_when_modifier_is_pressed() {
        let sut = HotkeyStore::new(vec![], vec![]);

        let result = sut.query(&KeySet::from([Key::CtrlLeft]));

        assert_eq!(result, Match::Impossible);
    }

    #[test]
    fn match_impossible_with_empty_config_when_non_modifier_is_pressed() {
        let sut = HotkeyStore::new(vec![], vec![]);

        let result = sut.query(&KeySet::from([Key::A]));

        assert_eq!(result, Match::Impossible);
    }

    #[test]
    fn match_impossible_with_empty_config_when_nothing_is_pressed() {
        let sut = HotkeyStore::new(vec![], vec![]);

        let result = sut.query(&KeySet::from([]));

        assert_eq!(result, Match::Impossible);
    }

    #[test]
    fn match_impossible_when_uncofigured_combination_pressed() {
        let sut = HotkeyStore::new(
            vec![Hotkey {
                on: vec![Key::CtrlLeft, Key::ShiftLeft, Key::A],
                action: Action::KeyCombination(vec![Key::B]),
            }],
            vec![],
        );

        let result = sut.query(&KeySet::from([Key::AltLeft, Key::C]));

        assert_eq!(result, Match::Impossible);
    }

    #[test]
    fn match_impossible_when_non_modifier_pressed_and_not_complete() {
        let sut = HotkeyStore::new(
            vec![Hotkey {
                on: vec![Key::CtrlLeft, Key::ShiftLeft, Key::A],
                action: Action::KeyCombination(vec![Key::B]),
            }],
            vec![],
        );

        let result = sut.query(&KeySet::from([Key::CtrlLeft, Key::A]));

        assert_eq!(result, Match::Impossible);
    }

    #[test]
    fn match_impossible_when_wrong_modifier_pressed() {
        let sut = HotkeyStore::new(
            vec![Hotkey {
                on: vec![Key::CtrlLeft, Key::ShiftLeft, Key::A],
                action: Action::KeyCombination(vec![Key::B]),
            }],
            vec![],
        );

        let result = sut.query(&KeySet::from([Key::AltRight]));

        assert_eq!(result, Match::Impossible);
    }

    #[test]
    fn match_impossible_when_modifier_pressed_but_none_is_configured() {
        let sut = HotkeyStore::new(
            vec![Hotkey {
                on: vec![Key::A],
                action: Action::KeyCombination(vec![Key::B]),
            }],
            vec![],
        );

        let result = sut.query(&KeySet::from([Key::AltRight]));

        assert_eq!(result, Match::Impossible);
    }

    #[test]
    fn match_possible_when_nothing_is_pressed() {
        let sut = HotkeyStore::new(
            vec![Hotkey {
                on: vec![Key::CtrlLeft, Key::A],
                action: Action::KeyCombination(vec![Key::B]),
            }],
            vec![],
        );

        let result = sut.query(&KeySet::from([]));

        assert_eq!(result, Match::Possible);
    }

    #[test]
    fn match_possible_when_single_matching_modifier_is_pressed() {
        let sut = HotkeyStore::new(
            vec![Hotkey {
                on: vec![Key::CtrlLeft, Key::A],
                action: Action::KeyCombination(vec![Key::B]),
            }],
            vec![Key::CtrlLeft],
        );

        let result = sut.query(&KeySet::from([Key::CtrlLeft]));

        assert_eq!(result, Match::Possible);
    }

    #[test]
    fn match_possible_when_one_of_multiple_matching_modifiers_are_pressed() {
        let sut = HotkeyStore::new(
            vec![Hotkey {
                on: vec![Key::CtrlLeft, Key::ShiftLeft, Key::AltLeft, Key::A],
                action: Action::KeyCombination(vec![Key::B]),
            }],
            vec![Key::CtrlLeft, Key::ShiftLeft, Key::AltLeft],
        );

        let result = sut.query(&KeySet::from([Key::ShiftLeft]));

        assert_eq!(result, Match::Possible);
    }

    #[test]
    fn match_possible_when_some_of_multiple_matching_modifiers_are_pressed() {
        let sut = HotkeyStore::new(
            vec![Hotkey {
                on: vec![Key::CtrlLeft, Key::ShiftLeft, Key::AltLeft, Key::A],
                action: Action::KeyCombination(vec![Key::B]),
            }],
            vec![Key::CtrlLeft, Key::ShiftLeft, Key::AltLeft],
        );

        let result = sut.query(&KeySet::from([Key::ShiftLeft, Key::AltLeft]));

        assert_eq!(result, Match::Possible);
    }

    #[test]
    fn match_possible_when_all_of_multiple_matching_modifiers_are_pressed() {
        let sut = HotkeyStore::new(
            vec![Hotkey {
                on: vec![Key::CtrlLeft, Key::ShiftLeft, Key::AltLeft, Key::A],
                action: Action::KeyCombination(vec![Key::B]),
            }],
            vec![Key::CtrlLeft, Key::ShiftLeft, Key::AltLeft],
        );

        let result = sut.query(&KeySet::from([Key::CtrlLeft, Key::ShiftLeft, Key::AltLeft]));

        assert_eq!(result, Match::Possible);
    }

    #[test]
    fn match_complete_when_no_modifier_configured() {
        let sut = HotkeyStore::new(
            vec![Hotkey {
                on: vec![Key::A],
                action: Action::KeyCombination(vec![Key::B]),
            }],
            vec![Key::CtrlLeft],
        );

        let result = sut.query(&KeySet::from([Key::A]));

        assert_eq!(
            result,
            Match::Complete(Action::KeyCombination(vec![Key::B]))
        );
    }

    #[test]
    fn match_complete_with_single_modifier() {
        let sut = HotkeyStore::new(
            vec![Hotkey {
                on: vec![Key::CtrlLeft, Key::A],
                action: Action::KeyCombination(vec![Key::B]),
            }],
            vec![Key::CtrlLeft],
        );

        let result = sut.query(&KeySet::from([Key::CtrlLeft, Key::A]));

        assert_eq!(
            result,
            Match::Complete(Action::KeyCombination(vec![Key::B]))
        );
    }

    #[test]
    fn match_complete_with_multiple_modifiers() {
        let sut = HotkeyStore::new(
            vec![Hotkey {
                on: vec![Key::CtrlLeft, Key::AltLeft, Key::A],
                action: Action::KeyCombination(vec![Key::B]),
            }],
            vec![Key::CtrlLeft, Key::AltLeft],
        );

        let result = sut.query(&KeySet::from([Key::CtrlLeft, Key::AltLeft, Key::A]));

        assert_eq!(
            result,
            Match::Complete(Action::KeyCombination(vec![Key::B]))
        );
    }

    #[test]
    fn incremental_with_multiple_hotkeys_when_match_is_found() {
        let sut = HotkeyStore::new(
            vec![
                Hotkey {
                    on: vec![Key::CtrlLeft, Key::AltLeft, Key::A],
                    action: Action::KeyCombination(vec![Key::B]),
                },
                Hotkey {
                    on: vec![Key::D],
                    action: Action::KeyCombination(vec![Key::E]),
                },
                Hotkey {
                    on: vec![Key::AltLeft, Key::K],
                    action: Action::KeyCombination(vec![Key::CtrlLeft, Key::K]),
                },
                Hotkey {
                    on: vec![Key::CtrlLeft, Key::AltLeft, Key::C],
                    action: Action::KeyCombination(vec![Key::CtrlLeft, Key::V]),
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
            Match::Complete(Action::KeyCombination(vec![Key::CtrlLeft, Key::V]))
        );
    }

    #[test]
    fn incremental_with_multiple_hotkeys_when_no_match_is_found() {
        let sut = HotkeyStore::new(
            vec![
                Hotkey {
                    on: vec![Key::CtrlLeft, Key::AltLeft, Key::A],
                    action: Action::KeyCombination(vec![Key::B]),
                },
                Hotkey {
                    on: vec![Key::D],
                    action: Action::KeyCombination(vec![Key::E]),
                },
                Hotkey {
                    on: vec![Key::ShiftLeft, Key::K],
                    action: Action::KeyCombination(vec![Key::CtrlLeft, Key::K]),
                },
                Hotkey {
                    on: vec![Key::CtrlLeft, Key::ShiftLeft, Key::C],
                    action: Action::KeyCombination(vec![Key::CtrlLeft, Key::V]),
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

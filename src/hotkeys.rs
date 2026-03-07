use std::collections::{BTreeSet, HashMap};

use crate::{
    config, keys::Key
};

#[derive(Default)]
pub struct Hotkeys {
    mappings: HashMap<KeySet, config::Action>,
    modifiers: Vec<Key>,
}

pub type KeySet = BTreeSet<Key>;

#[derive(Debug, PartialEq, Eq)]
pub enum Match {
    Impossible,
    Possible,
    Complete(config::Action),
}

impl Hotkeys {
    pub fn new(mappings: Vec<config::Hotkey>, modifiers: Vec<Key>) -> Self {
        Self {
            mappings: mappings
                .into_iter()
                .map(|m| (KeySet::from_iter(m.on), m.send))
                .collect(),
            modifiers,
        }
    }

    pub fn query(&self, combination: &KeySet) -> Match {
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
mod test {
    use crate::{
        config::{Action, Hotkey},
        hotkeys::{Hotkeys, KeySet, Match},
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
            vec![Hotkey {
                on: vec![Key::CtrlLeft, Key::ShiftLeft, Key::A],
                send: Action::KeyCombination(vec![Key::B]),
            }],
            vec![],
        );

        let result = sut.query(&KeySet::from([Key::AltLeft, Key::C]));

        assert_eq!(result, Match::Impossible);
    }

    #[test]
    fn match_impossible_when_non_modifier_pressed_and_not_complete() {
        let sut = Hotkeys::new(
            vec![Hotkey {
                on: vec![Key::CtrlLeft, Key::ShiftLeft, Key::A],
                send: Action::KeyCombination(vec![Key::B]),
            }],
            vec![],
        );

        let result = sut.query(&KeySet::from([Key::CtrlLeft, Key::A]));

        assert_eq!(result, Match::Impossible);
    }

    #[test]
    fn match_impossible_when_wrong_modifier_pressed() {
        let sut = Hotkeys::new(
            vec![Hotkey {
                on: vec![Key::CtrlLeft, Key::ShiftLeft, Key::A],
                send: Action::KeyCombination(vec![Key::B]),
            }],
            vec![],
        );

        let result = sut.query(&KeySet::from([Key::AltRight]));

        assert_eq!(result, Match::Impossible);
    }

    #[test]
    fn match_impossible_when_modifier_pressed_but_none_is_configured() {
        let sut = Hotkeys::new(
            vec![Hotkey {
                on: vec![Key::A],
                send: Action::KeyCombination(vec![Key::B]),
            }],
            vec![],
        );

        let result = sut.query(&KeySet::from([Key::AltRight]));

        assert_eq!(result, Match::Impossible);
    }

    #[test]
    fn match_possible_when_nothing_is_pressed() {
        let sut = Hotkeys::new(
            vec![Hotkey {
                on: vec![Key::CtrlLeft, Key::A],
                send: Action::KeyCombination(vec![Key::B]),
            }],
            vec![],
        );

        let result = sut.query(&KeySet::from([]));

        assert_eq!(result, Match::Possible);
    }

    #[test]
    fn match_possible_when_single_matching_modifier_is_pressed() {
        let sut = Hotkeys::new(
            vec![Hotkey {
                on: vec![Key::CtrlLeft, Key::A],
                send: Action::KeyCombination(vec![Key::B]),
            }],
            vec![Key::CtrlLeft],
        );

        let result = sut.query(&KeySet::from([Key::CtrlLeft]));

        assert_eq!(result, Match::Possible);
    }

    #[test]
    fn match_possible_when_one_of_multiple_matching_modifiers_are_pressed() {
        let sut = Hotkeys::new(
            vec![Hotkey {
                on: vec![Key::CtrlLeft, Key::ShiftLeft, Key::AltLeft, Key::A],
                send: Action::KeyCombination(vec![Key::B]),
            }],
            vec![Key::CtrlLeft, Key::ShiftLeft, Key::AltLeft],
        );

        let result = sut.query(&KeySet::from([Key::ShiftLeft]));

        assert_eq!(result, Match::Possible);
    }

    #[test]
    fn match_possible_when_some_of_multiple_matching_modifiers_are_pressed() {
        let sut = Hotkeys::new(
            vec![Hotkey {
                on: vec![Key::CtrlLeft, Key::ShiftLeft, Key::AltLeft, Key::A],
                send: Action::KeyCombination(vec![Key::B]),
            }],
            vec![Key::CtrlLeft, Key::ShiftLeft, Key::AltLeft],
        );

        let result = sut.query(&KeySet::from([Key::ShiftLeft, Key::AltLeft]));

        assert_eq!(result, Match::Possible);
    }

    #[test]
    fn match_possible_when_all_of_multiple_matching_modifiers_are_pressed() {
        let sut = Hotkeys::new(
            vec![Hotkey {
                on: vec![Key::CtrlLeft, Key::ShiftLeft, Key::AltLeft, Key::A],
                send: Action::KeyCombination(vec![Key::B]),
            }],
            vec![Key::CtrlLeft, Key::ShiftLeft, Key::AltLeft],
        );

        let result = sut.query(&KeySet::from([Key::CtrlLeft, Key::ShiftLeft, Key::AltLeft]));

        assert_eq!(result, Match::Possible);
    }

    #[test]
    fn match_complete_when_no_modifier_configured() {
        let sut = Hotkeys::new(
            vec![Hotkey {
                on: vec![Key::A],
                send: Action::KeyCombination(vec![Key::B]),
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
        let sut = Hotkeys::new(
            vec![Hotkey {
                on: vec![Key::CtrlLeft, Key::A],
                send: Action::KeyCombination(vec![Key::B]),
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
        let sut = Hotkeys::new(
            vec![Hotkey {
                on: vec![Key::CtrlLeft, Key::AltLeft, Key::A],
                send: Action::KeyCombination(vec![Key::B]),
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
        let sut = Hotkeys::new(
            vec![
                Hotkey {
                    on: vec![Key::CtrlLeft, Key::AltLeft, Key::A],
                    send: Action::KeyCombination(vec![Key::B]),
                },
                Hotkey {
                    on: vec![Key::D],
                    send: Action::KeyCombination(vec![Key::E]),
                },
                Hotkey {
                    on: vec![Key::AltLeft, Key::K],
                    send: Action::KeyCombination(vec![Key::CtrlLeft, Key::K]),
                },
                Hotkey {
                    on: vec![Key::CtrlLeft, Key::AltLeft, Key::C],
                    send: Action::KeyCombination(vec![Key::CtrlLeft, Key::V]),
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
        let sut = Hotkeys::new(
            vec![
                Hotkey {
                    on: vec![Key::CtrlLeft, Key::AltLeft, Key::A],
                    send: Action::KeyCombination(vec![Key::B]),
                },
                Hotkey {
                    on: vec![Key::D],
                    send: Action::KeyCombination(vec![Key::E]),
                },
                Hotkey {
                    on: vec![Key::ShiftLeft, Key::K],
                    send: Action::KeyCombination(vec![Key::CtrlLeft, Key::K]),
                },
                Hotkey {
                    on: vec![Key::CtrlLeft, Key::ShiftLeft, Key::C],
                    send: Action::KeyCombination(vec![Key::CtrlLeft, Key::V]),
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

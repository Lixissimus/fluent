use pest::Parser;
use pest_derive::Parser;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::keys::Key;

#[derive(Parser)]
#[grammar = "config.pest"]
struct ConfigParser;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("configuration syntax error: {0}")]
    Syntax(#[from] pest::error::Error<Rule>),

    #[error("invalid key `{name}`: {source}")]
    InvalidKey {
        name: String,
        source: serde_plain::Error,
    },
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_modifiers")]
    pub modifiers: Vec<Key>,
    pub mappings: Vec<Mapping>,
}

pub fn parse(input: &str) -> Result<Config, ConfigError> {
    let file = ConfigParser::parse(Rule::file, input)?
        .next()
        .expect("the file rule always produces one pair");

    let mut config = Config::default();

    for statement in file.into_inner() {
        match statement.as_rule() {
            Rule::modifiers => {
                let modifiers_list = statement
                    .into_inner()
                    .find(|pair| pair.as_rule() == Rule::list)
                    .expect("modifiers always contain a list");
                config.modifiers = parse_list(modifiers_list)?;
            }
            Rule::bind => {
                let mut lists = statement
                    .into_inner()
                    .filter(|pair| pair.as_rule() == Rule::list);
                let on = parse_list(lists.next().expect("bind always has an input list"))?;
                let send = parse_list(lists.next().expect("bind always has an output list"))?;
                config.mappings.push(Mapping { on, send });
            }
            _ => {}
        }
    }

    Ok(config)
}

fn parse_list(pair: pest::iterators::Pair<'_, Rule>) -> Result<Vec<Key>, ConfigError> {
    pair.into_inner()
        .filter(|item| item.as_rule() == Rule::identifier)
        .map(|item| {
            let name = item.as_str().to_owned();
            serde_plain::from_str(&name).map_err(|source| ConfigError::InvalidKey { name, source })
        })
        .collect()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            modifiers: default_modifiers(),
            mappings: Default::default(),
        }
    }
}

fn default_modifiers() -> Vec<Key> {
    let keys = vec![
        Key::CtrlLeft,
        Key::CtrlRight,
        Key::AltLeft,
        Key::AltRight,
        Key::ShiftLeft,
        Key::ShiftRight,
    ];
    keys
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mapping {
    pub on: Vec<Key>,
    pub send: Vec<Key>,
}

#[cfg(test)]
mod tests {
    use super::{ConfigError, parse};
    use crate::keys::Key;

    #[test]
    fn parses_modifiers_and_bindings() {
        let config = parse(
            "modifiers = ctrl_left, alt_left, capslock\n\
             bind = capslock, j -> left\n\
             bind = alt_left, capslock, j -> shift_left, left\n",
        )
        .unwrap();

        assert_eq!(
            config.modifiers,
            vec![Key::CtrlLeft, Key::AltLeft, Key::Capslock]
        );
        assert_eq!(config.mappings.len(), 2);
        assert_eq!(config.mappings[0].on, vec![Key::Capslock, Key::J]);
        assert_eq!(config.mappings[0].send, vec![Key::ArrowLeft]);
        assert_eq!(
            config.mappings[1].on,
            vec![Key::AltLeft, Key::Capslock, Key::J]
        );
        assert_eq!(
            config.mappings[1].send,
            vec![Key::ShiftLeft, Key::ArrowLeft]
        );
    }

    #[test]
    fn parses_multiline_commands() {
        let config = parse(
            "modifiers = ctrl_left,\n\
             alt_left,\n\
             capslock\n\
             bind =\n\n\n\
             capslock,\n\
             j\n\
             ->\n\
             left,\n\
             up\n\
             bind = capslock, k -> down\n",
        )
        .unwrap();

        assert_eq!(
            config.modifiers,
            vec![Key::CtrlLeft, Key::AltLeft, Key::Capslock]
        );
        assert_eq!(config.mappings.len(), 2);
        assert_eq!(config.mappings[0].on, vec![Key::Capslock, Key::J]);
        assert_eq!(config.mappings[0].send, vec![Key::ArrowLeft, Key::ArrowUp]);
        assert_eq!(config.mappings[1].on, vec![Key::Capslock, Key::K]);
        assert_eq!(config.mappings[1].send, vec![Key::ArrowDown]);
    }

    #[test]
    fn uses_default_modifiers_when_omitted() {
        let config = parse("bind = capslock, j -> left\n").unwrap();

        assert_eq!(
            config.modifiers,
            vec![
                Key::CtrlLeft,
                Key::CtrlRight,
                Key::AltLeft,
                Key::AltRight,
                Key::ShiftLeft,
                Key::ShiftRight
            ]
        );
        assert_eq!(config.mappings.len(), 1);
    }

    #[test]
    fn reports_unknown_keys() {
        assert!(matches!(
            parse("bind = capslock, unknown_key -> left\n"),
            Err(ConfigError::InvalidKey { .. })
        ));
    }

    #[test]
    fn reports_syntax_errors() {
        assert!(matches!(
            parse("bind => capslock, j -> left\n"),
            Err(ConfigError::Syntax(_))
        ));
    }
}

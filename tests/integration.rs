use fluent::{
    config::{Config, Mapping},
    keys::Key,
};
use input_event_codes::{KEY_A, KEY_B, KEY_LEFTCTRL};

use crate::common::InputEvent;

mod common;

#[test]
fn pass_through_unmapped_key_events() {
    let (mut input, mut output) = common::create_event_streams(&[
        InputEvent::key_press(KEY_A!()),
        InputEvent::key_repeat(KEY_A!()),
        InputEvent::key_release(KEY_A!()),
    ]);

    let _ = fluent::run(&mut input, &mut output, &Config::default());

    let output_events = output.extract_events();
    assert_eq!(output_events.len(), 3);
    assert_eq!(output_events[0], InputEvent::key_press(KEY_A!()));
    assert_eq!(output_events[1], InputEvent::key_repeat(KEY_A!()));
    assert_eq!(output_events[2], InputEvent::key_release(KEY_A!()));
}

#[test]
fn remap_single_key_events() {
    let (mut input, mut output) = common::create_event_streams(&[
        InputEvent::key_press(KEY_A!()),
        InputEvent::key_repeat(KEY_A!()),
        InputEvent::key_release(KEY_A!()),
    ]);

    let _ = fluent::run(
        &mut input,
        &mut output,
        &Config {
            mappings: vec![Mapping {
                on: vec![Key::A],
                send: vec![Key::B],
            }],
        },
    );

    let output_events = output.extract_events();
    assert_eq!(output_events.len(), 3);
    assert_eq!(output_events[0], InputEvent::key_press(KEY_B!()));
    assert_eq!(output_events[1], InputEvent::key_repeat(KEY_B!()));
    assert_eq!(output_events[2], InputEvent::key_release(KEY_B!()));
}

#[test]
fn press_and_release_once_with_single_modifier() {
    let (mut input, mut output) = common::create_event_streams(&[
        InputEvent::key_press(KEY_LEFTCTRL!()),
        InputEvent::key_press(KEY_A!()),
        InputEvent::key_release(KEY_A!()),
        InputEvent::key_release(KEY_LEFTCTRL!()),
    ]);

    let _ = fluent::run(
        &mut input,
        &mut output,
        &Config {
            mappings: vec![Mapping {
                on: vec![Key::CtrlLeft, Key::A],
                send: vec![Key::B],
            }],
        },
    );

    let output_events = output.extract_events();
    assert_eq!(output_events.len(), 2);
    assert_eq!(output_events[0], InputEvent::key_press(KEY_B!()));
    assert_eq!(output_events[1], InputEvent::key_release(KEY_B!()));
}

#[test]
fn press_and_release_modifier_first_once_with_single_modifier() {
    let (mut input, mut output) = common::create_event_streams(&[
        InputEvent::key_press(KEY_LEFTCTRL!()),
        InputEvent::key_press(KEY_A!()),
        InputEvent::key_release(KEY_LEFTCTRL!()),
        InputEvent::key_release(KEY_A!()),
    ]);

    let _ = fluent::run(
        &mut input,
        &mut output,
        &Config {
            mappings: vec![Mapping {
                on: vec![Key::CtrlLeft, Key::A],
                send: vec![Key::B],
            }],
        },
    );

    let output_events = output.extract_events();
    assert_eq!(output_events.len(), 2);
    assert_eq!(output_events[0], InputEvent::key_press(KEY_B!()));
    assert_eq!(output_events[1], InputEvent::key_release(KEY_B!()));
}

#[test]
fn press_repeat_and_release_with_single_modifier() {
    let (mut input, mut output) = common::create_event_streams(&[
        InputEvent::key_press(KEY_LEFTCTRL!()),
        InputEvent::key_press(KEY_A!()),
        InputEvent::key_repeat(KEY_A!()),
        InputEvent::key_repeat(KEY_A!()),
        InputEvent::key_release(KEY_A!()),
        InputEvent::key_release(KEY_LEFTCTRL!()),
    ]);

    let _ = fluent::run(
        &mut input,
        &mut output,
        &Config {
            mappings: vec![Mapping {
                on: vec![Key::CtrlLeft, Key::A],
                send: vec![Key::B],
            }],
        },
    );

    let output_events = output.extract_events();
    assert_eq!(output_events.len(), 4);
    assert_eq!(output_events[0], InputEvent::key_press(KEY_B!()));
    assert_eq!(output_events[1], InputEvent::key_repeat(KEY_B!()));
    assert_eq!(output_events[2], InputEvent::key_repeat(KEY_B!()));
    assert_eq!(output_events[3], InputEvent::key_release(KEY_B!()));
}

#[test]
fn press_repeat_and_release_modifier_first_with_single_modifier() {
    let (mut input, mut output) = common::create_event_streams(&[
        InputEvent::key_press(KEY_LEFTCTRL!()),
        InputEvent::key_press(KEY_A!()),
        InputEvent::key_repeat(KEY_A!()),
        InputEvent::key_repeat(KEY_A!()),
        InputEvent::key_release(KEY_LEFTCTRL!()),
        InputEvent::key_release(KEY_A!()),
    ]);

    let _ = fluent::run(
        &mut input,
        &mut output,
        &Config {
            mappings: vec![Mapping {
                on: vec![Key::CtrlLeft, Key::A],
                send: vec![Key::B],
            }],
        },
    );

    let output_events = output.extract_events();
    assert_eq!(output_events.len(), 4);
    assert_eq!(output_events[0], InputEvent::key_press(KEY_B!()));
    assert_eq!(output_events[1], InputEvent::key_repeat(KEY_B!()));
    assert_eq!(output_events[2], InputEvent::key_repeat(KEY_B!()));
    assert_eq!(output_events[3], InputEvent::key_release(KEY_B!()));
}

#[test]
fn press_and_release_twice_with_single_modifier() {
    let (mut input, mut output) = common::create_event_streams(&[
        InputEvent::key_press(KEY_LEFTCTRL!()),
        InputEvent::key_press(KEY_A!()),
        InputEvent::key_release(KEY_A!()),
        InputEvent::key_press(KEY_A!()),
        InputEvent::key_release(KEY_A!()),
        InputEvent::key_release(KEY_LEFTCTRL!()),
    ]);

    let _ = fluent::run(
        &mut input,
        &mut output,
        &Config {
            mappings: vec![Mapping {
                on: vec![Key::CtrlLeft, Key::A],
                send: vec![Key::B],
            }],
        },
    );

    let output_events = output.extract_events();
    assert_eq!(output_events.len(), 4);
    assert_eq!(output_events[0], InputEvent::key_press(KEY_B!()));
    assert_eq!(output_events[1], InputEvent::key_release(KEY_B!()));
    assert_eq!(output_events[2], InputEvent::key_press(KEY_B!()));
    assert_eq!(output_events[3], InputEvent::key_release(KEY_B!()));
}

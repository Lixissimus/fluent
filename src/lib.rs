use anyhow::Context;
use std::{
    io::{Read, Write},
    thread,
    time::Duration,
};

use crate::{
    config::Config,
    engine::Engine,
    event::{EventBuffer, EventType, InputEvent},
};

pub mod config;
pub mod keys;

mod engine;
mod event;

pub fn run<I: Read, O: Write>(
    input: &mut I,
    output: &mut O,
    config: &Config,
) -> anyhow::Result<()> {
    let mut input_buffer = EventBuffer::default();
    let mut engine = Engine::new(config);
    loop {
        input
            .read_exact(input_buffer.raw_mut())
            .context("could not read input event")?;

        let Ok(evt) = InputEvent::try_from(&input_buffer) else {
            eprintln!("could not parse input: {:?}", input_buffer);
            continue;
        };
        if evt.r#type != EventType::Key {
            print_event(output, &evt).context("could not forward non-key event")?;
            continue;
        }

        for output_event in engine.handle(evt) {
            print_event(output, &output_event).context("could not send event")?;
        }
    }
}

fn print_event<O: Write>(output: &mut O, evt: &InputEvent) -> anyhow::Result<()> {
    // it is recommended to not send multiple events at the same time, therefore sleep a tiny bit between events
    thread::sleep(Duration::from_micros(1));

    let buffer: EventBuffer = evt.into();
    output
        .write_all(buffer.raw())
        .context("error writing to stdout")?;
    output.flush().context("error flushing stdout")?;
    Ok(())
}

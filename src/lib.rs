use anyhow::Context;
use std::io::{Read, Write};

use crate::{
    engine::Engine,
    event::{EventBuffer, EventType, InputEvent},
};

mod engine;
mod event;
mod keys;

pub fn run<I: Read, O: Write>(input: &mut I, output: &mut O) -> anyhow::Result<()> {
    let mut input_buffer = EventBuffer::default();
    let mut engine = Engine::new();
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

        eprintln!("IN:  {:?}", evt);

        for output_event in engine.handle(evt) {
            eprintln!("OUT: {:?}", output_event);
            print_event(output, &output_event).context("could not send event")?;
        }
    }
}

fn print_event<O: Write>(output: &mut O, evt: &InputEvent) -> anyhow::Result<()> {
    let buffer: EventBuffer = evt.into();
    output
        .write_all(buffer.raw())
        .context("error writing to stdout")?;
    output.flush().context("error flushing stdout")?;
    Ok(())
}

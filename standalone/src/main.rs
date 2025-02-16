use std::process::Command;
use std::{
    io::{self, stdout},
    time::{Duration, Instant},
};

use grainiac_core::Sampler;
use jack::{AudioIn, AudioOut, Client, ClientOptions, MidiIn, Port};
use ratatui::crossterm::{
    event::{KeyboardEnhancementFlags, PushKeyboardEnhancementFlags},
    execute,
};

mod state;
mod ui;
mod waveform_widget;

fn main() -> io::Result<()> {
    let (client, _status) = Client::new("grainiac", ClientOptions::default()).unwrap();

    let out_port_l = client
        .register_port("output_l", AudioOut::default())
        .unwrap();

    let out_port_r = client
        .register_port("output_r", AudioOut::default())
        .unwrap();

    let input_port_l = client.register_port("input_l", AudioIn::default()).unwrap();

    let input_port_r = client.register_port("input_r", AudioIn::default()).unwrap();

    let midi_in_port = client.register_port("midi_in", MidiIn::default()).unwrap();

    let (sampler, out_buf) = Sampler::new(48000.0);

    struct Ports {
        input_l: Port<AudioIn>,
        input_r: Port<AudioIn>,
        output_l: Port<AudioOut>,
        output_r: Port<AudioOut>,
        midi_in: Port<MidiIn>,
        sampler: grainiac_core::Sampler,
    }

    let process = jack::contrib::ClosureProcessHandler::with_state(
        Ports {
            input_l: input_port_l,
            input_r: input_port_r,
            output_l: out_port_l,
            output_r: out_port_r,
            midi_in: midi_in_port,
            sampler,
        },
        |state, _, ps| -> jack::Control {
            let output_l = state.output_l.as_mut_slice(ps);
            let output_r = state.output_r.as_mut_slice(ps);
            let input_l = state.input_l.as_slice(ps);
            let input_r = state.input_r.as_slice(ps);

            let midi = state.midi_in.iter(ps);

            for event in midi {
                //println!("{:?}", event.bytes);
                let (message_type, midi_channel) = parse_status_byte(event.bytes[0]);
                match message_type {
                    9 => state.sampler.note_on(event.bytes[1] as usize),
                    8 => state.sampler.note_off(event.bytes[1] as usize),
                    11 => handle_midi_cc(
                        event.bytes[1],
                        event.bytes[2],
                        midi_channel as usize,
                        &mut state.sampler,
                    ),
                    _ => {} //println!("{:?}", event.bytes),
                }
            }

            output_l.copy_from_slice(input_l);
            output_r.copy_from_slice(input_r);

            for (sample_l, sample_r) in output_l.iter_mut().zip(output_r.iter_mut()) {
                state.sampler.render((sample_l, sample_r));
            }
            jack::Control::Continue
        },
        move |_, _, _| jack::Control::Continue,
    );

    let active_client = client.activate_async((), process).unwrap();
    active_client
        .as_client()
        .connect_ports_by_name("grainiac:output_l", "system:playback_1")
        .unwrap();
    active_client
        .as_client()
        .connect_ports_by_name("grainiac:output_r", "system:playback_2")
        .unwrap();
    active_client
        .as_client()
        .connect_ports_by_name("system:capture_1", "grainiac:input_l")
        .unwrap();
    active_client
        .as_client()
        .connect_ports_by_name("system:capture_1", "grainiac:input_r")
        .unwrap();

    let mut cmd = Command::new("bash");
    cmd.arg("./connect.sh");
    cmd.output().expect("failed to execute command");

    let mut state = state::State::new(out_buf);
    let mut terminal = ratatui::init();
    let mut stdout = stdout();

    execute!(
        stdout,
        PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES)
    )?;

    let tick_rate = Duration::from_millis(60);
    let mut last_tick = Instant::now();

    while !state.exiting {
        state.handle_event(1)?;
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();

            terminal.draw(|f| ui::draw(f, &mut state))?;
        }
    }

    ratatui::restore();

    Ok(())
}

fn handle_midi_cc(cc: u8, val: u8, instance: usize, sampler: &mut Sampler) {
    let value = val as f32 / 126.0;

    match cc {
        1 => {
            sampler.set_loop_start(instance, value);
        }
        2 => {
            sampler.set_loop_length(instance, value);
        }
        3 => {
            sampler.set_density(instance, value * 50.0);
        }
        4 => {
            sampler.set_grain_length(instance, value);
        }
        5 => {
            sampler.set_play_speed(instance, value * 2.0);
        }
        6 => {
            sampler.set_spray(instance, value);
        }
        7 => {
            sampler.set_pan(instance, (value * 2.0) - 1.0);
        }
        8 => {
            sampler.set_spread(instance, value);
        }
        9 => {
            sampler.set_attack(instance, value * 5.0);
        }
        10 => {
            sampler.set_release(instance, value * 5.0);
        }
        11 => {
            sampler.set_global_pitch(instance, (value * 24.0) as i8 - 12);
        }
        12 => {
            sampler.set_gain(instance, value);
        }
        13 => {
            if value > 0.0 {
                sampler.record(instance);
            }
        }
        14 => {
            if value > 0.0 {
                sampler.toggle_hold(instance);
            }
        }
        15 => {
            if value > 0.0 {
                sampler.toggle_play_dir(instance);
            }
        }
        16 => {
            if value > 0.0 {
                sampler.toggle_grain_dir(instance);
            }
        }
        _ => {} //println!("{:?}", event.bytes),
                //
    }
}

fn parse_status_byte(status: u8) -> (u8, u8) {
    let message_type = (status & 0xF0) >> 4; // Upper 4 bits
    let channel = status & 0x0F; // Lower 4 bits
    (message_type, channel)
}

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
                match event.bytes[0] {
                    144 => state.sampler.note_on(event.bytes[1] as usize),
                    128 => state.sampler.note_off(event.bytes[1] as usize),
                    176 => match event.bytes[1] {
                        23 => {
                            let value = event.bytes[2] as f32 / 127.0;
                            state.sampler.set_loop_start(0, value);
                        }
                        24 => {
                            let value = event.bytes[2] as f32 / 127.0;
                            state.sampler.set_loop_length(0, value);
                        }
                        25 => {
                            let value = event.bytes[2] as f32 / 127.0;
                            state.sampler.set_global_pitch(0, value * 1.5 + 0.5);
                        }
                        26 => {
                            let value = event.bytes[2] as f32 / 127.0;
                            state.sampler.set_play_speed(0, value * 2.0);
                        }
                        27 => {
                            if event.bytes[2] > 0 {
                                state.sampler.record(0);
                            }
                        }
                        28 => {
                            if event.bytes[2] > 0 {
                                state.sampler.toggle_hold(0);
                            }
                        }
                        29 => {
                            if event.bytes[2] > 0 {
                                state.sampler.toggle_play_dir(0);
                            }
                        }
                        // 30 => {
                        //     if event.bytes[2] > 0 {
                        //         state.sampler.record(3);
                        //     }
                        // }
                        _ => println!("{:?}", event.bytes),
                    },
                    _ => {} // println!("{:?}", event.bytes),
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
    // active_client
    //     .as_client()
    //     .connect_ports_by_name("grainiac:output_l", "system:playback_1")
    //     .unwrap();
    // active_client
    //     .as_client()
    //     .connect_ports_by_name("grainiac:output_r", "system:playback_2")
    //     .unwrap();
    // active_client
    //     .as_client()
    //     .connect_ports_by_name("grainiac:input_l", "system:capture_1")
    //     .unwrap();
    // active_client
    //     .as_client()
    //     .connect_ports_by_name("grainiac:input_r", "system:capture_2")
    //     .unwrap();

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

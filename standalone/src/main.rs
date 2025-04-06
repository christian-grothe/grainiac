use crossbeam::channel::{unbounded, Receiver};
use serde::{Deserialize, Serialize};

use std::env;
use std::fs::File;
use std::io::BufReader;
use std::process::Command;
use std::{
    io::{self, stdout},
    time::{Duration, Instant},
};

use grainiac_core::Sampler;
use jack::{AudioIn, AudioOut, Client, ClientOptions, MidiIn, Port};
use ratatui::crossterm::{
    event::{KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags},
    execute,
};

mod state;
mod ui;
mod waveform_widget;

#[derive(Deserialize, Serialize, Debug, Clone)]
struct Config {
    presets: Vec<Preset>,
    mapping: Mapping,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[allow(dead_code)]
pub struct Preset {
    gain: [f32; 4],
    loop_start: [f32; 4],
    loop_length: [f32; 4],
    density: [f32; 4],
    grain_length: [f32; 4],
    play_speed: [f32; 4],
    spray: [f32; 4],
    pan: [f32; 4],
    spread: [f32; 4],
    attack: [f32; 4],
    release: [f32; 4],
    pitch: [i8; 4],
    play_dir: [u8; 4],
    grain_dir: [u8; 4],
    name: String,
    char: char,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
struct Mapping {
    loop_start: u8,
    loop_length: u8,
    density: u8,
    grain_length: u8,
    play_speed: u8,
    spray: u8,
    pan: u8,
    spread: u8,
    attack: u8,
    release: u8,
    pitch: u8,
    gain: u8,
    record: u8,
    hold: u8,
    play_dir: u8,
    grain_dir: u8,
}

pub enum Msg {
    ApplyPreset(Preset),
    SaveAudio,
}

fn main() -> io::Result<()> {
    let (s, r) = unbounded();

    #[allow(deprecated)]
    let home_dir = env::home_dir().unwrap();
    let config_file_path = home_dir.join(".config/grainiac/config.json");
    let file = File::open(config_file_path).unwrap();
    let reader = BufReader::new(file);

    let config: Config = serde_json::from_reader(reader).expect("could not open json");

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

    let sr = client.sample_rate() as f32;
    let (sampler, out_buf) = Sampler::new(sr);

    struct State {
        input_l: Port<AudioIn>,
        input_r: Port<AudioIn>,
        output_l: Port<AudioOut>,
        output_r: Port<AudioOut>,
        midi_in: Port<MidiIn>,
        sampler: grainiac_core::Sampler,
        receiver: Receiver<Msg>,
        config: Config,
    }

    let process = jack::contrib::ClosureProcessHandler::with_state(
        State {
            input_l: input_port_l,
            input_r: input_port_r,
            output_l: out_port_l,
            output_r: out_port_r,
            midi_in: midi_in_port,
            sampler,
            receiver: r,
            config: config.clone(),
        },
        |state, _, ps| -> jack::Control {
            let output_l = state.output_l.as_mut_slice(ps);
            let output_r = state.output_r.as_mut_slice(ps);
            let input_l = state.input_l.as_slice(ps);
            let input_r = state.input_r.as_slice(ps);

            let midi = state.midi_in.iter(ps);

            if let Ok(msg) = state.receiver.try_recv() {
                match msg {
                    Msg::ApplyPreset(preset) => {
                        for (i, v) in preset.gain.iter().enumerate() {
                            state.sampler.set_gain(i, *v);
                        }

                        for (i, v) in preset.loop_start.iter().enumerate() {
                            state.sampler.set_loop_start(i, *v);
                        }

                        for (i, v) in preset.loop_length.iter().enumerate() {
                            state.sampler.set_loop_length(i, *v);
                        }

                        for (i, v) in preset.loop_length.iter().enumerate() {
                            state.sampler.set_loop_length(i, *v);
                        }

                        for (i, v) in preset.play_speed.iter().enumerate() {
                            state.sampler.set_play_speed(i, *v);
                        }

                        for (i, v) in preset.spray.iter().enumerate() {
                            state.sampler.set_spray(i, *v);
                        }

                        for (i, v) in preset.pan.iter().enumerate() {
                            state.sampler.set_pan(i, *v);
                        }

                        for (i, v) in preset.spread.iter().enumerate() {
                            state.sampler.set_spread(i, *v);
                        }

                        for (i, v) in preset.attack.iter().enumerate() {
                            state.sampler.set_attack(i, *v);
                        }

                        for (i, v) in preset.release.iter().enumerate() {
                            state.sampler.set_release(i, *v);
                        }

                        for (i, v) in preset.pitch.iter().enumerate() {
                            state.sampler.set_global_pitch(i, *v);
                        }

                        for (i, v) in preset.play_dir.iter().enumerate() {
                            state.sampler.set_play_dir_from_preset(i, *v);
                        }

                        for (i, v) in preset.grain_dir.iter().enumerate() {
                            state.sampler.set_grain_dir_from_preset(i, *v);
                        }
                    }
                    Msg::SaveAudio => {
                        let spec = hound::WavSpec {
                            channels: 1,
                            sample_rate: 48000,
                            bits_per_sample: 16,
                            sample_format: hound::SampleFormat::Int,
                        };
                        let mut writer = hound::WavWriter::create("test.wav", spec).unwrap();
                        let bufs = state.sampler.get_bufs();
                        let amplitude = i16::MAX as f32;
                        for buf in bufs.iter() {
                            for sample in buf.iter() {
                                writer.write_sample((sample * amplitude) as i16).unwrap();
                            }
                        }
                    }
                }
            }

            for event in midi {
                let (message_type, midi_channel) = parse_status_byte(event.bytes[0]);
                match message_type {
                    9 => state.sampler.note_on(event.bytes[1] as usize),
                    8 => state.sampler.note_off(event.bytes[1] as usize),
                    11 => handle_midi_cc(
                        event.bytes[1],
                        event.bytes[2],
                        midi_channel as usize,
                        &mut state.sampler,
                        &state.config.mapping,
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
        .unwrap_or_default();
    active_client
        .as_client()
        .connect_ports_by_name("grainiac:output_r", "system:playback_2")
        .unwrap_or_default();
    active_client
        .as_client()
        .connect_ports_by_name("system:capture_1", "grainiac:input_l")
        .unwrap_or_default();
    active_client
        .as_client()
        .connect_ports_by_name("system:capture_1", "grainiac:input_r")
        .unwrap_or_default();

    let mut cmd = Command::new("bash");
    cmd.arg("./connect.sh");
    cmd.output().expect("failed to execute command");

    let mut state = state::State::new(out_buf, s.clone(), config.presets);
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
    execute!(stdout, PopKeyboardEnhancementFlags);

    Ok(())
}

fn handle_midi_cc(cc: u8, val: u8, instance: usize, sampler: &mut Sampler, mapping: &Mapping) {
    let value = val as f32 / 126.0;

    match cc {
        x if x == mapping.loop_start => {
            sampler.set_loop_start(instance, value);
        }
        x if x == mapping.loop_length => {
            sampler.set_loop_length(instance, value);
        }
        x if x == mapping.density => {
            sampler.set_density(instance, value * 50.0);
        }
        x if x == mapping.grain_length => {
            sampler.set_grain_length(instance, value);
        }
        x if x == mapping.play_speed => {
            sampler.set_play_speed(instance, value * 2.0);
        }
        x if x == mapping.spray => {
            sampler.set_spray(instance, value);
        }
        x if x == mapping.pan => {
            sampler.set_pan(instance, (value * 2.0) - 1.0);
        }
        x if x == mapping.spread => {
            sampler.set_spread(instance, value);
        }
        x if x == mapping.attack => {
            sampler.set_attack(instance, value * 5.0);
        }
        x if x == mapping.release => {
            sampler.set_release(instance, value * 5.0);
        }
        x if x == mapping.pitch => {
            sampler.set_global_pitch(instance, (value * 24.0) as i8 - 12);
        }
        x if x == mapping.gain => {
            sampler.set_gain(instance, value);
        }
        x if x == mapping.record => {
            if value > 0.0 {
                sampler.record(instance);
            }
        }
        x if x == mapping.hold => {
            if value > 0.0 {
                sampler.toggle_hold(instance);
            }
        }
        x if x == mapping.play_dir => {
            if value > 0.0 {
                sampler.toggle_play_dir(instance);
            }
        }
        x if x == mapping.grain_dir => {
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

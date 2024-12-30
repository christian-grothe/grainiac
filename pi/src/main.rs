use std::error::Error;
use std::io::{stdin, stdout, Write};

use midir::{Ignore, MidiInput};

use grainiac_core::Sampler;
use rtaudio::{Api, Buffers, DeviceParams, SampleFormat, StreamInfo, StreamOptions, StreamStatus};

fn main() {
    let host = rtaudio::Host::new(Api::Unspecified).unwrap();
    let out_device = host.default_output_device().unwrap();
    let input_device = host.default_input_device().unwrap();
    let mut sampler = Sampler::new(48000.0);

    let mut stream_handle = host
        .open_stream(
            Some(DeviceParams {
                device_id: out_device.id,
                num_channels: 2,
                first_channel: 0,
            }),
            Some(DeviceParams {
                device_id: input_device.id,
                num_channels: 2,
                first_channel: 0,
            }),
            SampleFormat::Float32,
            out_device.preferred_sample_rate,
            256,
            StreamOptions::default(),
            |error| eprintln!("{}", error),
        )
        .unwrap();

    stream_handle
        .start(
            move |buffers: Buffers<'_>, _info: &StreamInfo, _status: StreamStatus| {
                if let Buffers::Float32 { output, input } = buffers {
                    output.copy_from_slice(input);
                    // By default, buffers are interleaved.
                    for frame in output.chunks_mut(2) {
                        let mut sample_channels = frame.into_iter();
                        let stereo_slice = (
                            sample_channels.next().unwrap(),
                            sample_channels.next().unwrap(),
                        );
                        sampler.render(stereo_slice);
                    }
                }
            },
        )
        .unwrap();

    let _ = run();

    loop {
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut input = String::new();

    let mut midi_in = MidiInput::new("midir reading input")?;
    midi_in.ignore(Ignore::None);

    // Get an input port (read from console if multiple are available)
    let in_ports = midi_in.ports();
    let in_port = match in_ports.len() {
        0 => return Err("no input port found".into()),
        1 => {
            println!(
                "Choosing the only available input port: {}",
                midi_in.port_name(&in_ports[0]).unwrap()
            );
            &in_ports[0]
        }
        _ => {
            println!("\nAvailable input ports:");
            for (i, p) in in_ports.iter().enumerate() {
                println!("{}: {}", i, midi_in.port_name(p).unwrap());
            }
            print!("Please select input port: ");
            stdout().flush()?;
            let mut input = String::new();
            stdin().read_line(&mut input)?;
            in_ports
                .get(input.trim().parse::<usize>()?)
                .ok_or("invalid input port selected")?
        }
    };

    println!("\nOpening connection");
    let in_port_name = midi_in.port_name(in_port)?;

    // _conn_in needs to be a named parameter, because it needs to be kept alive until the end of the scope
    let _conn_in = midi_in.connect(
        in_port,
        "midir-read-input",
        move |stamp, message, _| {
            println!("{}: {:?} (len = {})", stamp, message, message.len());
        },
        (),
    )?;

    println!(
        "Connection open, reading input from '{}' (press enter to exit) ...",
        in_port_name
    );

    input.clear();
    stdin().read_line(&mut input)?; // wait for next enter key press

    println!("Closing connection");
    Ok(())
}

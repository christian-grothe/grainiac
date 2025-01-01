fn main() {
    let (client, _status) = jack::Client::new("grainiac", jack::ClientOptions::default()).unwrap();

    let out_port_l = client
        .register_port("output_l", jack::AudioOut::default())
        .unwrap();

    let out_port_r = client
        .register_port("output_r", jack::AudioOut::default())
        .unwrap();

    let input_port_l = client
        .register_port("input_l", jack::AudioIn::default())
        .unwrap();

    let input_port_r = client
        .register_port("input_r", jack::AudioIn::default())
        .unwrap();

    let midi_in_port = client
        .register_port("midi_in", jack::MidiIn::default())
        .unwrap();

    let sampler = grainiac_core::Sampler::new(48000.0);

    struct Ports {
        input_l: jack::Port<jack::AudioIn>,
        input_r: jack::Port<jack::AudioIn>,
        output_l: jack::Port<jack::AudioOut>,
        output_r: jack::Port<jack::AudioOut>,
        midi_in: jack::Port<jack::MidiIn>,
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
                        22 => {
                            if event.bytes[2] > 0 {
                                state.sampler.record(0);
                            }
                        }
                        23 => {
                            if event.bytes[2] > 0 {
                                state.sampler.record(1);
                            }
                        }
                        24 => {
                            if event.bytes[2] > 0 {
                                state.sampler.record(2);
                            }
                        }
                        25 => {
                            if event.bytes[2] > 0 {
                                state.sampler.record(3);
                            }
                        }
                        _ => println!("{:?}", event.bytes),
                    },
                    _ => println!("{:?}", event.bytes),
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
    // active_client
    //     .as_client()
    //     .connect_ports_by_name("grainiac:input_l", "system:capture_1")
    //     .unwrap();
    // active_client
    //     .as_client()
    //     .connect_ports_by_name("grainiac:input_r", "system:capture_2")
    //     .unwrap();

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

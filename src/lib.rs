use nih_plug::prelude::*;
use nih_plug_vizia::ViziaState;
use std::sync::Arc;

mod editor;
mod sampler;

// This is a shortened version of the gain example with most comments removed, check out
// https://github.com/robbert-vdh/nih-plug/blob/master/plugins/examples/gain/src/lib.rs to get
// started

pub struct Grainiac {
    params: Arc<GrainiacParams>,
    sampler: sampler::Sampler,
}

#[derive(Params)]
struct GrainiacParams {
    #[persist = "editor-state"]
    editor_state: Arc<ViziaState>,

    #[id = "loop_start"]
    pub loop_start: FloatParam,

    #[id = "loop_end"]
    pub loop_end: FloatParam,

    #[id = "play_speed"]
    pub play_speed: FloatParam,

    #[id = "density"]
    pub density: FloatParam,

    #[id = "spray"]
    pub spray: FloatParam,

    #[id = "grain_length"]
    pub grain_length: FloatParam,
}

impl Default for Grainiac {
    fn default() -> Self {
        Self {
            params: Arc::new(GrainiacParams::default()),
            sampler: sampler::Sampler::new(48000.0),
        }
    }
}

impl Default for GrainiacParams {
    fn default() -> Self {
        Self {
            editor_state: editor::default_state(),

            loop_start: FloatParam::new(
                "Loop Start",
                0.25,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_value_to_string(formatters::v2s_f32_percentage(0)),

            loop_end: FloatParam::new("Loop End", 0.5, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_value_to_string(formatters::v2s_f32_percentage(0)),

            play_speed: FloatParam::new(
                "Play Speed",
                1.0,
                FloatRange::Linear { min: 0.0, max: 2.0 },
            ),

            density: FloatParam::new(
                "Density",
                0.25,
                FloatRange::Linear {
                    min: 0.5,
                    max: 20.0,
                },
            )
            .with_value_to_string(formatters::v2s_f32_hz_then_khz(2)),

            spray: FloatParam::new("Spray", 0.0, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_value_to_string(formatters::v2s_f32_percentage(0)),

            grain_length: FloatParam::new(
                "Grain Length",
                1.0,
                FloatRange::Linear { min: 0.1, max: 2.0 },
            )
            .with_unit(" sec"),
        }
    }
}

impl Plugin for Grainiac {
    const NAME: &'static str = "Grainiac";
    const VENDOR: &'static str = "Timerift";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "christian.grothe@posteo.de";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // The first audio IO layout is used as the default. The other layouts may be selected either
    // explicitly or automatically by the host or the user depending on the plugin API/backend.
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),

        aux_input_ports: &[],
        aux_output_ports: &[],

        // Individual ports and the layout as a whole can be named here. By default these names
        // are generated as needed. This layout will be called 'Stereo', while a layout with
        // only one input and output channel would be called 'Mono'.
        names: PortNames::const_default(),
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::Basic;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    // If the plugin can send or receive SysEx messages, it can define a type to wrap around those
    // messages here. The type implements the `SysExMessage` trait, which allows conversion to and
    // from plain byte buffers.
    type SysExMessage = ();
    // More advanced plugins can use this to run expensive background tasks. See the field's
    // documentation for more information. `()` means that the plugin does not have any background
    // tasks.
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(self.params.clone(), self.params.editor_state.clone())
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        // Resize buffers and perform other potentially expensive initialization operations here.
        // The `reset()` function is always called right after this function. You can remove this
        // function if you do not need it.
        true
    }

    fn reset(&mut self) {
        // Reset buffers and envelopes here. This can be called from the audio thread and may not
        // allocate. You can remove this function if you do not need it.
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        self.sampler
            .set_loop_start(0, self.params.loop_start.value());
        self.sampler.set_loop_end(0, self.params.loop_end.value());
        self.sampler
            .set_play_speed(0, self.params.play_speed.value());
        self.sampler.set_density(0, self.params.density.value());
        self.sampler.set_spray(0, self.params.spray.value());
        self.sampler
            .set_grain_length(0, self.params.grain_length.value());

        let next_event = context.next_event();

        match next_event {
            Some(event) => match event {
                NoteEvent::NoteOn { note, .. } => self.sampler.note_on(note as usize),
                NoteEvent::NoteOff { note, .. } => self.sampler.note_off(note as usize),
                NoteEvent::MidiCC { cc, value, .. } => match cc {
                    22 => {
                        if value > 0.0 {
                            self.sampler.record(0)
                        }
                    }
                    _ => {
                        nih_plug::nih_log!("{:?}", event)
                    }
                },
                _ => {
                    nih_plug::nih_log!("{:?}", event)
                }
            },
            _ => {}
        }

        for channels in buffer.iter_samples() {
            let mut sample_channels = channels.into_iter();
            let stereo_slice = (
                sample_channels.next().unwrap(),
                sample_channels.next().unwrap(),
            );
            self.sampler.render(stereo_slice);
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for Grainiac {
    const CLAP_ID: &'static str = "com.your-domain.grainiac";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A granular sampler instrument");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for Grainiac {
    const VST3_CLASS_ID: [u8; 16] = *b"Exactly16Chars!!";

    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Instrument,
        Vst3SubCategory::Sampler,
        Vst3SubCategory::Stereo,
    ];
}

nih_export_clap!(Grainiac);
nih_export_vst3!(Grainiac);

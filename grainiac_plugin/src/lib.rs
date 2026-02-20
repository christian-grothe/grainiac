use crossbeam::channel::{bounded, Receiver, Sender};
use grainiac_core::*;
use nih_plug::prelude::*;
use nih_plug_vizia::ViziaState;
use std::sync::{Arc, Mutex};

mod editor;
mod utils;

pub struct Grainiac {
    params: Arc<GrainiacParams>,
    sampler: Sampler,
    buf_output: Arc<Mutex<Output<Vec<DrawData>>>>,
    sender: Arc<Sender<FileMessage>>,
    receiver: Receiver<FileMessage>,
}

pub enum FileMessage {
    LoadAudio(Vec<f32>, usize),
    OpenFileDialog(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayDirection {
    Forward,
    Backward,
    BackAndForth,
}

impl Enum for PlayDirection {
    fn to_index(self) -> usize {
        match self {
            PlayDirection::Forward => 0,
            PlayDirection::Backward => 1,
            PlayDirection::BackAndForth => 2,
        }
    }

    fn from_index(index: usize) -> Self {
        if index == 0 {
            PlayDirection::Forward
        } else if index == 1 {
            PlayDirection::Backward
        } else {
            PlayDirection::BackAndForth
        }
    }

    fn ids() -> Option<&'static [&'static str]> {
        Some(&["forward", "backward", "back_and_forth"])
    }

    fn variants() -> &'static [&'static str] {
        &["", "", ""]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hold {
    On,
    Off,
}

impl Enum for Hold {
    fn to_index(self) -> usize {
        match self {
            Hold::Off => 0,
            Hold::On => 1,
        }
    }

    fn from_index(index: usize) -> Self {
        if index == 0 {
            Hold::Off
        } else {
            Hold::On
        }
    }

    fn ids() -> Option<&'static [&'static str]> {
        Some(&["off", "on"])
    }

    fn variants() -> &'static [&'static str] {
        &["", ""]
    }
}

#[derive(Params)]
struct InstanceParams {
    #[id = "loop_start"]
    pub loop_start: FloatParam,
    #[id = "loop_length"]
    pub loop_length: FloatParam,
    #[id = "play_speed"]
    pub play_speed: FloatParam,
    #[id = "density"]
    pub density: FloatParam,
    #[id = "spray"]
    pub spray: FloatParam,
    #[id = "grain_length"]
    pub grain_length: FloatParam,
    #[id = "attack"]
    pub attack: FloatParam,
    #[id = "release"]
    pub release: FloatParam,
    #[id = "pitch"]
    pub pitch: IntParam,
    #[id = "gain"]
    pub gain: FloatParam,
    #[id = "spread"]
    pub spread: FloatParam,
    #[id = "pan"]
    pub pan: FloatParam,
    #[id = "g_dir"]
    pub g_dir: EnumParam<PlayDirection>,
    #[id = "p_dir"]
    pub p_dir: EnumParam<PlayDirection>,
    #[id = "hold"]
    pub hold: EnumParam<Hold>,
}

impl InstanceParams {
    fn new() -> Self {
        InstanceParams {
            loop_start: FloatParam::new(
                "Loop Start",
                0.25,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            loop_length: FloatParam::new(
                "Loop End",
                0.25,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            play_speed: FloatParam::new(
                "Play Speed",
                1.0,
                FloatRange::Linear { min: 0.0, max: 2.0 },
            )
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

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
                .with_value_to_string(formatters::v2s_f32_rounded(2)),

            grain_length: FloatParam::new(
                "Grain Length",
                1.0,
                FloatRange::Linear { min: 0.1, max: 2.0 },
            )
            .with_value_to_string(formatters::v2s_f32_rounded(2))
            .with_unit(" sec"),

            attack: FloatParam::new("Attack", 0.01, FloatRange::Linear { min: 0.0, max: 5.0 })
                .with_value_to_string(formatters::v2s_f32_rounded(2))
                .with_unit(" sec"),

            release: FloatParam::new("Release", 0.01, FloatRange::Linear { min: 0.0, max: 5.0 })
                .with_value_to_string(formatters::v2s_f32_rounded(2))
                .with_unit(" sec"),

            pitch: IntParam::new("Pitch", 0, IntRange::Linear { min: -12, max: 12 })
                .with_unit(" st"),

            gain: FloatParam::new("Gain", 1.0, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_value_to_string(formatters::v2s_f32_rounded(2)),

            pan: FloatParam::new(
                "Pan",
                0.0,
                FloatRange::Linear {
                    min: -1.0,
                    max: 1.0,
                },
            )
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            spread: FloatParam::new("Spread", 0.5, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_value_to_string(formatters::v2s_f32_rounded(2)),

            g_dir: EnumParam::new("Play Direction", PlayDirection::Forward),

            p_dir: EnumParam::new("Grain Direction", PlayDirection::Forward),

            hold: EnumParam::new("Hold", Hold::Off),
        }
    }
}

#[derive(Params)]
struct GrainiacParams {
    #[persist = "editor-state"]
    editor_state: Arc<ViziaState>,

    #[nested(array, group = "instances")]
    instances: [InstanceParams; 2],
}

impl Default for Grainiac {
    fn default() -> Self {
        let (sampler, buf_output) = Sampler::new(48000.0, 2);
        let (sender, receiver) = bounded(1);

        Self {
            params: Arc::new(GrainiacParams::default()),
            sampler,
            buf_output: Arc::new(Mutex::new(buf_output)),
            sender: Arc::new(sender),
            receiver,
        }
    }
}

impl Default for GrainiacParams {
    fn default() -> Self {
        Self {
            editor_state: editor::default_state(),
            instances: [(); 2].map(|_| InstanceParams::new()),
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
        editor::create(
            self.params.clone(),
            self.params.editor_state.clone(),
            self.buf_output.clone(),
            self.sender.clone(),
        )
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
        for (i, instance) in self.params.instances.iter().enumerate() {
            self.sampler.set_loop_start(i, instance.loop_start.value());
            self.sampler
                .set_loop_length(i, instance.loop_length.value());
            self.sampler.set_play_speed(i, instance.play_speed.value());
            self.sampler.set_density(i, instance.density.value());
            self.sampler.set_spray(i, instance.spray.value());
            self.sampler
                .set_grain_length(i, instance.grain_length.value());
            self.sampler.set_attack(i, instance.attack.value());
            self.sampler.set_release(i, instance.release.value());
            self.sampler.set_gain(i, instance.gain.value());
            self.sampler
                .set_global_pitch(i, instance.pitch.value() as i8);
            self.sampler.set_pan(i, instance.pan.value());
            self.sampler.set_spread(i, instance.spread.value());
            self.sampler.set_grain_dir_from_preset(
                i,
                (instance.g_dir.unmodulated_normalized_value() * 3.0) as u8,
            );
            self.sampler.set_play_dir_from_preset(
                i,
                (instance.p_dir.unmodulated_normalized_value() * 3.0) as u8,
            );
        }

        if let Ok(msg) = self.receiver.try_recv() {
            match msg {
                FileMessage::LoadAudio(samples, index) => {
                    self.sampler.load_buf(samples, index);
                }
                _ => {}
            }
        }

        let mut next_event = context.next_event();
        while let Some(event) = next_event {
            match event {
                NoteEvent::NoteOn { note, .. } => self.sampler.note_on(note as usize),
                NoteEvent::NoteOff { note, .. } => self.sampler.note_off(note as usize),
                _ => {
                    nih_plug::nih_log!("{:?}", event)
                }
            }
            next_event = context.next_event();
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
    const VST3_CLASS_ID: [u8; 16] = *b"GrainiacGranular";

    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Instrument,
        Vst3SubCategory::Sampler,
        Vst3SubCategory::Stereo,
    ];
}

nih_export_clap!(Grainiac);
nih_export_vst3!(Grainiac);

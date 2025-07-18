use grainiac_core::{DrawData, Output, Sampler};
use nih_plug::prelude::*;
use nih_plug_vizia::ViziaState;
use std::sync::Arc;

mod editor;

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
    pub pitch: FloatParam,
    #[id = "gain"]
    pub gain: FloatParam,
    #[id = "spread"]
    pub spread: FloatParam,
    #[id = "pan"]
    pub pan: FloatParam,
}

impl InstanceParams {
    fn new() -> Self {
        InstanceParams {
            loop_start: FloatParam::new(
                "Loop Start",
                0.25,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_value_to_string(formatters::v2s_f32_percentage(0)),

            loop_length: FloatParam::new(
                "Loop End",
                0.25,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_value_to_string(formatters::v2s_f32_percentage(0)),

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
                .with_value_to_string(formatters::v2s_f32_percentage(0)),

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

            pitch: FloatParam::new("Pitch", 1.0, FloatRange::Linear { min: 0.5, max: 2.0 })
                .with_value_to_string(formatters::v2s_f32_rounded(2)),

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
        }
    }
}

#[derive(Params)]
struct GrainiacPluginParams {
    #[persist = "editor-state"]
    editor_state: Arc<ViziaState>,

    #[nested(array, group = "instances")]
    instances: [InstanceParams; 4],
}

struct GrainiacPlugin {
    params: Arc<GrainiacPluginParams>,
    sampler: Sampler,
    draw_data: Arc<Output<Vec<DrawData>>>,
}

impl Default for GrainiacPlugin {
    fn default() -> Self {
        let (sampler, draw_data) = Sampler::new(48000.0);

        Self {
            params: Arc::new(GrainiacPluginParams::default()),
            sampler,
            draw_data: Arc::new(draw_data),
        }
    }
}

impl Default for GrainiacPluginParams {
    fn default() -> Self {
        Self {
            editor_state: editor::default_state(),
            instances: [(); 4].map(|_| InstanceParams::new()),
        }
    }
}

impl Plugin for GrainiacPlugin {
    const NAME: &'static str = "Grainiac";
    const VENDOR: &'static str = "Christian Grothe";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "christian.grothe@posteo.de";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),

        aux_input_ports: &[],
        aux_output_ports: &[],

        names: PortNames::const_default(),
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        true
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(
            self.params.clone(),
            self.params.editor_state.clone(),
            self.draw_data.clone(),
        )
    }

    fn reset(&mut self) {}

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
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
            self.sampler.set_global_pitch(i, instance.pitch.value() as i8);
            self.sampler.set_pan(i, instance.pan.value());
            self.sampler.set_spread(i, instance.spread.value());
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

impl ClapPlugin for GrainiacPlugin {
    const CLAP_ID: &'static str = "com.christian-grothe.grainiac";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A granular sampler");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for GrainiacPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"123grainiac12345";

    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Instrument];
}

nih_export_clap!(GrainiacPlugin);
nih_export_vst3!(GrainiacPlugin);

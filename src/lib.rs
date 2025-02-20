use nih_plug::prelude::*;
use std::sync::Arc;

struct SlapDelay {
    params: Arc<SlapDelayParams>,
    delay_buffer: Vec<Vec<f32>>,
    write_pos: usize,
}

#[derive(Params)]
struct SlapDelayParams {
    #[id = "delay_time"]
    pub delay_time: FloatParam,

    #[id = "dry_wet"]
    pub dry_wet: FloatParam,
}

impl Default for SlapDelay {
    fn default() -> Self {
        Self {
            params: Arc::new(SlapDelayParams::default()),
            delay_buffer: vec![Vec::new(); 2], // Stereo buffer
            write_pos: 0,
        }
    }
}

impl Default for SlapDelayParams {
    fn default() -> Self {
        Self {
            delay_time: FloatParam::new(
                "Delay Time",
                120.0,
                FloatRange::Linear {
                    min: 1.0,
                    max: 1000.0,
                },
            )
            .with_unit(" ms")
            .with_value_to_string(formatters::v2s_f32_rounded(1)),

            dry_wet: FloatParam::new("Dry/Wet", 0.1, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_unit("")
                .with_value_to_string(Arc::new(|value| format!("{:.1}", value)))
                .with_step_size(0.001),
        }
    }
}

impl Plugin for SlapDelay {
    const NAME: &'static str = "Slap";
    const VENDOR: &'static str = "autoproduccionmusical.com";
    const URL: &'static str = "https://autoproduccionmusical.com";
    const EMAIL: &'static str = "info@autoproduccionmusical.com";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),
        ..AudioIOLayout::const_default()
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        // Calcular el máximo tamaño del buffer basado en 1001 ms
        let max_delay_samples = (buffer_config.sample_rate * 1.001) as usize;
        self.delay_buffer = vec![vec![0.0; max_delay_samples]; 2];
        self.write_pos = 0;
        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let delay_time_samples = (self.params.delay_time.smoothed.next()
            * 0.001
            * context.transport().sample_rate) as usize;
        // let delay_level = self.params.delay_level.smoothed.next();
        let dry_wet = self.params.dry_wet.value();

        for mut channel_samples in buffer.iter_samples() {
            for (channel_idx, sample) in channel_samples.iter_mut().enumerate() {
                // Write to delay buffer
                self.delay_buffer[channel_idx][self.write_pos] = *sample;

                // Calculate read position
                let read_pos = (self.write_pos + self.delay_buffer[channel_idx].len()
                    - delay_time_samples)
                    % self.delay_buffer[channel_idx].len();

                // Read from delay buffer
                let delayed_sample = self.delay_buffer[channel_idx][read_pos];

                // Mix dry and wet signals
                *sample = *sample * (1.0 - dry_wet) + delayed_sample * dry_wet;
            }

            // Increment and wrap write position
            self.write_pos = (self.write_pos + 1) % self.delay_buffer[0].len();
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for SlapDelay {
    const CLAP_ID: &'static str = "com.your-name.slap-delay";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A simple slap delay effect");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Delay,
    ];
}

impl Vst3Plugin for SlapDelay {
    const VST3_CLASS_ID: [u8; 16] = *b"SlapDelayPlugin_";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Delay];
}

nih_export_clap!(SlapDelay);
nih_export_vst3!(SlapDelay);
